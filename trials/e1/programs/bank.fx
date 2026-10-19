// src/lib.fx of package `trials/bank`
//
// Accounts live in `Shared` cells. A transfer locks both cells at once
// (`lock` acquires in a global order, so it cannot deadlock), refuses
// overdrafts, and either moves the whole amount or changes nothing.

use std::sync::{Poisoned, Shared};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Account {
    balance: u64,
}

#[derive(Debug, Error)]
pub enum BankError {
    #[error("no account {0}")]
    NoSuchAccount(usize),
    #[error("cannot transfer from account {0} to itself")]
    SameAccount(usize),
    #[error("insufficient funds: balance {balance}, requested {amount}")]
    Insufficient { balance: u64, amount: u64 },
    #[error("an account cell is poisoned")]
    Poisoned(#[from] Poisoned),
}

pub struct Bank {
    accounts: Vec<Shared<Account>>,
}

impl Bank {
    /// Panics if the balances sum past `u64::MAX`; after that, transfers
    /// conserve the total, so no credit can overflow.
    pub fn new(balances: &[u64]) -> Bank {
        balances
            .iter()
            .try_fold(0u64, |sum, &b| sum.checked_add(b))
            .expect("total balance fits in u64");
        Bank {
            accounts: balances.iter().map(|&balance| Shared::new(Account { balance })).collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.accounts.len()
    }

    fn cell(&self, id: usize) -> Result<&Shared<Account>, BankError> {
        self.accounts.get(id).ok_or(BankError::NoSuchAccount(id))
    }

    pub fn balance(&self, id: usize) -> Result<u64, BankError> {
        let cell = self.cell(id)?;
        let balance = lock (cell) |acct| { acct.balance }?;
        Ok(balance)
    }

    /// Moves `amount` from `from` to `to` atomically, or returns an error and
    /// changes nothing.
    pub fn transfer(&self, from: usize, to: usize, amount: u64) -> Result<(), BankError> {
        if from == to {
            // Two handles to one cell in a single `lock` would panic; refuse instead.
            return Err(BankError::SameAccount(from));
        }
        let src = self.cell(from)?;
        let dst = self.cell(to)?;
        lock (src, dst) |s, d| {
            if s.balance < amount {
                return Err(BankError::Insufficient { balance: s.balance, amount }); // leaves the body
            }
            s.balance -= amount;
            d.balance += amount;
            Ok(())
        }? // outer `?`: Err(Poisoned) -> BankError::Poisoned
    }

    /// Sum of all balances. Locks one cell at a time, so it is exact only when
    /// no transfer is running concurrently.
    pub fn total(&self) -> Result<u64, BankError> {
        let mut sum: u64 = 0;
        for cell in &self.accounts {
            sum += lock (cell) |acct| { acct.balance }?;
        }
        Ok(sum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::task::TaskError;

    const START: [u64; 5] = [100, 0, 250, 40, 10];

    #[test]
    fn moves_money() {
        let bank = Bank::new(&START);
        bank.transfer(0, 1, 30).unwrap();
        assert_eq!(bank.balance(0).unwrap(), 70);
        assert_eq!(bank.balance(1).unwrap(), 30);
    }

    #[test]
    fn overdraft_is_refused_and_changes_nothing() {
        let bank = Bank::new(&START);
        let r = bank.transfer(4, 0, 11);
        assert!(matches!(r, Err(BankError::Insufficient { balance: 10, amount: 11 })));
        assert_eq!(bank.balance(4).unwrap(), 10);
        assert_eq!(bank.balance(0).unwrap(), 100);
    }

    #[test]
    fn bad_accounts_are_refused() {
        let bank = Bank::new(&START);
        assert!(matches!(bank.transfer(2, 2, 1), Err(BankError::SameAccount(2))));
        assert!(matches!(bank.transfer(0, 9, 1), Err(BankError::NoSuchAccount(9))));
    }

    /// Property: random transfers from four concurrent tasks never create or
    /// destroy money. The seed picks the interleaving; `transfers` is generated
    /// and shrunk on failure.
    #[test]
    fn concurrent_transfers_conserve_total(seed: u64, transfers: Vec<(u8, u8, u16)>) {
        let sim = Root::sim(seed);
        let bank = Bank::new(&START);
        let before = bank.total().unwrap();
        let n = bank.len();
        let chunk_len = transfers.len().div_ceil(4).max(1);

        let run: Result<(), TaskError> = sim.tasks.scope(|s| {
            for chunk in transfers.chunks(chunk_len) {
                let bank = &bank;
                s.spawn(move || {
                    for &(from, to, amount) in chunk {
                        let from = usize::from(from) % n;
                        let to = usize::from(to) % n;
                        match bank.transfer(from, to, u64::from(amount)) {
                            Ok(()) | Err(BankError::Insufficient { .. }) | Err(BankError::SameAccount(_)) => {}
                            Err(e) => panic!("unexpected error: {e}"),
                        }
                    }
                    Ok(())
                });
            }
            Ok(())
        });
        run.unwrap();

        assert_eq!(bank.total().unwrap(), before);
    }
}
