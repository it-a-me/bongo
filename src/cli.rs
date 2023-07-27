use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
pub struct Cli {
    #[arg(short, long, default_value_t = tracing::Level::INFO)]
    ///the log level for the applications {trace, debug, info, warn, error}
    pub log_level: tracing::Level,
    #[arg(short, long)]
    ///disable browser auth for spotify
    pub no_browser: bool,
    ///music directory [default: ./ ]
    pub directory: Option<PathBuf>,
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
    },
    ///update metadata for files
    Update {
        #[arg(short, long)]
        ///metadata source
        backend: Backend,
    },
    ///create a bongo.db in the music dir
    Init {
        ///create even if a bongo.db already exists
        #[arg(short, long)]
        force_reinit: bool,
    },

    ///dump the contents of the database
    DumpDb,

    ///list tagged files in a directory
    List {
        // #[arg(short, long)]
        // ///music dir [default:current-dir]
        // sub_directory: Option<PathBuf>,
    },
    ///edit the metadata of a song
    Edit {
        ///path to the song to edit
        song: PathBuf,
        ///override the editor
        #[arg(short, long)]
        editor: Option<String>,
    },
    Show {
        ///path to the songs to edit
        songs: Vec<PathBuf>,
    },
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum Backend {
    #[cfg(feature = "backend-spotify")]
    Spotify,
}
