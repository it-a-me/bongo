use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
pub struct Cli {
    #[arg(short, long, default_value_t = tracing::Level::INFO)]
    ///the log level for the applications {trace, debug, info, warn, error}
    pub log_level: tracing::Level,
    #[arg(short, long)]
    ///disable browser auth for spotify
    pub no_browser: bool,
    #[command(subcommand)]
    pub command: Command,
}
#[derive(clap::Subcommand, Debug, Clone)]
pub enum Command {
    ///sort files based off their metadata
    Sort {
        #[arg(short, long)]
        ///copy music files to dir rather than move files
        destination_directory: Option<PathBuf>,
        #[arg(short, long)]
        ///music dir [default:current-dir]
        source_directory: Option<PathBuf>,
    },
    ///update metadata for files
    Update {
        #[arg(short, long)]
        ///music dir [default:current-dir]
        directory: Option<PathBuf>,
        backend: Backend,
    },
    ///list tagged files in a directory
    List {
        ///music dir [default:current-dir]
        directory: Option<PathBuf>,
    },
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum Backend {
    #[cfg(feature = "backend-spotify")]
    Spotify,
}
