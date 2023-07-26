use std::path::{Path, PathBuf};

use lofty::{Accessor, TaggedFile, TaggedFileExt};

pub fn sort(songs: Vec<(TaggedFile, PathBuf)>, dest_dir: &Path, copy: bool) -> anyhow::Result<()> {
    for (tags, source) in songs {
        let mut dest = dest_dir.to_path_buf();
        for dir in sort_path(&tags, &source) {
            dest.push(dir)
        }
        std::fs::create_dir_all(dest.parent().unwrap())?;
        if source != dest {
            match copy {
                true => {
                    tracing::info!(
                        "copying file from '{}' to '{}'",
                        source.to_string_lossy(),
                        dest.to_string_lossy()
                    );
                    std::fs::copy(source, dest)?;
                }
                false => {
                    tracing::info!(
                        "moving file from '{}' to '{}'",
                        source.to_string_lossy(),
                        dest.to_string_lossy()
                    );
                    std::fs::copy(&source, dest)?;
                    std::fs::remove_file(source)?;
                }
            }
        }
    }
    Ok(())
}

fn sort_path(file: &TaggedFile, path: &Path) -> Vec<String> {
    let Some(tags )= file.tag(file.primary_tag_type()) else {
        return Vec::new();
    };
    let Some(filename)= tags.title().map(|title| {
        format!(
            "{title}.{}",
            path.extension()
                .expect("song does not have an extention")
                .to_string_lossy()
        )
    }) else {
        return Vec::new();
    };
    let artist = match tags.artist() {
        Some(artist) => artist.to_string(),
        None => String::from("Unknown Artist"),
    };
    if let Some(album) = tags.album() {
        vec![artist, album.to_string(), filename]
    } else if tags.artist().is_some() {
        vec![artist, String::from("Singles"), filename]
    } else {
        vec![artist, filename]
    }
}
