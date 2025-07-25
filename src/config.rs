use std::{env, path::PathBuf};

use anyhow::Result;
use gpui::{App, Global};
use serde::Deserialize;
use tracing::{info, warn};

pub fn init(cx: &mut App) -> Result<()> {
  cx.spawn(async move |cx| {
    let Ok(config) = load_config()
      .await
      .inspect_err(|err| warn!(?err, "Failed to load config"))
    else {
      return;
    };

    let config = config.unwrap_or_default();
  })
  .detach();

  Ok(())
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Config {
  pub time_display: Option<String>,
}

pub struct ConfigStore {
  config: Config,
}

impl Global for ConfigStore {}

fn get_config_location() -> Option<PathBuf> {
  let home = env::var("HOME").ok()?;
  let config_path = PathBuf::from(home).join(".config/status/config.toml");
  Some(config_path)
}

async fn load_config() -> Result<Option<Config>> {
  let Some(config_path) = get_config_location() else {
    info!("No config file found");
    return Ok(None);
  };

  let meta = smol::fs::metadata(&config_path).await?;
  if !meta.is_file() {
    warn!("Config file is not a file");
    return Ok(None);
  }

  let config = smol::fs::read_to_string(&config_path).await?;
  Ok(Some(toml::from_str(&config)?))
}
