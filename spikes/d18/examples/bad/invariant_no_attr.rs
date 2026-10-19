// invariant! without a function-level contract attribute: rejected, never silently unchecked.
pub fn f() {
    for _ in 0..0 {
        invariant!(false);
    }
}
