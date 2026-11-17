//! `cargo run -p tide-demo`: the demo. Exit 0 when every step shows what it should, 1 when one
//! does not, 2 on a setup error.
use std::process::ExitCode;

fn main() -> ExitCode {
    match tide_demo::Demo::default().run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            failure.report();
            ExitCode::from(failure.exit_code())
        }
    }
}
