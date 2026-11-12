//@ rules: ROOT-1, SINK-1, SINK-2, SINK-3, SINK-4, TASK-6
use std::io::{self, Write};
use std::log::{Format, Level};
struct Shown<'a>(&'a std::cell::Cell<u32>);
impl std::fmt::Display for Shown<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.set(self.0.get() + 1);
        write!(f, "formatted")
    }
}
#[tide::main]
fn main(root: Root) -> io::Result<()> {
    let mut seen = 0;
    info!("default {}", {
        seen += 1;
        seen
    });
    debug!("disabled {}", {
        seen += 1;
        seen
    });
    assert_eq!(seen, 2);
    let formatted = std::cell::Cell::new(0);
    debug!("{}", Shown(&formatted));
    assert_eq!(formatted.get(), 1);
    root.log.level(Level::Trace);
    error!("error");
    warn!("warn");
    debug!("debug");
    trace!("trace");
    root.log.format(Format::Json);
    info!("quote=\" newline=\n café");
    root.log
        .to_file(root.fs.create(root.env.var("LOG_FILE").unwrap())?);
    info!("file");
    {
        let mut writer = io::BufWriter::new(io::stdout());
        writer.write_all(b"secret-payload")?;
    }
    println!("done");
    Ok(())
}
