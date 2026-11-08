use std::process::ExitCode;
fn run(env: &std::env::Env, fs: &std::fs::Fs) -> u8 {
    let mut args: Vec<String> = env.args().skip(1).collect();
    let insensitive = args.first().is_some_and(|s| s == "-i");
    if insensitive {
        args.remove(0);
    }
    if args.len() < 2 || args.iter().any(|s| s.starts_with('-')) {
        eprintln!("usage: findtext [-i] PATTERN FILE...");
        return 2;
    }
    let pattern = if insensitive {
        args[0].to_ascii_lowercase()
    } else {
        args[0].clone()
    };
    let (mut found, mut error) = (false, false);
    for file in &args[1..] {
        match fs.read_to_string(file) {
            Ok(text) => {
                for (i, line) in text.lines().enumerate() {
                    if if insensitive {
                        line.to_ascii_lowercase().contains(&pattern)
                    } else {
                        line.contains(&pattern)
                    } {
                        println!("{file}:{}:{line}", i + 1);
                        found = true;
                    }
                }
            }
            Err(e) => {
                eprintln!("findtext: {file}: {e}");
                error = true;
            }
        }
    }
    if error {
        2
    } else if found {
        0
    } else {
        1
    }
}
#[tide::main]
fn main(root: Root) -> ExitCode {
    ExitCode::from(run(&root.env, &root.fs))
}
