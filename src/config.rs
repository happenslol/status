use gpui::{App, Global};
use serde::Deserialize;
use tracing::{info, warn};

#[derive(Debug, Default, Deserialize)]
pub struct Config {
  pub time: TimeConfig,
}

#[derive(Debug, Default, Deserialize)]
pub struct TimeConfig {
  pub display: Option<String>,
  pub opacity: Option<f32>,
}

impl Global for Config {}

impl Config {
  pub fn load_sync() -> Self {
    let Some(path) = dirs::config_dir().map(|p| p.join("status/config.toml")) else {
      return Self::default();
    };

    if !path.exists() {
      info!(?path, "No config file found, using default");
      return Self::default();
    }

    let contents = match std::fs::read_to_string(&path) {
      Ok(contents) => contents,
      Err(err) => {
        warn!(?path, ?err, "Failed to read config file, using default");
        return Self::default();
      }
    };

    let config: Self = match toml::from_str(&contents) {
      Ok(config) => config,
      Err(err) => {
        warn!(?path, ?err, "Failed to parse config file, using default");
        return Self::default();
      }
    };

    config
  }
}

pub trait ConfigExt {
  fn config(&self) -> &Config;
}

impl ConfigExt for App {
  fn config(&self) -> &Config {
    self.global::<Config>()
  }
}
