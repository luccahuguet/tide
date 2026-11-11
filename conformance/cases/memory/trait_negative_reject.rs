//@ rules: REF-0405
// Neither unstable spelling is admitted by Tide.
pub auto trait Shape {} //~ E0658 FX0405
pub struct Stone;
impl !Send for Stone {} //~ E0658 FX0405
