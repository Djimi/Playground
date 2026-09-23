use std::{env, path::Path};

use anyhow::{Context, Result};
use sofia_playgrounds_backend::{
    commons::COMMONS_API_URL,
    db,
    enrichment::SOFIAPLAN_URL,
    importer::{DEFAULT_NEIGHBORHOODS_PATH, DEFAULT_OVERPASS_URL, ImportEndpoints, run_import},
};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;
    let endpoints = ImportEndpoints {
        overpass_url: env::var("OVERPASS_URL").unwrap_or_else(|_| DEFAULT_OVERPASS_URL.to_owned()),
        sofiaplan_url: env::var("SOFIAPLAN_URL").unwrap_or_else(|_| SOFIAPLAN_URL.to_owned()),
        commons_api_url: env::var("COMMONS_API_URL").unwrap_or_else(|_| COMMONS_API_URL.to_owned()),
    };
    let neighborhoods_path =
        env::var("NEIGHBORHOODS_PATH").unwrap_or_else(|_| DEFAULT_NEIGHBORHOODS_PATH.to_owned());
    let pool = db::connect_and_migrate(&database_url).await?;
    let counts = run_import(&pool, &endpoints, Path::new(&neighborhoods_path)).await?;

    println!(
        "Imported: OSM source records={}, SofiaPlan source records={}, canonical playgrounds={}, clear matches={}, ambiguous records={}, excluded source records={}, accepted photos={}, rejected photos={}, neighborhoods={}, memberships={}",
        counts.osm_source_records,
        counts.sofia_source_records,
        counts.canonical_playgrounds,
        counts.clear_matches,
        counts.ambiguous_records,
        counts.excluded_source_records,
        counts.accepted_photos,
        counts.rejected_photos,
        counts.neighborhoods,
        counts.memberships
    );
    Ok(())
}
