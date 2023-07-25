use std::path::{Path, PathBuf};

use anyhow::Result;
use lofty::{Tag, TaggedFile, TaggedFileExt};
use uuid::Uuid;

pub struct Song {
    tagged: TaggedFile,
    uuid: uuid::Uuid,
    path: PathBuf,
}

impl Song {
    pub fn parse(path: PathBuf, write_uuid: bool) -> Result<Self, Error> {
        let mut tagged =
            lofty::read_from_path(&path).map_err(|e| OpenError::from(e).at(path.to_path_buf()))?;
        let tags = tagged.get_tag_mut(&path)?;
        let uuid_field = tags.get_string(&lofty::ItemKey::CatalogNumber);
        let uuid = if let Some(uuid) = uuid_field {
            uuid::Uuid::parse_str(uuid).map_err(|e| OpenError::from(e).at(path.to_path_buf()))?
        } else {
            if !write_uuid {
                return Err(OpenError::MissingUuid.at(path.to_path_buf()));
            }
            let uuid = Uuid::new_v4();
            if tags.insert_text(lofty::ItemKey::CatalogNumber, uuid.to_string()) {
                uuid
            } else {
                return Err(OpenError::Write.at(path.to_path_buf()));
            }
        };

        Ok(Self { tagged, uuid, path })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum OpenError {
    #[error("error parsing file {0}")]
    Parse(#[from] lofty::LoftyError),
    #[error("error writing tags to file")]
    Write,
    #[error("missing uuid")]
    MissingUuid,
    #[error("Invalid uuid '{0}'")]
    InvalidUuid(#[from] uuid::Error),
    #[error("untagged file")]
    UntaggedFile,
}
impl OpenError {
    pub fn at(self, path: PathBuf) -> Error {
        Error { error: self, path }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("failed to parse '{path}'. {error}")]
pub struct Error {
    error: OpenError,
    path: PathBuf,
}

pub struct MusicDir {
    songs: Vec<Song>,
    root: PathBuf,
    db: redb::Database,
}

impl MusicDir {
    pub fn init(root: PathBuf) -> Result<Self> {
        let db = redb::Database::create(root.join(".bongo.db"))?;
        let songs = Self::find_songs(&root, true)?;
        Ok(Self { songs, root, db })
    }
    fn song_table(&self) {}
    fn find_songs(root: &Path, write_uuid: bool) -> Result<Vec<Song>> {
        Ok(walkdir::WalkDir::new(root)
            .max_depth(5)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| e.file_name().to_string_lossy().chars().nth(0) != Some('.'))
            .collect::<Result<Vec<_>, walkdir::Error>>()?
            .into_iter()
            .filter(|e| is_music_file(e.path()))
            .map(|d| Song::parse(d.into_path(), write_uuid).map_err(Into::into))
            .collect::<Result<Vec<_>>>()?)
    }
}

trait GetTags {
    fn get_tag(&self, path: &Path) -> Result<&Tag, Error>;
    fn get_tag_mut(&mut self, path: &Path) -> Result<&mut Tag, Error>;
}
impl GetTags for TaggedFile {
    fn get_tag(&self, path: &Path) -> Result<&Tag, Error> {
        self.tag(self.primary_tag_type())
            .ok_or_else(|| OpenError::UntaggedFile.at(path.to_path_buf()))
    }
    fn get_tag_mut(&mut self, path: &Path) -> Result<&mut Tag, Error> {
        self.tag_mut(self.primary_tag_type())
            .ok_or_else(|| OpenError::UntaggedFile.at(path.to_path_buf()))
    }
}
fn is_music_file(path: &Path) -> bool {
    let music_filetypes = vec!["mp3", "flac", "aac"];
    if !path.is_file() {
        return false;
    }
    let Some(Some(ext)) = path.extension().map(|e| e.to_str()) else {
        tracing::warn!("file '{}' does not contain a valid filetype", path.to_string_lossy());
        return false;
    };
    music_filetypes.contains(&ext)
}
