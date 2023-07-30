#![warn(
    clippy::complexity,
    clippy::correctness,
    clippy::perf,
    clippy::pedantic,
    clippy::style
)]
#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;

use anyhow::Result;
use bongo_core::{song, db::SONGTABLE, rexports::redb::ReadableTable};
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
            destination_directory,
            ignore_db,
            auto_init
        } => {
            if ignore_db && auto_init {
                anyhow::bail!("unable to both ignore and create a db");
            }
            song::MusicDir::open(&music_dir)?.sort(destination_directory, ignore_db, auto_init)?;
        },
        cli::Command::Fetch { backend: _ } => todo!(),
        cli::Command::Update{/* regen_uuid*/} => {
            let mut music_dir= song::MusicDir::open(&music_dir)?;
            music_dir.update(true)?;
        
        },
        cli::Command::List { /*sub_directory: _*/ } => song::MusicDir::open(&music_dir)?.list(),
        cli::Command::Init { force_reinit } => {
            song::MusicDir::init(music_dir, force_reinit)?;
        }
        cli::Command::Show { songs }=> {
            let mut show_map = HashMap::with_capacity(songs.len());
            for path in songs 
            {
                match song::Song::parse(path.clone()).map(|s| s.to_map()){
                    Ok(Ok(map)) => {show_map.insert(path.to_string_lossy().into_owned(), map);},
                    Ok(Err(e)) => tracing::error!("{e}"),
                    Err(e) => tracing::error!("{e}"),                   
                }
                           
            }
            print!("{}", toml::to_string_pretty(&show_map)?);
        },
        cli::Command::DumpDb => {
            let db = bongo_core::db::Database::open(&music_dir)?;
            let reader = db.0.begin_read()?;
            {
                let song_tbl = reader.open_table(SONGTABLE)?;
                for entry in song_tbl.iter()?{
                    let entry = entry?;
                    let (u, e) = entry;
                    (u.value(), e.value());

                }
                let x = song_tbl.iter()?
                    .map(
                        |x| 
                            x.map_err(Into::into).map(|(u, e)| (u.value().0, e.value()))).collect::<Result<HashMap<_,_>>>()?;
                println!("{}", toml::to_string_pretty(&x)?);

            }
        }
        cli::Command::Edit { .. } => todo!(),
    };
    Ok(())
}
