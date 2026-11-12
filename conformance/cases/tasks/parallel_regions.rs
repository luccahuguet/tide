//@ rules: PAR-1, PAR-2, PAR-4, PAR-5, PAR-6, PAR-7, SINK-4
// build.rs executes this shared production surface on both runtimes, debug and release.
use std::par::join;
use std::task::{TaskError, Tasks};

fn exercise() {
    let mut left = String::from("left");
    let mut right = String::from("right");
    let lengths = join(
        || {
            left.push('!');
            println!("left");
            left.len()
        },
        || {
            right.push('!');
            println!("right");
            right.len()
        },
    );
    assert_eq!(lengths, (5, 6));
    let xs = [0i32, 1, 2, 3];
    let mapped = xs.par_map(|x| {
        println!("map {x}");
        x * 3
    });
    assert_eq!(mapped, vec![0, 3, 6, 9]);
    let mut changed = mapped;
    changed.par_for(|x| {
        println!("for {x}");
        *x += 1;
    });
    assert_eq!(changed, vec![1, 4, 7, 10]);
    changed.reverse();
    changed.par_sort();
    assert_eq!(changed, vec![1, 4, 7, 10]);
    let mut empty: Vec<String> = vec![];
    assert!(empty.par_map(|x| x.len()).is_empty());
    empty.par_for(|x| x.push('!'));
    empty.par_sort();
    let mut singleton = [String::from("one")];
    singleton.par_for(|x| x.push('!'));
    singleton.par_sort();
    assert_eq!(singleton.par_map(|x| x.len()), vec![4]);
    let nested = join(|| join(|| 3, || 4), || 5);
    assert_eq!(nested, ((3, 4), 5));
}

fn cancelled(tasks: &Tasks) {
    tasks
        .scope::<_, TaskError, _>(|scope| {
            scope.cancel();
            let mapped = [1, 2, 3].par_map(|x| x + 1);
            assert_eq!(mapped, vec![2, 3, 4]);
            assert!(tasks.checkpoint().is_err());
            Ok(())
        })
        .unwrap();
    let value = std::sync::Shared::new(0, tasks);
    assert_eq!(
        value.with(|n| {
            *n = join(|| 2, || 3).0;
            *n
        }),
        Ok(2)
    );
}

#[tide::main]
fn main(root: Root) {
    exercise();
    cancelled(&root.tasks);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::SimRoot;

    #[sim_test(seeds = 4)]
    fn simulated(sim: SimRoot) {
        exercise();
        cancelled(&sim.tasks);
        let records = sim.records();
        let messages: Vec<_> = records.iter().map(|r| r.message.as_str()).collect();
        assert_eq!(
            messages,
            vec![
                "left\n", "right\n", "map 0\n", "map 1\n", "map 2\n", "map 3\n", "for 0\n",
                "for 3\n", "for 6\n", "for 9\n"
            ]
        );
    }

    #[test]
    fn real() {
        exercise();
    }

    fn panics() {
        join(|| panic!("left winner"), || panic!("right loser"));
    }
    #[test]
    #[should_panic(expected = "left winner")]
    fn real_panic() {
        panics();
    }

    #[test]
    #[should_panic(expected = "left winner")]
    fn simulated_panic() {
        let _sim = Root::sim(9);
        panics();
    }
}
