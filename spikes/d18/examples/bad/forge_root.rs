pub fn forge() -> Root {
    Root { fs: std::fs::Fs { base: None }, clock: std::time::Clock(()), tasks: std::task::Tasks(()) }
}
