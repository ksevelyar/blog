mod error;
mod generator;
mod posts;
mod rss;
mod wasm_components;

use std::path::PathBuf;

use crate::error::{AppError, Result};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let source_path = std::env::var_os("SOURCE")
        .map(PathBuf::from)
        .ok_or_else(|| {
            AppError::Configuration("SOURCE environment variable is not set".to_owned())
        })?;
    let output_path = std::env::var_os("OUTPUT")
        .map(PathBuf::from)
        .ok_or_else(|| {
            AppError::Configuration("OUTPUT environment variable is not set".to_owned())
        })?;
    let public_site_url = std::env::var("SITE_URL").map_err(|_| {
        AppError::Configuration("SITE_URL environment variable is not set".to_owned())
    })?;

    generator::generate_site(&source_path, &output_path, &public_site_url)
}
