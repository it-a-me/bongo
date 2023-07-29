use std::path::PathBuf;

use lofty::Accessor;

use crate::{
    db::RelativePath,
    song::{GetTags, MusicDir},
};

impl MusicDir {
    pub fn sort(
        &mut self,
        destination_dir: Option<PathBuf>,
        ignore_db: bool,
        auto_init: bool,
    ) -> anyhow::Result<()> {
        if let Some(destination_dir) = destination_dir {
            if self.root == destination_dir {
                anyhow::bail!("source and destination directories are the same");
            }
            if !destination_dir.exists() {
                std::fs::create_dir(&destination_dir)?;
            }
            if !destination_dir.is_dir() {
                anyhow::bail!("destination is not a directory");
            }
            for (dest, source) in self.song_paths()? {
                let dest = dest.rebase(destination_dir.clone());
                tracing::info!(
                    "copying '{}' to '{}'",
                    source.to_string_lossy(),
                    dest.to_string_lossy()
                );
                if &dest == source {
                    anyhow::bail!("unable to copy to self");
                }
                std::fs::create_dir_all(dest.parent().unwrap())?;
                std::fs::copy(source, dest)?;
            }
            if auto_init {
                Self::init(destination_dir, false)?;
            }
        } else {
            for (dest, source) in self.song_paths()? {
                let dest = dest.rebase(self.root.clone());
                if &dest != source {
                    tracing::info!(
                        "moving '{}' to '{}'",
                        source.to_string_lossy(),
                        dest.to_string_lossy()
                    );
                    std::fs::create_dir_all(dest.parent().unwrap())?;
                    std::fs::copy(source, dest)?;
                    std::fs::remove_file(source)?;
                }
            }
            if !ignore_db {
                self.update(false)?;
            }
        }
        Ok(())
    }

    fn song_paths(&self) -> anyhow::Result<Vec<(RelativePath, &PathBuf)>> {
        let mut paths = Vec::with_capacity(self.songs.len());
        for song in &self.songs {
            let tags = song.tagged.get_tag(&song.path)?;
            let mut title = if let Some(title) = tags.title() {
                title
            } else {
                song.path
                    .file_name()
                    .ok_or(anyhow::anyhow!(
                        "'{}' is not a file",
                        song.path.to_string_lossy()
                    ))?
                    .to_string_lossy()
            };
            title.to_mut().push('.');
            title
                .to_mut()
                .push_str(&song.path.extension().unwrap().to_string_lossy());
            let album = tags.album().unwrap_or("Singles".into());
            let artist = tags.artist().unwrap_or("UnknownArtist".into());
            let relative_path = RelativePath::from(
                [artist, album, title]
                    .map(|s| s.to_string())
                    .into_iter()
                    .collect::<Vec<_>>(),
            );
            paths.push((relative_path, &song.path));
        }
        Ok(paths)
    }
}
