//! Randomness authority and its five draws (03 §1.1 RAND-DRAW, §9.3; research/53 §§2–5).
//!
//! Every draw is defined over 64-bit words. Real words are 8 bytes of Linux `getrandom(2)`
//! (flags 0) read little-endian through rustix, with no file fallback; an OS error other than
//! `EINTR`, which is retried, panics naming it. Simulated words are the simulation's `rand`
//! sub-stream, and each call is one scheduling point recording one `rand` event. Argument
//! checks panic before any word or scheduling point.

type Span = &'static std::panic::Location<'static>;

crate::capability!(
    /// Randomness authority: draws from the OS source, or from the seed under simulation, and
    /// the keys of `collections::hash_map::RandomState::from_rand`.
    Rand
);

mod sealed {
    /// Only `rand` can make one, so a program cannot call the methods below, even through a
    /// generic bound that names the public trait.
    pub struct Token(pub(super) ());

    /// The integer types `random_range` samples. `word` maps a value to its offset from the
    /// type's minimum, so every range is a span of `u64` offsets.
    pub trait Uniform: Copy + PartialOrd + std::fmt::Display {
        fn word(self, _: Token) -> u64;
        fn from_word(word: u64, _: Token) -> Self;
    }

    /// `a..b` and `a..=b`: `(low, s)` with `s` values, `s == 0` meaning all 2^64.
    pub trait Range<T> {
        fn span(&self, _: Token) -> (T, u64);
    }
}
use sealed::Token;

/// The integer types `Rand::random_range` samples: the integers up to 64 bits, `usize` and
/// `isize`. Sealed, with no callable method.
pub trait SampleUniform: sealed::Uniform {}

/// The ranges `Rand::random_range` takes: `a..b` and `a..=b`. Sealed, with no callable method.
pub trait SampleRange<T>: sealed::Range<T> {}

macro_rules! uniform {
    ($($t:ty),*) => {$(
        impl sealed::Uniform for $t {
            fn word(self, _: Token) -> u64 {
                (self as i128 - <$t>::MIN as i128) as u64
            }
            fn from_word(word: u64, _: Token) -> Self {
                (word as i128 + <$t>::MIN as i128) as $t
            }
        }
        impl SampleUniform for $t {}
    )*};
}
uniform!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

impl<T: SampleUniform> sealed::Range<T> for std::ops::Range<T> {
    #[track_caller]
    fn span(&self, _: Token) -> (T, u64) {
        assert!(self.start < self.end, "Rand::random_range: empty range {}..{}", self.start, self.end);
        (self.start, self.end.word(Token(())) - self.start.word(Token(())))
    }
}
impl<T: SampleUniform> SampleRange<T> for std::ops::Range<T> {}

impl<T: SampleUniform> sealed::Range<T> for std::ops::RangeInclusive<T> {
    #[track_caller]
    fn span(&self, _: Token) -> (T, u64) {
        // `is_empty` includes the exhausted state an iterated range keeps: `u8::MAX..=u8::MAX`
        // after one `next()` has endpoints `0..=255` but is empty.
        let (start, end) = (*self.start(), *self.end());
        assert!(!self.is_empty(), "Rand::random_range: empty range {start}..={end}");
        (start, (end.word(Token(())) - start.word(Token(()))).wrapping_add(1))
    }
}
impl<T: SampleUniform> SampleRange<T> for std::ops::RangeInclusive<T> {}

// research/53 §2.1: uniform over `s` values by rejection; `s == 0` is all 2^64, one word.
fn below(s: u64, next: &mut dyn FnMut() -> u64) -> u64 {
    if s == 0 {
        return next();
    }
    let threshold = s.wrapping_neg() % s;
    loop {
        let x = next();
        if x >= threshold {
            return x % s;
        }
    }
}

fn offset<T: SampleUniform>(low: T, s: u64, next: &mut dyn FnMut() -> u64) -> T {
    T::from_word(low.word(Token(())).wrapping_add(below(s, next)), Token(()))
}

fn shuffled<T>(slice: &mut [T], next: &mut dyn FnMut() -> u64) {
    for i in (1..slice.len()).rev() {
        slice.swap(i, below(i as u64 + 1, next) as usize);
    }
}

impl Rand {
    // The one dispatch between backends: real words, or one simulated scheduling point.
    fn draw<T>(
        &self,
        name: &'static str,
        span: Span,
        draw: impl FnOnce(&mut dyn FnMut() -> u64) -> T,
        result: impl FnOnce(&T) -> String,
    ) -> T {
        match &self.0 {
            Some(tasks) => tasks.rand_op(name, span, |next| {
                let value = draw(next);
                let text = result(&value);
                (value, text)
            }),
            None => {
                crate::task::outside_simulation();
                draw(&mut os::word)
            }
        }
    }

    /// One word: 64 bits from the OS, or the simulation's next word.
    #[track_caller]
    pub fn next_u64(&self) -> u64 {
        self.draw("rand.next_u64", std::panic::Location::caller(), |next| next(), |x| format!(",\"result\":{x}"))
    }

    /// Fills `dest`: OS bytes, or `ceil(len / 8)` simulated words little-endian with the last
    /// truncated. An empty `dest` takes no word.
    #[track_caller]
    pub fn fill_bytes(&self, dest: &mut [u8]) {
        let len = dest.len();
        let span = std::panic::Location::caller();
        match &self.0 {
            Some(tasks) => tasks.rand_op("rand.fill_bytes", span, |next| {
                for chunk in dest.chunks_mut(8) {
                    chunk.copy_from_slice(&next().to_le_bytes()[..chunk.len()]);
                }
                ((), format!(",\"len\":{len}"))
            }),
            None => {
                crate::task::outside_simulation();
                os::fill(dest);
            }
        }
    }

    /// A uniform value of `a..b` or `a..=b`, by rejection, never modulo bias. An empty range
    /// panics before drawing.
    #[track_caller]
    pub fn random_range<T: SampleUniform, R: SampleRange<T>>(&self, range: R) -> T {
        let (low, s) = range.span(Token(()));
        let span = std::panic::Location::caller();
        self.draw("rand.random_range", span, |next| offset(low, s, next), |x| format!(",\"result\":{x}"))
    }

    /// `true` with probability `floor(p * 2^64) / 2^64`, and always for `p == 1.0`; one word.
    /// `p` outside `[0.0, 1.0]`, NaN included, panics before drawing.
    #[track_caller]
    pub fn random_bool(&self, p: f64) -> bool {
        assert!((0.0..=1.0).contains(&p), "Rand::random_bool: p = {p} is outside [0.0, 1.0]");
        let threshold = (p * 18_446_744_073_709_551_616.0) as u64;
        let span = std::panic::Location::caller();
        self.draw("rand.random_bool", span, |next| next() < threshold || p == 1.0, |x| format!(",\"result\":{x}"))
    }

    /// Fisher-Yates from the end over `random_range(0..=i)`. A slice of length 0 or 1 takes no
    /// word.
    #[track_caller]
    pub fn shuffle<T>(&self, slice: &mut [T]) {
        let len = slice.len();
        let span = std::panic::Location::caller();
        self.draw("rand.shuffle", span, |next| shuffled(slice, next), |()| format!(",\"len\":{len}"))
    }

    /// `RandomState::from_rand`'s simulated key prefix: two words, one scheduling point and one
    /// `rand` event (research/53 §5). `None` for a real `Rand`, after the identity check.
    pub(crate) fn simulated_keys(&self, span: Span) -> Option<(u64, u64)> {
        match &self.0 {
            Some(tasks) => Some(tasks.rand_op("rand.from_rand", span, |next| ((next(), next()), String::new()))),
            None => {
                crate::task::outside_simulation();
                None
            }
        }
    }
}

// The real source: `getrandom(2)` with flags 0, the only OS call a draw makes.
mod os {
    pub(super) fn word() -> u64 {
        let mut bytes = [0; 8];
        fill(&mut bytes);
        u64::from_le_bytes(bytes)
    }

    pub(super) fn fill(dest: &mut [u8]) {
        let mut filled = 0;
        while filled < dest.len() {
            match getrandom(&mut dest[filled..]) {
                Ok(n) => filled += n,
                Err(rustix::io::Errno::INTR) => {}
                Err(e) => panic!("Rand: getrandom(2) failed with {}: {e}", name(e)),
            }
        }
    }

    fn name(e: rustix::io::Errno) -> String {
        use rustix::io::Errno;
        match e {
            Errno::PERM => "EPERM".into(),
            Errno::NOSYS => "ENOSYS".into(),
            Errno::FAULT => "EFAULT".into(),
            Errno::INVAL => "EINVAL".into(),
            Errno::AGAIN => "EAGAIN".into(),
            e => format!("errno {}", e.raw_os_error()),
        }
    }

    // Flags 0 (research/53 §3): block only until the pool is first initialized. The seam
    // receives the same flags, so RAND-4's seam test checks what the system call gets.
    const FLAGS: rustix::rand::GetRandomFlags = rustix::rand::GetRandomFlags::empty();

    fn getrandom(dest: &mut [u8]) -> rustix::io::Result<usize> {
        #[cfg(test)]
        if let Some(reply) = seam::call(dest, FLAGS) {
            return reply;
        }
        rustix::rand::getrandom(dest, FLAGS)
    }

    /// RAND-4's seam: while a test holds one on this thread, it answers in place of the
    /// system call, in order, and records each request's length and flags.
    #[cfg(test)]
    pub(super) mod seam {
        use std::cell::RefCell;
        use std::collections::VecDeque;

        pub(crate) enum Reply {
            Bytes(Vec<u8>),
            Fail(rustix::io::Errno),
        }

        thread_local! {
            static SEAM: RefCell<Option<(VecDeque<Reply>, Vec<Request>)>> = const { RefCell::new(None) };
        }

        pub(crate) type Request = (usize, rustix::rand::GetRandomFlags);

        /// Runs `f` with the seam answering; returns its value and the requests.
        pub(crate) fn with<T>(replies: Vec<Reply>, f: impl FnOnce() -> T) -> (T, Vec<Request>) {
            SEAM.with(|s| *s.borrow_mut() = Some((replies.into(), Vec::new())));
            let value = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
            let (left, requests) = SEAM.with(|s| s.borrow_mut().take()).unwrap();
            let value = value.unwrap_or_else(|p| std::panic::resume_unwind(p));
            assert!(left.is_empty(), "unused seam replies");
            (value, requests)
        }

        pub(super) fn call(dest: &mut [u8], flags: rustix::rand::GetRandomFlags) -> Option<rustix::io::Result<usize>> {
            SEAM.with(|s| {
                let mut seam = s.borrow_mut();
                let (replies, requests) = seam.as_mut()?;
                requests.push((dest.len(), flags));
                Some(match replies.pop_front().expect("the seam ran out of replies") {
                    Reply::Bytes(bytes) => {
                        let n = bytes.len().min(dest.len());
                        dest[..n].copy_from_slice(&bytes[..n]);
                        Ok(n)
                    }
                    Reply::Fail(e) => Err(e),
                })
            })
        }
    }
}

#[cfg(test)]
mod tests;
