use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;
use tracing_subscriber::prelude::*;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Crawl {},
}

// config
#[derive(Debug, serde::Deserialize)]
struct Config {
    //
}

fn load_toml_config<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> T {
    let path = path.as_ref();
    tracing::info!("Loading config from: {}", path.display());

    let raw_config = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: '{}'", path.display()))
        .unwrap();
    tracing::debug!("Raw config: {}", raw_config);

    toml::from_str(&raw_config)
        .with_context(|| format!("Failed to parse config: '{}'", path.display()))
        .unwrap()
}

fn main() -> Result<()> {
    // let trace max level configurable (default to info)
    let trace_max_level = std::env::var("TRACE_MAX_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .parse()
        .unwrap_or(tracing::Level::INFO);
    tracing_subscriber::fmt()
        .with_max_level(trace_max_level)
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive("neos=info".parse().unwrap())
                .from_env_lossy(),
        )
        .without_time()
        .with_target(false)
        .with_writer(std::io::stderr)
        .finish()
        .init();

    // Load the configuration file.
    let config_path = std::env::var("APP_CONFIG").unwrap_or_else(|_| "../app.toml".to_string());
    let config: Config = load_toml_config(&config_path);
    tracing::info!("Loaded config: {:?}", config);

    // Parse the command line arguments.
    let args = Args::parse();
    match args.command {
        Commands::Crawl {} => {
            //TODO
            tracing::info!("Crawling...");
        }
    }

    Ok(())
}
