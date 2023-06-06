use anyhow::Ok;
use lofty::{Accessor, Tag, TagType, TaggedFile, TaggedFileExt};
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
        cli::Command::Edit {
            song: songs,
            editor,
        } => {
            let editor = if let Some(editor) = editor {
                Ok(editor)
            } else {
                Ok(std::env::var("EDITOR")?)
            }?;
            edit(songs, &editor)?;
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
fn edit(paths: Vec<PathBuf>, editor: &str) -> anyhow::Result<()> {
    use std::collections::HashMap;

    use lofty::TagExt;
    let mut files = Vec::new();
    for path in paths {
        let mut file = lofty::read_from_path(&path)?;
        let mut fileinfo = FileInfo::new(
            file.tag(file.primary_tag_type()).unwrap(),
            file.primary_tag_type(),
        );
        files.push(MusicFile {
            file,
            fileinfo,
            path,
        });
    }
    let temp = tempfile::NamedTempFile::new()?.into_temp_path();

    std::fs::write(
        &temp,
        toml::to_string_pretty(
            &files
                .iter()
                .map(|f| (f.path.file_name().unwrap().to_string_lossy(), &f.fileinfo))
                .collect::<HashMap<_, _>>(),
        )?,
    )?;
    {
        let mut edit = true;
        while edit {
            std::process::Command::new(editor).arg(&temp).status()?;
            edit =
                match toml::from_str::<HashMap<String, FileInfo>>(&std::fs::read_to_string(&temp)?)
                {
                    Err(err) => {
                        println!("{err}");
                        dialoguer::Confirm::new()
                            .with_prompt("edit again?")
                            .interact()?
                    }
                    core::result::Result::Ok(new) => {
                        files = files
                            .into_iter()
                            .zip(new.into_values())
                            .map(|(mut music_file, fileinfo)| {
                                music_file.fileinfo = fileinfo;
                                music_file
                            })
                            .collect();
                        false
                    }
                }
        }
    }
    for mut music_file in files {
        let tags = music_file
            .file
            .tag_mut(music_file.file.primary_tag_type())
            .unwrap();
        music_file.fileinfo.update(tags);
        tags.save_to_path(music_file.path)?;
    }
    Ok(())
}
struct MusicFile {
    fileinfo: FileInfo,
    file: TaggedFile,
    path: PathBuf,
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
