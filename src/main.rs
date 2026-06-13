mod app_state;
mod config;
mod db;
mod error;
mod index;
mod ingest;
mod util {
    pub mod fs;
    pub mod hash;
    pub mod time;
}
mod web;

use std::sync::Arc;

use actix_web::{App, HttpServer};

use crate::{app_state::AppState, config::AppConfig, db::assets::AssetStore};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = AppConfig::from_env_and_args()?;
    config.ensure_dirs()?;

    let db = db::open_sqlite(&config.database_url).await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    db::jobs::reset_stale_running_jobs(&db).await?;
    let auth_token = db::get_or_create_auth_token(&db).await?;

    let asset_store = Arc::new(AssetStore::new(config.covers_dir.clone())?);

    let (job_signal_tx, job_signal_rx) = tokio::sync::mpsc::channel(256);
    let (event_tx, _) = tokio::sync::broadcast::channel(1024);
    let (tantivy_tx, tantivy_rx) = tokio::sync::mpsc::channel(256);

    let state = actix_web::web::Data::new(AppState {
        db: db.clone(),
        job_signal_tx: job_signal_tx.clone(),
        event_tx: event_tx.clone(),
        asset_store: asset_store.clone(),
        tantivy_tx: tantivy_tx.clone(),
        auth_token,
        tantivy_dir: config.tantivy_dir.clone(),
    });

    tokio::spawn(index::tantivy_index::run_worker(
        db.clone(),
        tantivy_rx,
        config.tantivy_dir.clone(),
    ));
    tokio::spawn(ingest::worker::run_workers(
        db.clone(),
        job_signal_rx,
        job_signal_tx.clone(),
        event_tx.clone(),
        tantivy_tx.clone(),
        asset_store.clone(),
    ));
    tokio::spawn(ingest::watcher::run_manager(
        db.clone(),
        job_signal_tx.clone(),
        event_tx.clone(),
        tantivy_tx.clone(),
    ));
    tokio::spawn(ingest::scheduler::run_periodic_scans(
        db.clone(),
        job_signal_tx.clone(),
    ));

    tracing::info!(
        port = config.port,
        data_dir = %config.data_dir.display(),
        "starting papercache"
    );
    HttpServer::new(move || App::new().app_data(state.clone()).configure(web::configure))
        .bind(("127.0.0.1", config.port))?
        .run()
        .await?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt().init();
}
