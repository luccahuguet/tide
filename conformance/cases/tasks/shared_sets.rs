//@ rules: LOCK-1, LOCK-2, LOCK-3, LOCK-4, LOCK-5, LOCK-6, SEND-3
// Production owns these operations; build.rs executes this case on threads and under Root::sim.
use std::sync::{LockError, Shared};
use std::task::{TaskError, Tasks};

fn exercise(tasks: &Tasks) {
    let a = Shared::new(100i32, tasks);
    let b = Shared::new(100i32, tasks);
    let c = Shared::new(100i32, tasks);
    assert!(Shared::same(&a, &a.clone()));
    assert!(!Shared::same(&a, &b));
    let mut ran = false;
    assert_eq!(
        Shared::with2(&a, &a.clone(), |_, _| ran = true),
        Err(LockError::Aliased)
    );
    assert_eq!(
        Shared::with_all(&[a.clone(), b.clone(), a.clone()], |_| ran = true),
        Err(LockError::Aliased)
    );
    assert!(!ran);
    assert_eq!(Shared::<i32>::with_all(&[], |xs| xs.len()), Ok(0));
    Shared::with_all(&[c.clone(), a.clone(), b.clone()], |xs| {
        *xs[0] = 103;
        *xs[1] = 98;
        *xs[2] = 99;
    })
    .unwrap();
    assert_eq!(a.with(|v| *v), Ok(98));
    assert_eq!(b.with(|v| *v), Ok(99));
    assert_eq!(c.with(|v| *v), Ok(103));
    a.reset(100);
    b.reset(100);
    c.reset(100);
    tasks
        .scope::<_, TaskError, _>(|scope| {
            scope.cancel(); // Acquisition and reset still work after cancellation.
            for i in 0..6 {
                let (a, b, c) = (a.clone(), b.clone(), c.clone());
                scope.spawn(move || {
                    for _ in 0..30 {
                        if i % 2 == 0 {
                            Shared::with2(&a, &b, |x, y| {
                                *x -= 1;
                                *y += 1;
                            })
                            .unwrap();
                            Shared::with_all(&[c.clone(), b.clone(), a.clone()], |xs| {
                                assert_eq!(*xs[0] + *xs[1] + *xs[2], 300);
                                *xs[0] -= 1;
                                *xs[2] += 1;
                            })
                            .unwrap();
                        } else {
                            Shared::with2(&b, &a, |x, y| {
                                *x -= 1;
                                *y += 1;
                            })
                            .unwrap();
                            Shared::with_all(&[a.clone(), c.clone()], |xs| {
                                *xs[0] -= 1;
                                *xs[1] += 1;
                            })
                            .unwrap();
                        }
                    }
                    Ok(())
                });
            }
            Ok(())
        })
        .unwrap();
    assert_eq!(
        Shared::with_all(&[a.clone(), b, c], |xs| *xs[0] + *xs[1] + *xs[2]),
        Ok(300)
    );
    let text = Shared::new(String::from("pair"), tasks);
    assert_eq!(
        Shared::with2(&text, &a, |s, n| {
            s.push('!');
            *n = 7;
            s.len()
        }),
        Ok(5)
    );
    assert_eq!(a.with(|n| *n), Ok(7));
}

#[tide::main]
fn main(root: Root) {
    exercise(&root.tasks);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::SimRoot;

    #[test]
    fn plain_simulation() {
        let sim = Root::sim(17);
        exercise(&sim.tasks);
    }

    #[sim_test(seeds = 8)]
    fn scheduled_sets(sim: SimRoot) {
        exercise(&sim.tasks);
    }

    struct Recovery {
        cells: [Shared<i32>; 3],
        poisoned: usize,
    }

    impl Drop for Recovery {
        fn drop(&mut self) {
            for i in 0..self.poisoned {
                let cell = &self.cells[i];
                let mut ran = false;
                assert_eq!(cell.with(|_| ran = true), Err(LockError::Poisoned));
                assert_eq!(
                    Shared::with2(cell, cell, |_, _| ran = true),
                    Err(LockError::Aliased)
                );
                assert_eq!(
                    Shared::with_all(&[cell.clone(), cell.clone()], |_| ran = true),
                    Err(LockError::Aliased)
                );
                assert!(!ran);
            }
            let mut ran = false;
            assert_eq!(
                Shared::with_all(&[self.cells[2].clone(), self.cells[0].clone()], |_| ran =
                    true),
                Err(LockError::Poisoned)
            );
            assert!(!ran);
            if self.poisoned == 2 {
                assert_eq!(self.cells[2].with(|n| *n), Ok(0));
            }
            for cell in &self.cells {
                cell.reset(9);
            }
            assert_eq!(
                Shared::with_all(&self.cells, |xs| *xs[0] + *xs[1] + *xs[2]),
                Ok(27)
            );
        }
    }

    #[test]
    #[should_panic(expected = "pair panic")]
    fn pair_poison_and_reset() {
        let sim = Root::sim(5);
        let recovery = Recovery {
            cells: [
                Shared::new(0, &sim.tasks),
                Shared::new(0, &sim.tasks),
                Shared::new(0, &sim.tasks),
            ],
            poisoned: 2,
        };
        Shared::with2(&recovery.cells[1], &recovery.cells[0], |a, b| {
            *a = 1;
            *b = 2;
            panic!("pair panic");
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "set panic")]
    fn set_poison_and_reset() {
        let sim = Root::sim(7);
        let recovery = Recovery {
            cells: [
                Shared::new(0, &sim.tasks),
                Shared::new(0, &sim.tasks),
                Shared::new(0, &sim.tasks),
            ],
            poisoned: 3,
        };
        Shared::with_all(&recovery.cells, |_| panic!("set panic")).unwrap();
    }
}
