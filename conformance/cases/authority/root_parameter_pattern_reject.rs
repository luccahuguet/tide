//@ rules: ROOT-1, ROOT-2
// Destructuring a helper parameter does not allow a Root-typed helper signature.
pub fn fields(Root { fs, .. }: Root) { let _ = fs; } //~ TD0503
pub fn borrowed(Root { fs, .. }: &Root) { let _ = fs; } //~ TD0503
