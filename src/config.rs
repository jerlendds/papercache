use std::{path::PathBuf, time::Duration};

use crate::util::fs::default_data_dir;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub data_dir: PathBuf,
    pub auth_token_path: PathBuf,
    pub database_url: String,
    pub tantivy_dir: PathBuf,
    pub covers_dir: PathBuf,
    pub page_cache_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub port: u16,
    pub show_token: bool,
}

impl AppConfig {
    pub fn from_env_and_args() -> anyhow::Result<Self> {
        let mut data_dir = default_data_dir();
        let mut port = std::env::var("PAPERCACHE_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(3141);
        let mut show_token = false;

        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--data-dir" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--data-dir requires a value"))?;
                    data_dir = PathBuf::from(value);
                }
                "--port" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--port requires a value"))?;
                    port = value.parse()?;
                }
                "--show-token" => {
                    show_token = true;
                }
                "--help" | "-h" => {
                    println!("papercache [--data-dir PATH] [--port PORT] [--show-token]");
                    std::process::exit(0);
                }
                other => anyhow::bail!("unknown argument: {other}"),
            }
        }

        let database_url = format!("sqlite://{}", data_dir.join("app.sqlite").display());
        Ok(Self {
            auth_token_path: data_dir.join("token.json"),
            tantivy_dir: data_dir.join("tantivy"),
            covers_dir: data_dir.join("covers"),
            page_cache_dir: data_dir.join("page-cache"),
            logs_dir: data_dir.join("logs"),
            data_dir,
            database_url,
            port,
            show_token,
        })
    }

    pub fn ensure_dirs(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.tantivy_dir)?;
        std::fs::create_dir_all(&self.covers_dir)?;
        std::fs::create_dir_all(&self.page_cache_dir)?;
        std::fs::create_dir_all(&self.logs_dir)?;
        Ok(())
    }
}

pub const WATCH_DEBOUNCE: Duration = Duration::from_secs(2);
