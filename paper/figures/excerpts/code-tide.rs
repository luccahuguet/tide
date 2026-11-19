pub fn load(fs: &Fs, path: &str) -> Result<Config, ConfigError> {
    parse(&fs.read_to_string(path)?)
}
