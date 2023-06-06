use std::path::PathBuf;

use tracing::Level;
#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[arg(short, long, default_value_t = Level::INFO)]
    pub log_level: Level,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, clap::Subcommand, Clone)]
pub enum Command {
    #[cfg(feature = "edit")]
    ///edit a song's metadata
    Edit {
        #[arg(short, long)]
        editor: Option<String>,
        ///path the to song
        song: PathBuf,
    },
    #[cfg(feature = "fetch")]
    Fetch,
    ///print a song's metadata
    Show {
        ///path the to song
        song: Vec<PathBuf>,
    },
}
