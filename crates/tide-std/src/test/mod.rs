//! Property tests for driver-built libtest binaries.
//!
//! Parameterized tests generate 100 deterministic cases. Set `TIDE_TEST_SEED`
//! to a reported decimal seed to replay one case. Inputs need `Debug`.
//! Built-in collections generate up to 32 elements with four collection nesting
//! levels. Tuples support up to twelve elements. Implement `Arbitrary` to
//! choose a type's generation behavior.
//! Shrinking allows 1,000 evaluations and stops after 128 without improvement.
//!
//! `#[sim_test] fn t(sim: std::sim::SimRoot)` runs 100 seeded simulations in one child
//! process; `seeds = N` selects 0..N. `TIDE_SIM_SEED` selects one fresh seed internally.
//! Generated simulation parameters and shrinking remain planned.

mod arbitrary;
mod runner;

pub use arbitrary::{Arbitrary, Borrowed};
pub use runner::{property_result, run};
pub use tide_macros::{sim_test, test};

use std::collections::VecDeque;
use std::fmt::{Debug, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Integer(u128),
    Signed(u32),
    Float(u8),
    Index(usize),
    Bool,
    More,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Choice {
    value: u128,
    kind: Kind,
}

/// The source of choices for an [`Arbitrary`] implementation (provisional).
///
/// Draw through this generator so the same seed can replay the case and the
/// runner can shrink it by asking the implementation to draw again.
pub struct Gen {
    random: u64,
    replay: Option<VecDeque<Choice>>,
    choices: Vec<Choice>,
    input: String,
    collection_depth: u8,
    replay_remaining: usize,
    replay_overrun: bool,
}

struct ReplayOverrun;

impl Gen {
    fn new(seed: u64, replay: Option<Vec<Choice>>) -> Self {
        // Permit changed branches to expand, but never retry default draws forever.
        let replay_remaining = replay
            .as_ref()
            .map_or(0, |choices| choices.len().saturating_add(128));
        Self {
            random: seed,
            replay: replay.map(VecDeque::from),
            choices: Vec::new(),
            input: String::new(),
            collection_depth: 0,
            replay_remaining,
            replay_overrun: false,
        }
    }

    /// Choose one index in `0..choices`. Panics if there are no choices.
    pub fn index(&mut self, choices: usize) -> usize {
        assert!(choices > 0, "cannot draw from zero choices");
        self.draw(Kind::Index(choices), |g| {
            (g.next() % choices as u64) as u128
        }) as usize
    }

    #[doc(hidden)]
    pub fn record<T: Debug + ?Sized>(&mut self, name: &str, value: &T) {
        if !self.input.is_empty() {
            self.input.push_str(", ");
        }
        write!(self.input, "{name} = {value:?}").expect("formatting a property input failed");
    }

    fn next(&mut self) -> u64 {
        self.random = self.random.wrapping_add(0x9e3779b97f4a7c15);
        mix(self.random)
    }

    fn draw(&mut self, kind: Kind, generate: impl FnOnce(&mut Self) -> u128) -> u128 {
        let value = if let Some(replay) = &mut self.replay {
            if self.replay_remaining == 0 {
                self.replay_overrun = true;
                std::panic::panic_any(ReplayOverrun);
            }
            self.replay_remaining -= 1;
            // A deleted or changed branch can leave a draw with no matching
            // choice. Supply the simplest value and leave the next draw intact.
            if replay.front().is_some_and(|choice| choice.kind == kind) {
                replay.pop_front().unwrap().value
            } else {
                0
            }
        } else {
            generate(self)
        };
        self.choices.push(Choice { value, kind });
        value
    }

    fn integer(&mut self, bits: u32, signed: bool) -> u128 {
        let max = u128::MAX >> (128 - bits);
        let kind = if signed {
            Kind::Signed(bits)
        } else {
            Kind::Integer(max)
        };
        self.draw(kind, |g| match g.next() % 8 {
            0 => 0,
            1 => 1,
            2 => max,
            3 => max - 1,
            4 => u128::from(g.next() % 32),
            5 => 2,
            _ if bits <= 64 => u128::from(g.next()) & max,
            _ => (u128::from(g.next()) << 64) | u128::from(g.next()),
        })
    }

    fn collection<T: Arbitrary>(&mut self) -> Vec<T> {
        let mut values = Vec::new();
        // Bound recursive values such as a struct containing Vec<Self>.
        if self.collection_depth < 4 {
            self.collection_depth += 1;
            while values.len() < 32 && self.draw(Kind::More, |g| (g.next() % 4 != 0) as u128) != 0 {
                values.push(T::arbitrary(self));
            }
            self.collection_depth -= 1;
        }
        values
    }
}

// SplitMix64's output permutation. This can move to std::rand when it owns a PRNG.
fn mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}
