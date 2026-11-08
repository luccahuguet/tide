use std::process::ExitCode;
fn run(env: &std::env::Env, stdin: &std::io::Stdin) -> Result<String, (u8, String)> {
    let usage = || (2, "usage: colstats [--col N] [--sep C]".to_string());
    let (mut col, mut sep) = (1usize, ',');
    let mut args = env.args().skip(1);
    while let Some(a) = args.next() {
        let v = args.next().ok_or_else(usage)?;
        match a.as_str() {
            "--col" => {
                col = v.parse().map_err(|_| usage())?;
                if col == 0 {
                    return Err(usage());
                }
            }
            "--sep" => {
                let mut chars = v.chars();
                sep = chars.next().ok_or_else(usage)?;
                if chars.next().is_some() {
                    return Err(usage());
                }
            }
            _ => return Err(usage()),
        }
    }
    let mut text = String::new();
    stdin
        .read_to_string(&mut text)
        .map_err(|e| (1, e.to_string()))?;
    let (mut count, mut sum, mut min, mut max) = (0, 0i64, i64::MAX, i64::MIN);
    for (i, line) in text
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
    {
        let n: i64 = line
            .split(sep)
            .nth(col - 1)
            .and_then(|s| s.trim().parse().ok())
            .ok_or_else(|| {
                (
                    1,
                    format!("colstats: line {}: invalid integer column", i + 1),
                )
            })?;
        sum = sum
            .checked_add(n)
            .ok_or_else(|| (1, "colstats: overflow".into()))?;
        count += 1;
        min = min.min(n);
        max = max.max(n);
    }
    Ok(if count == 0 {
        "count=0\n".into()
    } else {
        format!("count={count} sum={sum} min={min} max={max}\n")
    })
}
#[tide::main]
fn main(root: Root) -> ExitCode {
    match run(&root.env, &root.stdin) {
        Ok(s) => {
            print!("{s}");
            ExitCode::SUCCESS
        }
        Err((code, e)) => {
            eprintln!("{e}");
            ExitCode::from(code)
        }
    }
}
