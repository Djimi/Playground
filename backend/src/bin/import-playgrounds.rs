use std::{env, path::Path};

use anyhow::{Context, Result};
use sofia_playgrounds_backend::{
    db,
    importer::{DEFAULT_NEIGHBORHOODS_PATH, DEFAULT_OVERPASS_URL, run_import},
};

#[tokio::main]
async fn main() -> Result<()> {
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;
    let overpass_url = env::var("OVERPASS_URL").unwrap_or_else(|_| DEFAULT_OVERPASS_URL.to_owned());
    let neighborhoods_path =
        env::var("NEIGHBORHOODS_PATH").unwrap_or_else(|_| DEFAULT_NEIGHBORHOODS_PATH.to_owned());
    let pool = db::connect_and_migrate(&database_url).await?;
    let counts = run_import(&pool, &overpass_url, Path::new(&neighborhoods_path)).await?;

    println!(
        "Imported {} playgrounds, {} neighborhoods, and {} memberships",
        counts.playgrounds, counts.neighborhoods, counts.memberships
    );
    Ok(())
}
