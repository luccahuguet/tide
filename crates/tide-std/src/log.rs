//! Write-only log configuration (03 SINK-2). Text and JSON line shapes are provisional.
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
impl Level {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
    pub(crate) fn from_name(name: &str) -> Self {
        match name {
            "error" => Self::Error,
            "warn" => Self::Warn,
            "info" => Self::Info,
            "debug" => Self::Debug,
            "trace" => Self::Trace,
            _ => unreachable!("unsupported log level"),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Text,
    Json,
}

/// Process sink configuration, minted only as `Root::log`. Setters never expose sink state.
pub struct LogConfig(pub(crate) ());
struct Config {
    level: Level,
    format: Format,
    file: Option<crate::fs::File>,
}
static CONFIG: Mutex<Config> = Mutex::new(Config {
    level: Level::Info,
    format: Format::Text,
    file: None,
});
impl LogConfig {
    pub fn level(&self, level: Level) {
        CONFIG.lock().unwrap().level = level;
    }
    pub fn format(&self, format: Format) {
        CONFIG.lock().unwrap().format = format;
    }
    /// Redirects the log sink to an already opened file. Out and err keep their destinations.
    pub fn to_file(&self, file: crate::fs::File) {
        CONFIG.lock().unwrap().file = Some(file);
    }
}

// Formatting of user values has already finished, outside both locks and the level filter.
pub(crate) fn emit(level: Level, message: &[u8]) {
    use std::io::Write;
    let config = CONFIG.lock().unwrap();
    if level > config.level {
        return;
    }
    let message = std::str::from_utf8(message).expect("formatted log is UTF-8");
    let line = match config.format {
        Format::Text => format!("{}: {message}\n", level.name()),
        Format::Json => format!(
            "{{\"level\":\"{}\",\"message\":{}}}\n",
            level.name(),
            String::from_utf8(crate::json::encode(message)).unwrap()
        ),
    };
    if let Some(file) = &config.file {
        file.write_sink(line.as_bytes());
    } else {
        let mut stderr = std::io::stderr().lock();
        let _ = stderr
            .write_all(line.as_bytes())
            .and_then(|_| stderr.flush());
    }
}
