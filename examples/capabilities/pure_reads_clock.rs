//@ expect: rejected TD0511
// config.rs's loader with `parse` handed a `Clock` to stamp when the config was read. Passing
// the capability is legal, but `parse` is declared `#[pure]`.
use std::fs::Fs;
use std::time::{Clock, Instant};

#[derive(Debug, PartialEq)]
pub struct Config {
    pub port: u16,
    pub workers: u16,
    pub loaded_at: Instant,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("cannot read the config")]
    Io(#[from] std::io::Error),
    #[error("bad line: {0}")]
    BadLine(String),
}

pub fn load(fs: &Fs, clock: &Clock, path: &str) -> Result<Config, ConfigError> {
    parse(&fs.read_to_string(path)?, clock)
}

#[pure]
pub fn parse(text: &str, clock: &Clock) -> Result<Config, ConfigError> {
    let mut config = Config { port: 80, workers: 1, loaded_at: clock.now() };
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
