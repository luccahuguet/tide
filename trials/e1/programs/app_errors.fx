// src/main.fx of package `trials/statusd`
//
// Fetches a status document from one host and stores it in one directory.
// `main` attenuates authority before anything else runs: the rest of the
// program can reach only DATA_DIR and API_HOST.

use std::fs::{Fs, Mode};
use std::io;
use std::net::{Net, NetError, Url};
use std::process::ExitCode;
use std::time::{Clock, Duration};

const DATA_DIR: &str = "/var/lib/statusd";
const API_HOST: &str = "status.example.com:443";
const STATUS_URL: &str = "https://status.example.com/v1/status";
const REQUEST_BUDGET: Duration = Duration::from_secs(10);

/// The application's error. The two library errors convert in through `?`.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("file system operation failed")]
    Io(#[from] io::Error),
    #[error("status request failed")]
    Net(#[from] NetError),
    #[error("invalid status URL {0:?}")]
    BadUrl(&'static str),
    #[error("the status document is empty")]
    EmptyStatus,
}

/// Authority: `data` (one directory) and `net` (one host), both attenuated by
/// `main`, and the clock for the request deadline.
pub fn sync_status(data: &Fs, net: &Net, clock: &Clock) -> Result<usize, AppError> {
    let url = Url::parse(STATUS_URL).map_err(|_| AppError::BadUrl(STATUS_URL))?;
    let body = net.get(&url, clock.now() + REQUEST_BUDGET)?; // NetError -> AppError::Net
    if body.is_empty() {
        return Err(AppError::EmptyStatus);
    }
    // Write a temporary file and rename it, so readers never see a partial document.
    data.write("status.json.tmp", &body)?; // io::Error -> AppError::Io
    data.rename("status.json.tmp", "status.json")?;
    Ok(body.len())
}

/// Logs the error and its whole `source()` chain.
fn report(e: &AppError) -> ExitCode {
    error!("{e}");
    let mut cause = e.source();
    while let Some(c) = cause {
        error!("  caused by: {c}");
        cause = c.source();
    }
    ExitCode::FAILURE
}

fn main(root: Root) -> ExitCode {
    let net = root.net.only([API_HOST]);
    let data = match root.fs.dir(DATA_DIR, Mode::ReadWrite) {
        Ok(fs) => fs,
        Err(e) => return report(&AppError::from(e)),
    };
    match sync_status(&data, &net, &root.clock) {
        Ok(n) => {
            info!("stored {n} bytes of status in {DATA_DIR}");
            ExitCode::SUCCESS
        }
        Err(e) => report(&e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::Reply;

    const BODY: &str = r#"{"up":true}"#;

    #[test]
    fn stores_the_status_in_the_data_dir() {
        let sim = Root::sim(3);
        sim.world.http(API_HOST, |_req| Reply::ok(BODY));
        sim.fs.create_dir_all(DATA_DIR).unwrap();
        let data = sim.fs.dir(DATA_DIR, Mode::ReadWrite).unwrap();
        let net = sim.net.only([API_HOST]);

        assert_eq!(sync_status(&data, &net, &sim.clock).unwrap(), BODY.len());
        assert_eq!(data.read_to_string("status.json").unwrap(), BODY);
        assert!(!data.exists("status.json.tmp").unwrap());
    }

    #[test]
    fn server_errors_arrive_as_net_errors() {
        let sim = Root::sim(4);
        sim.world.http(API_HOST, |_req| Reply::status(503));
        sim.fs.create_dir_all(DATA_DIR).unwrap();
        let data = sim.fs.dir(DATA_DIR, Mode::ReadWrite).unwrap();
        let net = sim.net.only([API_HOST]);

        let err = sync_status(&data, &net, &sim.clock).unwrap_err();
        assert!(matches!(err, AppError::Net(_)), "{err:?}");
        assert!(err.source().is_some());
    }

    #[test]
    fn other_hosts_are_out_of_reach() {
        let sim = Root::sim(5);
        sim.world.http("elsewhere.example.net:443", |_req| Reply::ok("secret"));
        let net = sim.net.only([API_HOST]);
        let url = Url::parse("https://elsewhere.example.net/").unwrap();
        assert!(net.get(&url, sim.clock.now() + REQUEST_BUDGET).is_err());
    }

    #[test]
    fn the_data_dir_cannot_be_escaped() {
        let sim = Root::sim(6);
        sim.fs.create_dir_all(DATA_DIR).unwrap();
        sim.world.file("/etc/passwd", "root:x:0:0");
        let data = sim.fs.dir(DATA_DIR, Mode::ReadWrite).unwrap();
        assert!(data.read_to_string("../../../etc/passwd").is_err());
    }

    #[test]
    fn library_errors_convert_with_question_mark() {
        fn io_step() -> Result<(), AppError> {
            Err(io::Error::from(io::ErrorKind::PermissionDenied))?
        }
        fn net_step() -> Result<(), AppError> {
            Err(NetError::Timeout)?
        }
        assert!(matches!(io_step(), Err(AppError::Io(e)) if e.kind() == io::ErrorKind::PermissionDenied));
        assert!(matches!(net_step(), Err(AppError::Net(NetError::Timeout))));
    }
}
