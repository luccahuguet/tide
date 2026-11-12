//@ rules: CAP-1
// Ambient timed parks and deprecated sleeps read time without Clock.
pub fn ambient() {
    std::thread::park_timeout(std::time::Duration::ZERO); //~ E0433 FX0501
    std::thread::park_timeout_ms(0); //~ E0433 FX0501
    std::thread::sleep_ms(0); //~ E0433 FX0501
}
