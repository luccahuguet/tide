// A Tide program as a bin crate, entry spelled `#[tide::main]`.
use std::sync::{Shared, channel};

#[derive(Debug)]
#[allow(dead_code)] // read only through Debug
enum AppError {
    Asked,
    Io(std::io::Error),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

#[tide::main]
fn main(root: Root) -> Result<(), AppError> {
    if root.fs.read("FAIL").is_ok() {
        return Err(AppError::Asked);
    }
    let counter = Shared::new(0u32, &root.tasks);
    let (tx, rx) = channel(&root.tasks);
    root.tasks.scope(|s| {
        for i in 1..=3u32 {
            let (c, tx) = (counter.clone(), tx.clone());
            s.spawn(move || {
                c.with(|n| *n += i).unwrap();
                tx.send(i).unwrap();
            });
        }
    });
    drop(tx);
    let sent: u32 = rx.iter().sum();
    let t = root.clock.now();
    root.fs.write("attr.txt", format!("{sent}"))?;
    println!("attr entry: Shared total {:?}, channel sum {sent}, wrote attr.txt at {t:?}", counter.with(|n| *n));
    Ok(())
}
