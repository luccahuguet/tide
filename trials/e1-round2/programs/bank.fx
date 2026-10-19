//! Accounts in shared cells. A transfer moves money between two accounts as
//! one atomic step and refuses overdrafts.

use std::sync::{LockError, Shared};
use std::task::{TaskError, Tasks};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: usize,
    pub balance: u64,
}

#[derive(Debug, Error)]
pub enum BankError {
    #[error("account {id} holds {balance}, cannot send {amount}")]
    Overdraft { id: usize, balance: u64, amount: u64 },
    #[error("account {id} cannot hold {amount} more")]
    Overflow { id: usize, amount: u64 },
    #[error("an account cannot transfer to itself")]
    SameAccount,
    #[error("account lock failed")]
    Lock(#[from] LockError),
    #[error("transfer task failed")]
    Task(#[from] TaskError),
}

/// Moves `amount` from `from` to `to`. Both cells are held together, so no
/// task ever sees the money in both accounts or in neither. A refused
/// transfer leaves both balances unchanged.
pub fn transfer(from: &Shared<Account>, to: &Shared<Account>, amount: u64) -> Result<(), BankError> {
    if Shared::same(from, to) {
        return Err(BankError::SameAccount);             // with2 would return Err(Aliased)
    }
    Shared::with2(from, to, |a, b| {
        let debited = a.balance.checked_sub(amount)
            .ok_or(BankError::Overdraft { id: a.id, balance: a.balance, amount })?;
        let credited = b.balance.checked_add(amount)
            .ok_or(BankError::Overflow { id: b.id, amount })?;
        a.balance = debited;                            // both checks passed: commit both
        b.balance = credited;
        Ok(())
    })?                                                 // LockError -> BankError via #[from]
}

/// The sum of all balances, read as one consistent snapshot.
pub fn total(accounts: &[Shared<Account>]) -> Result<u64, LockError> {
    Shared::with_all(accounts, |xs| xs.iter().map(|a| a.balance).sum())
}

/// Runs every `(from, to, amount)` transfer as its own task, all at once.
/// Indices must be in range for `accounts`. Refused transfers are expected
/// outcomes, not failures; returns how many transfers went through.
pub fn run_transfers(tasks: &Tasks, accounts: &[Shared<Account>], plan: &[(usize, usize, u64)])
    -> Result<usize, BankError>
{
    tasks.scope(|s| {
        let running: Vec<_> = plan.iter()
            .map(|&(from, to, amount)| s.spawn(move || Ok(transfer(&accounts[from], &accounts[to], amount))))
            .collect();
        let mut done = 0;
        for task in running {
            match task.join()? {
                Ok(()) => done += 1,
                Err(BankError::Overdraft { .. } | BankError::SameAccount) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(done)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accounts(balances: &[u64]) -> Vec<Shared<Account>> {
        balances.iter().enumerate()
            .map(|(id, &balance)| Shared::new(Account { id, balance }))
            .collect()
    }

    fn balance(account: &Shared<Account>) -> u64 {
        account.with(|a| a.balance).unwrap()
    }

    #[test]
    fn overdrafts_are_refused_and_change_nothing() {
        let xs = accounts(&[10, 5]);
        assert!(matches!(
            transfer(&xs[0], &xs[1], 11),
            Err(BankError::Overdraft { id: 0, balance: 10, amount: 11 })
        ));
        assert_eq!((balance(&xs[0]), balance(&xs[1])), (10, 5));
        transfer(&xs[0], &xs[1], 10).unwrap();
        assert_eq!((balance(&xs[0]), balance(&xs[1])), (0, 15));
        assert!(matches!(transfer(&xs[1], &xs[1].clone(), 1), Err(BankError::SameAccount)));   // a clone names the same cell
    }

    /// Random transfers, all running concurrently under a seed-chosen
    /// schedule, never create or destroy money.
    #[test]
    fn total_is_conserved(seed: u64, balances: Vec<u32>, plan: Vec<(usize, usize, u32)>)
        requires balances.len() >= 2
    {
        let sim = Root::sim(seed);
        let xs = accounts(&balances.iter().map(|&b| b as u64).collect::<Vec<u64>>());
        let n = xs.len();
        let plan: Vec<(usize, usize, u64)> = plan.iter()
            .map(|&(from, to, amount)| (from % n, to % n, amount as u64))
            .collect();
        let before = total(&xs).unwrap();
        let done = run_transfers(&sim.tasks, &xs, &plan).unwrap();
        assert!(done <= plan.len());
        assert_eq!(total(&xs).unwrap(), before);
    }
}
