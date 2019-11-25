use config::{ConfigError, Config, File, Environment};

#[derive(Debug, Deserialize)]
pub struct Database {
    pub rd_windows: String,
    pub rd_linux: String,
    pub rd_macos: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: Database,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name("config/default"))?;
        s.merge(File::with_name("config/local").required(false))?;
        s.merge(Environment::with_prefix("app"))?;
        s.try_into()
    }
}