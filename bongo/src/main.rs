use anyhow::Ok;
use lofty::{Accessor, Tag, TagType, TaggedFileExt};
use std::path::PathBuf;

use clap::Parser;
mod cli;
fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .pretty()
            .with_max_level(args.log_level)
            .finish(),
    )?;

    match args.command {
        cli::Command::Show { song: songs } => {
            for song in songs {
                show(song)?
            }
        }
        #[cfg(feature = "edit")]
        cli::Command::Edit { song, editor } => {
            let editor = if let Some(editor) = editor {
                Ok(editor)
            } else {
                Ok(std::env::var("EDITOR")?)
            }?;
            edit(song, &editor)?;
        }
    }
    Ok(())
}

#[tracing::instrument]
fn show(path: PathBuf) -> anyhow::Result<()> {
    let file = lofty::read_from_path(path)?;
    let tags = file.tag(file.primary_tag_type()).unwrap();
    let fileinfo = FileInfo::new(tags, tags.tag_type());
    println!("{}", toml::to_string(&fileinfo)?);
    Ok(())
}
#[cfg(feature = "edit")]
fn edit(path: PathBuf, editor: &str) -> anyhow::Result<()> {
    use lofty::TagExt;
    let mut file = lofty::read_from_path(&path)?;
    let mut fileinfo = FileInfo::new(
        file.tag(file.primary_tag_type()).unwrap(),
        file.primary_tag_type(),
    );
    let temp = tempfile::NamedTempFile::new()?.into_temp_path();

    std::fs::write(&temp, toml::to_string_pretty(&fileinfo)?)?;
    std::process::Command::new(editor).arg(&temp).status()?;

    fileinfo = toml::from_str(&std::fs::read_to_string(temp)?)?;
    let tags = file.tag_mut(file.primary_tag_type()).unwrap();
    fileinfo.update(tags);
    tags.save_to_path(path)?;
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FileInfo {
    tag_type: String,
    artist: Option<String>,
    album: Option<String>,
    title: Option<String>,
    track: Option<u32>,
}
impl FileInfo {
    pub fn new(tags: &Tag, tag_type: TagType) -> Self {
        Self {
            tag_type: format!("{:?}", tag_type),
            artist: tags.artist().map(|s| s.to_string()),
            album: tags.album().map(|s| s.to_string()),
            title: tags.title().map(|s| s.to_string()),
            track: tags.track(),
        }
    }

    #[cfg(feature = "modify")]
    pub fn update(self, tag: &mut Tag) {
        if let Some(artist) = self.artist {
            tag.set_artist(artist);
        }
        if let Some(album) = self.album {
            tag.set_album(album);
        }
        if let Some(title) = self.title {
            tag.set_title(title);
        }
        if let Some(track) = self.track {
            tag.set_track(track);
        }
    }
}
