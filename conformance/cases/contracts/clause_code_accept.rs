//@ rules: CLAUSE-2
// A clause is a Rust boolean expression typed as code, and names resolve as in the
// body, so a `pub` function's clause reads private fields and calls private functions.
pub struct Account {
    balance: u64,
}

fn solvent(a: &Account) -> bool {
    a.balance > 0
}

impl Account {
    #[requires(solvent(self))]
    #[ensures(self.balance == old(self.balance) - amount)]
    pub fn withdraw(&mut self, amount: u64) {
        self.balance -= amount;
    }
}

#[ensures(result == a as u16 + b as u16)]
pub fn widen(a: u8, b: u8) -> u16 {
    a as u16 + b as u16
}
