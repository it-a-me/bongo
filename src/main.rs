#![warn(
    clippy::complexity,
    clippy::correctness,
    clippy::perf,
    clippy::pedantic,
    clippy::style
)]
mod backend;
mod db;
mod edit;
mod list;
mod song;
mod sort;

use anyhow::Result;
use clap::Parser;

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
    let music_dir = if let Some(dir) = args.directory {
        dir
    } else {
        tracing::debug!("no directory supplied, defaulting to current directory");
        std::env::current_dir()?
    };
    match args.command {
        cli::Command::Sort {
            destination_directory: _,
        } => todo!(),
        cli::Command::Update { backend: _ } => todo!(),
        cli::Command::List { sub_directory: _ } => todo!(),
        cli::Command::Init { force_reinit } => {
            song::MusicDir::init(music_dir, force_reinit)?;
        }
        cli::Command::DumpDb => song::MusicDir::dumpdb(music_dir)?,
        cli::Command::Edit { song, editor } => edit::edit(song, editor)?,
    };
    Ok(())
}
