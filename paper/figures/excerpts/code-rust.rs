pub fn load(path: &str) -> Result<Config, ConfigError> {
    parse(&std::fs::read_to_string(path)?)
}
