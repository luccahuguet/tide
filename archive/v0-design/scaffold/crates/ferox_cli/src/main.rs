#![forbid(unsafe_code)]

use ferox_driver::{bootstrap_smoke_pipeline, bootstrap_status};

fn main() {
    let mut args = std::env::args().skip(1);

    match args.next().as_deref() {
        Some("bootstrap-status") => {
            let status = bootstrap_status();
            println!("{}", status.render());
            println!("workspace crates: {}", status.workspace_crates.join(", "));
            println!("trace env var: {}", status.trace_env_var);
        }
        Some("fmt-smoke") => {
            let input = args.next().unwrap_or_else(|| "fn main() -> Int { 1 }".to_string());
            match bootstrap_smoke_pipeline(&input) {
                Ok(formatted) => print!("{formatted}"),
                Err(message) => {
                    eprintln!("bootstrap format smoke failed: {message}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            println!("ferox bootstrap CLI");
            println!("usage:");
            println!("  ferox bootstrap-status");
            println!("  ferox fmt-smoke [source]");
        }
    }
}

