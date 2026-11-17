//@ expect: rejected TD0501 E0425
// config.rs's loader written as a Rust habit: `load` reads the file with Rust's ambient
// `std::fs::read_to_string` and takes no `Fs`. Tide's std has no such function, so rustc also
// reports `E0425` at the call; admission names it `TD0501`.
#[derive(Debug, PartialEq)]
pub struct Config {
    pub port: u16,
    pub workers: u16,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("cannot read the config")]
    Io(#[from] std::io::Error),
    #[error("bad line: {0}")]
    BadLine(String),
}

pub fn load(path: &str) -> Result<Config, ConfigError> {
    parse(&std::fs::read_to_string(path)?)
}

#[pure]
pub fn parse(text: &str) -> Result<Config, ConfigError> {
    let mut config = Config { port: 80, workers: 1 };
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let bad = || ConfigError::BadLine(line.to_string());
        let (key, value) = line.split_once('=').ok_or_else(bad)?;
        let value: u16 = value.trim().parse().map_err(|_| bad())?;
        match key.trim() {
            "port" => config.port = value,
            "workers" => config.workers = value,
            _ => return Err(bad()),
        }
    }
    Ok(config)
}
