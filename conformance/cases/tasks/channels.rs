//@ rules: SCOPE-1, SCOPE-3, SEND-3
// The driver executes this surface in debug/release, on both runtimes.
use std::sync::mpsc::{
    RecvError, RecvTimeoutError, SendError, TryRecvError, TrySendError, channel, sync_channel,
};
use std::task::{TaskError, Tasks};
use std::time::{Clock, Duration};

fn exercise(tasks: &Tasks, clock: &Clock) -> Result<(), TaskError> {
    let (tx, rx) = channel(tasks);
    tx.send(11).unwrap();
    tx.clone().send(12).unwrap();
    assert_eq!(rx.try_iter().collect::<Vec<_>>(), vec![11, 12]);
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
    tx.send(13).unwrap();
    drop(tx);
    assert_eq!(rx.iter().collect::<Vec<_>>(), vec![13]);
    assert_eq!(rx.recv(), Err(RecvError::Disconnected));
    assert!(!RecvError::Disconnected.is_cancelled());
    assert!(RecvError::Cancelled.is_cancelled());
    assert_eq!(
        RecvTimeoutError::from(RecvError::Cancelled),
        RecvTimeoutError::Cancelled
    );
    assert_eq!(
        RecvTimeoutError::from(RecvError::Disconnected),
        RecvTimeoutError::Disconnected
    );
    assert!(RecvTimeoutError::Cancelled.is_cancelled());
    assert!(!RecvTimeoutError::Timeout.is_cancelled());
    assert!(SendError::Cancelled(4).is_cancelled());
    assert!(!SendError::Disconnected(4).is_cancelled());
    assert_eq!(SendError::Cancelled(4).into_inner(), 4);
    assert_eq!(SendError::Disconnected(5).into_inner(), 5);
    assert!(!RecvError::Cancelled.to_string().is_empty());
    assert!(!RecvTimeoutError::Timeout.to_string().is_empty());
    assert!(!SendError::Cancelled(0).to_string().is_empty());

    let (tx, rx) = sync_channel(1, tasks);
    tx.try_send(21).unwrap();
    assert_eq!(tx.try_send(22), Err(TrySendError::Full(22)));
    let copy = tx.clone();
    tasks.scope(|s| {
        s.spawn(move || {
            copy.send(22).unwrap();
            Ok(())
        });
        // In simulation the producer must block before time can advance.
        clock.sleep(Duration::from_millis(1)).unwrap();
        assert_eq!(rx.recv().unwrap(), 21);
        assert_eq!(rx.recv().unwrap(), 22);
        Ok(())
    })?;
    assert_eq!(
        rx.recv_deadline(clock.now()),
        Err(RecvTimeoutError::Timeout)
    );
    let until = clock.now() + Duration::from_millis(1);
    assert_eq!(rx.recv_deadline(until), Err(RecvTimeoutError::Timeout));
    assert!(clock.now() >= until);
    tx.send(23).unwrap();
    assert_eq!(rx.recv_deadline(until).unwrap(), 23);
    drop(tx);
    assert_eq!(
        rx.recv_deadline(clock.now()),
        Err(RecvTimeoutError::Disconnected)
    );

    let (tx, rx) = sync_channel(0, tasks);
    assert_eq!(tx.try_send(31), Err(TrySendError::Full(31)));
    tasks.scope(|s| {
        s.spawn(move || {
            tx.send(32).unwrap();
            tx.send(33).unwrap();
            Ok(())
        });
        assert_eq!(rx.into_iter().collect::<Vec<_>>(), vec![32, 33]);
        Ok(())
    })?;
    let (tx, rx) = channel(tasks);
    tx.send(34).unwrap();
    drop(tx);
    assert_eq!((&rx).into_iter().collect::<Vec<_>>(), vec![34]);

    let (tx, rx) = sync_channel(1, tasks);
    tx.send(41).unwrap();
    tasks.scope(|s| {
        s.cancel();
        assert_eq!(rx.recv(), Err(RecvError::Cancelled));
        assert_eq!(
            rx.recv_deadline(clock.now()),
            Err(RecvTimeoutError::Cancelled)
        );
        assert_eq!(tx.send(42), Err(SendError::Cancelled(42)));
        assert_eq!(rx.iter().next(), None);
        assert_eq!(rx.try_recv(), Ok(41));
        assert_eq!(tx.try_send(43), Ok(()));
        assert_eq!(rx.try_recv(), Ok(43));
        // A free bounded slot still checks cancellation.
        assert_eq!(tx.send(44), Err(SendError::Cancelled(44)));
        let (unbounded, values) = channel(tasks);
        unbounded.send(45).unwrap();
        assert_eq!(values.try_recv(), Ok(45));
        Ok(())
    })?;

    // Deadlines cancel blocked send and receive operations through the shared wait primitive.
    tasks.scope(|s| {
        s.set_deadline(clock.now() + Duration::from_millis(1));
        assert_eq!(rx.recv(), Err(RecvError::Cancelled));
        Ok(())
    })?;
    tasks.scope(|s| {
        let at = clock.now() + Duration::from_millis(1);
        s.set_deadline(at);
        assert_eq!(rx.recv_deadline(at), Err(RecvTimeoutError::Cancelled));
        Ok(())
    })?;
    tx.send(51).unwrap();
    tasks.scope(|s| {
        s.set_deadline(clock.now() + Duration::from_millis(1));
        assert_eq!(tx.send(52), Err(SendError::Cancelled(52)));
        Ok(())
    })?;
    assert_eq!(rx.try_recv(), Ok(51));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
    drop(rx);
    assert_eq!(tx.send(53), Err(SendError::Disconnected(53)));
    assert_eq!(tx.try_send(54), Err(TrySendError::Disconnected(54)));
    let (tx, rx) = channel(tasks);
    drop(rx);
    assert_eq!(tx.send(55), Err(SendError::Disconnected(55)));
    Ok(())
}

#[tide::main]
fn main(root: Root) -> Result<(), TaskError> {
    exercise(&root.tasks, &root.clock)?;
    println!("CHANNELS_RAN");
    Ok(())
}

#[cfg(test)]
fn observed(seed: u64) -> String {
    let sim = Root::sim(seed);
    exercise(&sim.tasks, &sim.clock).unwrap();
    let trace = sim.trace();
    for operation in [
        "new",
        "send",
        "try_send",
        "recv",
        "try_recv",
        "recv_deadline",
        "close",
    ] {
        assert!(
            trace.contains(&format!("\"kind\":\"chan\",\"operation\":\"{operation}\"")),
            "{trace}"
        );
    }
    assert!(trace.contains("\"operation\":\"send\",\"target\":1"));
    assert!(trace.contains("\"wait\":\"blocked\""));
    assert!(!trace.contains("\"payload\""));
    trace
}

#[test]
fn simulated_channels() {
    for seed in 0..12 {
        assert_eq!(observed(seed), observed(seed));
    }
}
