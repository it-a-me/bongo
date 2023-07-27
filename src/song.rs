use crate::db::{DbEntry, RelativePath, SongUuid, SONGTABLE};
use anyhow::Result;
use lofty::{AudioFile, Tag, TaggedFile, TaggedFileExt};
use redb::ReadableTable;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
};
use uuid::Uuid;

pub struct Song {
    tagged: TaggedFile,
    uuid: Option<SongUuid>,
    path: PathBuf,
}

impl Song {
    pub fn parse(path: PathBuf, write_uuid: bool) -> Result<Self, Error> {
        let mut tagged =
            lofty::read_from_path(&path).map_err(|e| OpenError::from(e).at(path.clone()))?;
        let tags = tagged.get_tag_mut(&path)?;
        let uuid = match tags
            .get_string(&lofty::ItemKey::CatalogNumber)
            .map(Uuid::from_str)
        {
            Some(Ok(uuid)) => Some(uuid.into()),
            _ => None,
        };
        let mut self_ = Self { tagged, uuid, path };
        if write_uuid && self_.uuid.is_none() {
            self_.write_uuid()?;
        }
        Ok(self_)
    }

    fn write_uuid(&mut self) -> Result<(), Error> {
        let uuid = Uuid::new_v4();
        {
            let tags = self.tags_mut()?;
            tags.re_map(lofty::TagType::Id3v2);
            if !tags.insert_text(lofty::ItemKey::CatalogNumber, uuid.to_string()) {
                return Err(OpenError::WriteTag.at(self.path.to_owned()));
            }
        }
        self.tagged
            .save_to_path(self.path.to_owned())
            .map_err(|e| OpenError::Save(e).at(self.path.to_owned()))?;
        self.uuid = Some(uuid.into());
        Ok(())
    }
    #[allow(dead_code)]
    fn tags(&self) -> Result<&Tag, Error> {
        self.tagged
            .primary_tag()
            .ok_or(OpenError::UntaggedFile.at(self.path.to_owned()))
    }
    fn tags_mut(&mut self) -> Result<&mut Tag, Error> {
        self.tagged
            .primary_tag_mut()
            .ok_or(OpenError::UntaggedFile.at(self.path.to_owned()))
    }
    fn to_db_entry(&self, root: &Path) -> anyhow::Result<DbEntry> {
        let relative_path = RelativePath::new(&self.path, root)?;
        Ok(DbEntry {
            old_path: relative_path,
        })
    }
    pub fn to_string_pretty(&self) -> Result<String, Error> {
        let tags = self.tagged.get_tag(&self.path)?;

        Ok(tags
            .items()
            .filter_map(|i| {
                i.value()
                    .text()
                    .map(|s| format!("{:?} => '{s}'\n", i.key()))
            })
            .collect())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum OpenError {
    #[error("error parsing file {0}")]
    Parse(#[from] lofty::LoftyError),
    #[error("error writing tags to file")]
    WriteTag,
    #[error("error writing tags to file {0}")]
    Save(lofty::LoftyError),
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
    playlists: Vec<PathBuf>,
    root: PathBuf,
    db: redb::Database,
}

impl MusicDir {
    pub fn init(root: PathBuf, force: bool) -> Result<Self> {
        let db = redb::Database::create(root.join(".bongo.db"))?;
        let songs = Self::find_songs(&root, true)?;
        let playlists = Self::find_playlists(&root)?;
        let writer = db.begin_write()?;
        {
            if force {
                writer.delete_table(SONGTABLE)?;
            }
            let mut song_tbl = writer.open_table(SONGTABLE)?;
            for song in &songs {
                if let Some(uuid) = &song.uuid {
                    if song_tbl.get(uuid)?.is_none() {
                        song_tbl.insert(uuid, song.to_db_entry(&root)?)?;
                    }
                } else {
                    tracing::warn!(
                        "'{}' does not have a uuid.  Unable to save it to .bongo.db",
                        song.path.to_string_lossy()
                    );
                }
            }
        }
        writer.commit()?;

        Ok(Self {
            songs,
            playlists,
            root,
            db,
        })
    }
    pub fn dumpdb(root: PathBuf) -> Result<()> {
        let db = redb::Database::open(root.join(".bongo.db"))?;
        let reader = db.begin_read()?;
        let tbl = reader.open_table(SONGTABLE)?;
        let db_map = tbl
            .iter()?
            .map(|e| e.map(|(k, v)| (k.value().0, v.value())))
            .collect::<Result<HashMap<_, _>, _>>()?;
        println!("{}", toml::to_string_pretty(&db_map)?);
        Ok(())
    }
    fn find_songs(root: &Path, write_uuid: bool) -> Result<Vec<Song>> {
        walkdir::WalkDir::new(root)
            .max_depth(5)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .collect::<Result<Vec<_>, walkdir::Error>>()?
            .into_iter()
            .filter(|e| is_music_file(e.path()))
            .map(|d| Song::parse(d.into_path(), write_uuid).map_err(Into::into))
            .collect::<Result<Vec<_>>>()
    }
    fn find_playlists(root: &Path) -> Result<Vec<PathBuf>> {
        let paths = root.read_dir()?;
        paths
            .into_iter()
            .map(|entry| entry.map(|f| f.path()).map_err(std::convert::Into::into))
            .filter(|e| {
                if let Ok(dir) = e {
                    dir.extension().unwrap_or_default().to_string_lossy() == ".m3u"
                } else {
                    true
                }
            })
            .collect()
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
    let Some(Some(ext)) = path.extension().map(std::ffi::OsStr::to_str) else {
        tracing::warn!("file '{}' does not contain a valid filetype", path.to_string_lossy());
        return false;
    };
    music_filetypes.contains(&ext)
}
