use std::{collections::HashMap, process::ExitCode};
fn run(env: &std::env::Env, fs: &std::fs::Fs) -> Result<(String, u8), String> {
    let mut args = env.args().skip(1);
    let mut names: Vec<String> = args.by_ref().collect();
    let mut defaults = HashMap::new();
    if names.first().is_some_and(|s| s == "--defaults") {
        if names.len() < 3 {
            return Err("usage: envreport [--defaults FILE] NAME...".into());
        }
        let path = names[1].clone();
        let text = fs
            .read_to_string(&path)
            .map_err(|e| format!("envreport: {path}: {e}"))?;
        for (i, line) in text
            .lines()
            .enumerate()
            .filter(|(_, s)| !s.trim().is_empty() && !s.starts_with('#'))
        {
            let (k, v) = line
                .split_once('=')
                .filter(|(k, _)| !k.trim().is_empty())
                .ok_or_else(|| format!("envreport: {path}:{}: invalid default", i + 1))?;
            defaults.insert(k.trim().to_string(), v.to_string());
        }
        names.drain(..2);
    }
    if names.is_empty() {
        return Err("usage: envreport [--defaults FILE] NAME...".into());
    }
    let (mut out, mut code) = (String::new(), 0);
    for name in names {
        match env.var(&name).ok().or_else(|| defaults.get(&name).cloned()) {
            Some(v) => out.push_str(&format!("{name}={v}\n")),
            None => {
                out.push_str(&format!("{name} is unset\n"));
                code = 1;
            }
        }
    }
    Ok((out, code))
}
#[tide::main]
fn main(root: Root) -> ExitCode {
    match run(&root.env, &root.fs) {
        Ok((out, code)) => {
            print!("{out}");
            ExitCode::from(code)
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}
