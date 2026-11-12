//@ rules: REF-0605
// Every user-written select! has this code, regardless of its provider.
pub fn choose() { select! {}; } //~ error FX0605
pub fn runtime() { tokio::select! {}; } //~ E0433 FX0605
