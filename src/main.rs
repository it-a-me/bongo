#![warn(
    clippy::complexity,
    clippy::correctness,
    clippy::perf,
    clippy::pedantic,
    clippy::style
)]
mod backend;
mod list;
mod song;
mod sort;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;
use lofty::TaggedFile;

mod cli;
fn setup_logger(level: tracing::Level) -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .without_time()
            .with_max_level(level)
            .pretty()
            .finish(),
    )
}
fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    setup_logger(args.log_level)?;
    match args.command {
        cli::Command::Sort {
            destination_directory: clone_to_dir,
            source_directory,
        } => {
            let source_directory = source_directory.unwrap_or(std::env::current_dir()?);
            let copy = clone_to_dir.as_ref() != Some(&source_directory) && clone_to_dir.is_some();
            if copy {
                tracing::info!("source and dest are different.  Copy enabled");
            }
            let songs = find_songs(&source_directory)?;
            sort::sort(songs, &clone_to_dir.unwrap_or(source_directory), copy)?;
        }
        cli::Command::Update { backend, directory } => {
            update(backend, directory.unwrap_or(std::env::current_dir()?))?;
        }
        cli::Command::List { directory } => {
            let songs = find_songs(&directory.unwrap_or(std::env::current_dir()?))?;
            let mut untagged_songs = 0;
            for song in songs {
                if let Some(tagged) = list::list(&song.0) {
                    println!("{tagged}")
                } else {
                    untagged_songs += 1;
                }
            }
            if untagged_songs > 0 {
                println!("and {untagged_songs} untagged songs")
            }
        }
    }
    Ok(())
}

fn update(backend: cli::Backend, directory: PathBuf) -> anyhow::Result<()> {
    let songs = find_songs(&directory)?;
    match backend {
        #[cfg(feature = "backend-spotify")]
        cli::Backend::Spotify => {
            backend::spotify::update(songs.into_iter().map(|(s, _)| s).collect())?
        }
    }
    Ok(())
}

fn is_music_file(path: &Path) -> bool {
    let music_filetypes = vec!["mp3", "flac", "aac"];
    if !path.is_file() {
        return false;
    }
    let Some(Some(ext)) = path.extension().map(|e| e.to_str()) else {
        tracing::warn!("file '{}' does not contain a valid filetype", path.to_string_lossy());
        return false;
    };
    music_filetypes.contains(&ext)
}
