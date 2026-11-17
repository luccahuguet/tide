//@ expect: tests pass
// A config loader. `load` may read files only through the `Fs` it is handed; `parse` is
// declared authority-free with `#[pure]`. The tests hand `load` a simulated file system.
use std::fs::Fs;

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

pub fn load(fs: &Fs, path: &str) -> Result<Config, ConfigError> {
    parse(&fs.read_to_string(path)?)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_from_a_simulated_file_system() {
        let sim = Root::sim(1);
        sim.world.file("/etc/app.conf", "port = 8080\nworkers = 4\n");
        let config = load(&sim.fs, "/etc/app.conf").unwrap();
        assert_eq!(config, Config { port: 8080, workers: 4 });
    }

    #[test]
    fn a_missing_file_is_an_io_error() {
        let sim = Root::sim(1);
        assert!(matches!(load(&sim.fs, "/etc/app.conf"), Err(ConfigError::Io(_))));
    }

    #[test]
    fn an_unknown_key_is_a_bad_line() {
        assert!(matches!(parse("colour = 3"), Err(ConfigError::BadLine(_))));
    }
}
