//! `app`: loads `appdata/config.json`, fetches the configured status URL
//! from the one allowed host, and saves the body as `appdata/latest.json`.
//! Its authority is one directory, one host, and the monotonic clock.

use std::fs::{Fs, Mode};
use std::io;
use std::net::Net;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Clock, Duration};

const DATA_DIR: &str = "appdata";
const API_HOST: &str = "api.example.com:443";

/// Stands in for a configuration library.
mod config {
    use std::fs::Fs;
    use std::io;
    use std::json;

    #[derive(Debug, Clone, PartialEq, Decode)]
    #[codec(rename_all = "camelCase")]
    pub struct Config {
        pub status_url: String,
        #[codec(default)]
        pub timeout_secs: Option<u64>,
    }

    #[derive(Debug, Error)]
    pub enum ConfigError {
        #[error("cannot read config.json")]
        Read(#[source] io::Error),
        #[error("config.json is not valid")]
        Parse(#[from] json::Error),
    }

    pub fn load(dir: &Fs) -> Result<Config, ConfigError> {
        let bytes = dir.read("config.json").map_err(ConfigError::Read)?;
        Ok(json::decode(&bytes)?)
    }
}

/// Stands in for an HTTP client library.
mod client {
    use std::net::{Net, NetError, Url, UrlError};
    use std::time::Instant;

    #[derive(Debug, Error)]
    pub enum ClientError {
        #[error("invalid URL")]
        Url(#[from] UrlError),
        #[error("request failed")]
        Net(#[from] NetError),
    }

    pub fn get(net: &Net, url: &str, deadline: Instant) -> Result<Vec<u8>, ClientError> {
        let url = Url::parse(url)?;
        Ok(net.get(&url, deadline)?)
    }
}

/// Messages name this layer only; the cause is in `source()`.
#[derive(Debug, Error)]
enum AppError {
    #[error("cannot open data directory {path:?}")]
    DataDir { path: PathBuf, source: io::Error },
    #[error("cannot load configuration")]
    Config(#[from] config::ConfigError),
    #[error("cannot fetch status")]
    Client(#[from] client::ClientError),
    #[error("cannot save the response")]
    Save(#[source] io::Error),
}

/// Authority: `data` (one directory), `net` (one host), `clock`.
fn run(data: &Fs, net: &Net, clock: &Clock) -> Result<usize, AppError> {
    let cfg = config::load(&data.read_only())?;                     // ConfigError -> AppError
    let timeout = Duration::from_secs(cfg.timeout_secs.unwrap_or(10));
    let body = client::get(net, &cfg.status_url, clock.now() + timeout)?;   // ClientError -> AppError
    data.write_synced("latest.json", &body).map_err(AppError::Save)?;
    Ok(body.len())
}

fn main(root: Root) -> ExitCode {
    let data = match root.fs.dir(DATA_DIR, Mode::ReadWrite) {
        Ok(data) => data,
        Err(source) => return fail(&AppError::DataDir { path: PathBuf::from(DATA_DIR), source }),
    };
    let net = root.net.only([API_HOST]).no_bind();
    let clock = root.clock.monotonic_only();
    match run(&data, &net, &clock) {
        Ok(n) => {
            println!("saved {n} bytes to {DATA_DIR}/latest.json");
            ExitCode::SUCCESS
        }
        Err(e) => fail(&e),
    }
}

/// Prints `e` and its `source()` chain to stderr.
fn fail(e: &AppError) -> ExitCode {
    eprintln!("error: {e}");
    let mut cause = e.source();
    while let Some(c) = cause {
        eprintln!("  caused by: {c}");
        cause = c.source();
    }
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::NetError;
    use std::sim::{Reply, SimRoot};

    #[sim_test]
    fn saves_the_status_body(sim: SimRoot) {
        sim.world.file("/app/config.json", r#"{"statusUrl": "https://api.example.com/status", "timeoutSecs": 2}"#);
        sim.world.http(API_HOST, |_req| Reply::ok("{\"up\":true}"));
        let data = sim.fs.dir("/app", Mode::ReadWrite).unwrap();
        let net = sim.net.only([API_HOST]).no_bind();
        assert_eq!(run(&data, &net, &sim.clock).unwrap(), 11);
        assert_eq!(data.read_to_string("latest.json").unwrap(), "{\"up\":true}");
    }

    #[sim_test]
    fn other_hosts_are_denied(sim: SimRoot) {
        sim.world.file("/app/config.json", r#"{"statusUrl": "https://elsewhere.test/"}"#);
        sim.world.http("elsewhere.test:443", |_req| Reply::ok("leaked"));
        let data = sim.fs.dir("/app", Mode::ReadWrite).unwrap();
        let net = sim.net.only([API_HOST]).no_bind();
        let err = run(&data, &net, &sim.clock).unwrap_err();
        assert!(matches!(err, AppError::Client(client::ClientError::Net(NetError::Denied))), "{err:?}");
        assert!(!data.exists("latest.json"));
    }

    #[sim_test]
    fn a_missing_config_keeps_its_cause(sim: SimRoot) {
        sim.fs.create_dir_all("/app").unwrap();
        let data = sim.fs.dir("/app", Mode::ReadWrite).unwrap();
        let err = run(&data, &sim.net, &sim.clock).unwrap_err();
        assert_eq!(err.to_string(), "cannot load configuration");
        assert_eq!(err.source().unwrap().to_string(), "cannot read config.json");
        assert!(matches!(
            err,
            AppError::Config(config::ConfigError::Read(ref e)) if e.kind() == io::ErrorKind::NotFound
        ));
    }
}
