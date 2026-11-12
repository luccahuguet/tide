use crate::task::{Channel, Signal, Tasks};
use crate::time::Instant;
use std::cell::Cell;
use std::collections::VecDeque;
use std::fmt;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

pub use std::sync::mpsc::{TryRecvError, TrySendError};
type Span = &'static std::panic::Location<'static>;

/// An unbounded FIFO channel. Sending does not check cancellation or block.
#[track_caller]
pub fn channel<T: Send>(tasks: &Tasks) -> (Sender<T>, Receiver<T>) {
    let inner = Chan::new(None, tasks, std::panic::Location::caller());
    (Sender(inner.clone()), Receiver(inner, PhantomData))
}

/// A bounded FIFO channel; zero capacity is a rendezvous. Every `send` is a checkpoint.
#[track_caller]
pub fn sync_channel<T: Send>(bound: usize, tasks: &Tasks) -> (SyncSender<T>, Receiver<T>) {
    let inner = Chan::new(Some(bound), tasks, std::panic::Location::caller());
    (SyncSender(inner.clone()), Receiver(inner, PhantomData))
}

pub struct Sender<T>(Arc<Chan<T>>);
pub struct SyncSender<T>(Arc<Chan<T>>);
/// The single consumer. It can move between tasks, but cannot be shared between them.
pub struct Receiver<T>(Arc<Chan<T>>, PhantomData<Cell<()>>);

struct Chan<T> {
    runtime: Channel,
    bound: Option<usize>,
    state: Mutex<State<T>>,
}

struct State<T> {
    buffer: VecDeque<T>,
    pending: VecDeque<Pending<T>>,
    receiver: Option<Arc<Signal>>,
    senders: usize,
    alive: bool,
    next: usize,
}

struct Pending<T> {
    ticket: usize,
    value: T,
    signal: Arc<Signal>,
}

impl<T> State<T> {
    fn wake_receiver(&self) {
        if let Some(receiver) = &self.receiver {
            receiver.wake();
        }
    }

    fn take(&mut self, bound: Option<usize>) -> Option<T> {
        if let Some(value) = self.buffer.pop_front() {
            if bound != Some(0) {
                if let Some(pending) = self.pending.pop_front() {
                    self.buffer.push_back(pending.value);
                    pending.signal.wake();
                }
            }
            Some(value)
        } else {
            self.pending.pop_front().map(|pending| {
                pending.signal.wake();
                pending.value
            })
        }
    }
}

impl<T> Chan<T> {
    fn new(bound: Option<usize>, tasks: &Tasks, span: Span) -> Arc<Self> {
        Arc::new(Self {
            runtime: Channel::new(tasks, bound, span),
            bound,
            state: Mutex::new(State {
                buffer: VecDeque::new(),
                pending: VecDeque::new(),
                receiver: None,
                senders: 1,
                alive: true,
                next: 0,
            }),
        })
    }

    fn clone_sender(&self) {
        self.state.lock().unwrap().senders += 1;
    }

    fn drop_sender(&self, span: Span) {
        self.runtime.close("sender", span, || {
            let mut state = self.state.lock().unwrap();
            state.senders -= 1;
            if state.senders == 0 {
                state.wake_receiver();
            }
            ((), Some(state.senders))
        });
    }

    fn send(&self, value: T, span: Span) -> Result<(), SendError<T>> {
        let context = self.runtime.begin("send", span);
        let result = (|| {
            let pending = self.runtime.with(&context, |cancelled| {
                let mut state = self.state.lock().unwrap();
                if self.bound.is_some() && cancelled() {
                    return Err(SendError::Cancelled(value));
                }
                if !state.alive {
                    return Err(SendError::Disconnected(value));
                }
                if state.pending.is_empty() && self.bound.is_none_or(|n| state.buffer.len() < n) {
                    state.buffer.push_back(value);
                    state.wake_receiver();
                    return Ok(None);
                }
                let ticket = state.next;
                state.next += 1;
                let signal = Arc::new(Signal::new());
                state.pending.push_back(Pending {
                    ticket,
                    value,
                    signal: signal.clone(),
                });
                state.wake_receiver();
                Ok(Some((ticket, signal)))
            })?;
            let Some((ticket, signal)) = pending else {
                return Ok(());
            };
            loop {
                self.runtime
                    .wait(&context, true, span, None, signal.clone());
                let result = self.runtime.with(&context, |cancelled| {
                    let mut state = self.state.lock().unwrap();
                    let Some(index) = state.pending.iter().position(|p| p.ticket == ticket) else {
                        // Acceptance into the buffer or by the receiver preceded cancellation.
                        return Some(Ok(()));
                    };
                    let cancelled = cancelled();
                    if cancelled || !state.alive {
                        let value = state.pending.remove(index).unwrap().value;
                        Some(Err(if cancelled {
                            SendError::Cancelled(value)
                        } else {
                            SendError::Disconnected(value)
                        }))
                    } else {
                        None
                    }
                });
                if let Some(result) = result {
                    return result;
                }
            }
        })();
        self.runtime.record(
            &context,
            "send",
            span,
            match &result {
                Ok(()) => "ok",
                Err(SendError::Cancelled(_)) => "cancelled",
                Err(SendError::Disconnected(_)) => "disconnected",
            },
            None,
        );
        result
    }
}

impl<T> Sender<T> {
    #[track_caller]
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.0.send(value, std::panic::Location::caller())
    }
}

impl<T> SyncSender<T> {
    #[track_caller]
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.0.send(value, std::panic::Location::caller())
    }

    /// Never blocks or checks cancellation. A rendezvous needs a blocked receiver.
    #[track_caller]
    pub fn try_send(&self, value: T) -> Result<(), TrySendError<T>> {
        let span = std::panic::Location::caller();
        let context = self.0.runtime.begin("try_send", span);
        let result = self.0.runtime.with(&context, |_| {
            let mut state = self.0.state.lock().unwrap();
            if !state.alive {
                return Err(TrySendError::Disconnected(value));
            }
            let bound = self.0.bound.unwrap();
            let room = if bound == 0 {
                state.receiver.is_some() && state.buffer.is_empty()
            } else {
                state.buffer.len() < bound
            };
            if !room || !state.pending.is_empty() {
                return Err(TrySendError::Full(value));
            }
            state.buffer.push_back(value);
            state.wake_receiver();
            Ok(())
        });
        self.0.runtime.record(
            &context,
            "try_send",
            span,
            match &result {
                Ok(()) => "ok",
                Err(TrySendError::Full(_)) => "full",
                Err(TrySendError::Disconnected(_)) => "disconnected",
            },
            None,
        );
        result
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.0.clone_sender();
        Self(self.0.clone())
    }
}
impl<T> Clone for SyncSender<T> {
    fn clone(&self) -> Self {
        self.0.clone_sender();
        Self(self.0.clone())
    }
}
impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.0.drop_sender(std::panic::Location::caller());
    }
}
impl<T> Drop for SyncSender<T> {
    fn drop(&mut self) {
        self.0.drop_sender(std::panic::Location::caller());
    }
}
impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        let values = self
            .0
            .runtime
            .close("receiver", std::panic::Location::caller(), || {
                let mut state = self.0.state.lock().unwrap();
                state.alive = false;
                for pending in &state.pending {
                    pending.signal.wake();
                }
                // User destructors may use channels or unwind. Run them with both locks released.
                (std::mem::take(&mut state.buffer), None)
            });
        drop(values);
    }
}

impl<T> Receiver<T> {
    /// Checks cancellation before consuming buffered values, except an already accepted
    /// zero-capacity `try_send` hand-off.
    #[track_caller]
    pub fn recv(&self) -> Result<T, RecvError> {
        self.receive(None, "recv", std::panic::Location::caller())
            .map_err(|e| match e {
                RecvTimeoutError::Cancelled => RecvError::Cancelled,
                RecvTimeoutError::Disconnected => RecvError::Disconnected,
                RecvTimeoutError::Timeout => unreachable!("untimed receive timed out"),
            })
    }

    /// A ready value beats the deadline; cancellation beats both, except an already
    /// accepted zero-capacity `try_send` hand-off.
    #[track_caller]
    pub fn recv_deadline(&self, deadline: Instant) -> Result<T, RecvTimeoutError> {
        self.receive(
            Some(deadline),
            "recv_deadline",
            std::panic::Location::caller(),
        )
    }

    fn receive(
        &self,
        deadline: Option<Instant>,
        name: &'static str,
        span: Span,
    ) -> Result<T, RecvTimeoutError> {
        let context = self.0.runtime.begin(name, span);
        let mut timed_out = false;
        let result = loop {
            let mut waiter = None;
            let result = self.0.runtime.with(&context, |cancelled| {
                let mut state = self.0.state.lock().unwrap();
                state.receiver = None;
                if self.0.bound == Some(0) {
                    // A zero-capacity try_send already handed this value to this receiver.
                    if let Some(value) = state.buffer.pop_front() {
                        return Some(Ok(value));
                    }
                }
                if cancelled() {
                    return Some(Err(RecvTimeoutError::Cancelled));
                }
                if let Some(value) = state.take(self.0.bound) {
                    return Some(Ok(value));
                }
                if state.senders == 0 {
                    return Some(Err(RecvTimeoutError::Disconnected));
                }
                if timed_out {
                    return Some(Err(RecvTimeoutError::Timeout));
                }
                let signal = Arc::new(Signal::new());
                state.receiver = Some(signal.clone());
                waiter = Some(signal);
                None
            });
            if let Some(result) = result {
                break result;
            }
            timed_out = self.0.runtime.wait(
                &context,
                false,
                span,
                deadline.map(|t| t.0),
                waiter.unwrap(),
            );
        };
        self.0.runtime.record(
            &context,
            name,
            span,
            match &result {
                Ok(_) => "ok",
                Err(RecvTimeoutError::Cancelled) => "cancelled",
                Err(RecvTimeoutError::Disconnected) => "disconnected",
                Err(RecvTimeoutError::Timeout) => "timeout",
            },
            deadline.map(|t| t.0),
        );
        result
    }

    #[track_caller]
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        let span = std::panic::Location::caller();
        let context = self.0.runtime.begin("try_recv", span);
        let result = self.0.runtime.with(&context, |_| {
            let mut state = self.0.state.lock().unwrap();
            state.take(self.0.bound).ok_or(if state.senders == 0 {
                TryRecvError::Disconnected
            } else {
                TryRecvError::Empty
            })
        });
        self.0.runtime.record(
            &context,
            "try_recv",
            span,
            match &result {
                Ok(_) => "ok",
                Err(TryRecvError::Empty) => "empty",
                Err(TryRecvError::Disconnected) => "disconnected",
            },
            None,
        );
        result
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter { receiver: self }
    }
    pub fn try_iter(&self) -> TryIter<'_, T> {
        TryIter { receiver: self }
    }
}

pub struct Iter<'a, T> {
    receiver: &'a Receiver<T>,
}
pub struct TryIter<'a, T> {
    receiver: &'a Receiver<T>,
}
pub struct IntoIter<T> {
    receiver: Receiver<T>,
}
impl<T> Iterator for Iter<'_, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.receiver.recv().ok()
    }
}
impl<T> Iterator for TryIter<'_, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.receiver.try_recv().ok()
    }
}
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.receiver.recv().ok()
    }
}
impl<T> IntoIterator for Receiver<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { receiver: self }
    }
}
impl<'a, T> IntoIterator for &'a Receiver<T> {
    type Item = T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecvError {
    Disconnected,
    Cancelled,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecvTimeoutError {
    Timeout,
    Disconnected,
    Cancelled,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SendError<T> {
    Disconnected(T),
    Cancelled(T),
}

impl RecvError {
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }
}
impl RecvTimeoutError {
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }
}
impl<T> SendError<T> {
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled(_))
    }
    pub fn into_inner(self) -> T {
        match self {
            Self::Disconnected(v) | Self::Cancelled(v) => v,
        }
    }
}
impl From<RecvError> for RecvTimeoutError {
    fn from(e: RecvError) -> Self {
        match e {
            RecvError::Disconnected => Self::Disconnected,
            RecvError::Cancelled => Self::Cancelled,
        }
    }
}
impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Disconnected => "receiving on a closed channel",
            Self::Cancelled => "channel receive cancelled",
        })
    }
}
impl fmt::Display for RecvTimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Timeout => "timed out waiting on channel",
            Self::Disconnected => "receiving on a closed channel",
            Self::Cancelled => "channel receive cancelled",
        })
    }
}
impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Disconnected(_) => "sending on a closed channel",
            Self::Cancelled(_) => "channel send cancelled",
        })
    }
}
impl std::error::Error for RecvError {}
impl std::error::Error for RecvTimeoutError {}
// Like Rust's send error, formatting does not require a Debug payload.
impl<T> fmt::Debug for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(match self {
            Self::Disconnected(_) => "Disconnected",
            Self::Cancelled(_) => "Cancelled",
        })
        .finish_non_exhaustive()
    }
}
impl<T> std::error::Error for SendError<T> {}

impl<T> fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sender").finish_non_exhaustive()
    }
}
impl<T> fmt::Debug for SyncSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyncSender").finish_non_exhaustive()
    }
}
impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Receiver").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    include!("mpsc/tests.rs");
}
