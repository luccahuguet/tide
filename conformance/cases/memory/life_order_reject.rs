//@ rules: LIFE-1
// A lifetime parameter after a type parameter is a parse error.
pub fn get<T, 'a>(x: &'a T) -> &'a T { //~ error
    x
}
