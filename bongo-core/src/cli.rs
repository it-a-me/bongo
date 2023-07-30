use std::path::PathBuf;

#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[derive(Debug)]
pub struct Cli {
    #[cfg_attr(feature = "clap", arg(short, long, default_value_t = tracing::Level::INFO))]
    ///the log level for the applications {trace, debug, info, warn, error}
    pub log_level: tracing::Level,
    #[cfg_attr(feature = "clap", arg(short, long))]
    ///disable browser auth for spotify
    pub no_browser: bool,
    ///music directory [default: ./ ]
    #[cfg_attr(feature = "clap", arg(short, long))]
    pub directory: Option<PathBuf>,
    #[cfg_attr(feature = "clap", command(subcommand))]
    pub command: Command,
}
#[cfg_attr(feature = "clap", derive(clap::Subcommand))]
#[derive(Debug, Clone)]
pub enum Command {
    ///sort files based off their metadata
    Sort {
        #[cfg_attr(feature = "clap", arg(short, long))]
        ///copy music files to dir rather than move files
        destination_directory: Option<PathBuf>,
        #[cfg_attr(feature = "clap", arg(short, long))]
        ///don't update the bongo db
        ignore_db: bool,
        #[cfg_attr(feature = "clap", arg(short, long))]
        ///create a bongo db if it doesn't exist
        auto_init: bool,
    },
    ///fetch metadata for files
    Fetch {
        #[cfg_attr(feature = "clap", arg(short, long))]
        ///metadata source
        backend: Backend,
    },
    ///update metadata for files
    Update {
        // #[cfg_attr(feature = "clap", arg(short, long))]
        // regen_uuid: bool,
    },
    ///create a bongo.db in the music dir
    Init {
        ///create even if a bongo.db already exists
        #[cfg_attr(feature = "clap", arg(short, long))]
        force_reinit: bool,
    },

    ///dump the contents of the database
    DumpDb,

    ///list tagged files in a directory
    List {
        // #[cfg_attr(feature = "clap", arg(short, long))]
        // ///music dir [default:current-dir]
        // sub_directory: Option<PathBuf>,
    },
    ///edit the metadata of a song
    Edit {
        ///path to the song to edit
        song: PathBuf,
        ///override the editor
        #[cfg_attr(feature = "clap", arg(short, long))]
        editor: Option<String>,
    },
    Show {
        ///path to the songs to edit
        songs: Vec<PathBuf>,
    },
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum Backend {
    #[cfg(feature = "backend-spotify")]
    Spotify,
}
