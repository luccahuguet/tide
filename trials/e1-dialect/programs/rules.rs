// Task 2: rules -- src/lib.rs
//
// A list of row predicates. `'a` bounds everything a predicate may borrow:
// owned data (`at_least`), a caller's slice (`one_of`), or a caller's locals
// (`custom`).

/// Predicates over `&T`; `matches` is their conjunction.
pub struct Rules<'a, T> {
    preds: Vec<Box<dyn Fn(&T) -> bool + 'a>>,
}

impl<'a, T> Rules<'a, T> {
    pub fn new() -> Self {
        Rules { preds: Vec::new() }
    }

    /// True when every predicate accepts `x`; an empty list accepts everything.
    pub fn matches(&self, x: &T) -> bool {
        self.preds.iter().all(|p| p(x))
    }

    /// How many elements of `xs` match.
    pub fn count_matching(&self, xs: &[T]) -> usize {
        xs.iter().filter(|x| self.matches(x)).count()
    }

    /// Adds a predicate that owns `min`: accepts `x >= min`.
    pub fn at_least(&mut self, min: T)
    where
        T: PartialOrd + 'a,
    {
        self.preds.push(Box::new(move |x: &T| *x >= min));
    }

    /// Adds a predicate that borrows `allowed`: accepts members of it.
    pub fn one_of(&mut self, allowed: &'a [T])
    where
        T: PartialEq + 'a,
    {
        self.preds.push(Box::new(move |x: &T| allowed.contains(x)));
    }

    /// Adds any predicate, including one that borrows the caller's locals.
    pub fn custom<F>(&mut self, f: F)
    where
        F: Fn(&T) -> bool + 'a,
    {
        self.preds.push(Box::new(f));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn empty_rules_accept_everything() {
        let r: Rules<'_, i32> = Rules::new();
        assert!(r.matches(&-7));
        assert_eq!(r.count_matching(&[1, 2, 3]), 3);
    }

    #[test]
    fn integers_with_at_least_and_one_of() {
        let mut r = Rules::new();
        r.at_least(10);
        r.one_of(&[5, 10, 20, 30]);
        assert!(!r.matches(&5)); // allowed, but below the minimum
        assert!(r.matches(&10));
        assert!(!r.matches(&15)); // above the minimum, but not allowed
        assert_eq!(r.count_matching(&[5, 10, 15, 20, 30, 40]), 3);
    }

    #[test]
    fn str_rows_with_custom_closure_borrowing_a_local_set() {
        let banned: HashSet<&str> = ["spam", "junk"].into_iter().collect();
        let mut r: Rules<'_, &str> = Rules::new();
        r.custom(|row| !banned.contains(row));
        r.custom(|row| !row.is_empty());
        let rows = ["ham", "spam", "", "eggs", "junk"];
        assert_eq!(r.count_matching(&rows), 2);
        assert!(r.matches(&"eggs"));
        assert!(!r.matches(&"junk"));
    }

    #[test]
    fn one_of_borrows_a_vec_from_the_same_block() {
        let allowed: Vec<i64> = vec![3, 6, 9, 12];
        let mut r = Rules::new();
        r.one_of(&allowed);
        r.at_least(5);
        assert_eq!(r.count_matching(&[1, 3, 6, 7, 9, 12]), 3);
        assert!(!r.matches(&3));
    }
}
