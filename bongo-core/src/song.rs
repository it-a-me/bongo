use crate::db::{Database, DbEntry, SongUuid, SONGTABLE};
use anyhow::Result;
use lofty::{AudioFile, Tag, TaggedFile, TaggedFileExt};
use redb::ReadableTable;
use relative_path::RelativePath;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
};
use uuid::Uuid;

pub struct Song {
    pub(crate) tagged: TaggedFile,
    pub(crate) uuid: Option<SongUuid>,
    pub path: PathBuf,
}

impl Song {
    pub fn parse(path: PathBuf) -> Result<Self, Error> {
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
        Ok(Self { tagged, uuid, path })
    }
    fn clean_tags(&mut self) -> Result<()> {
        let tags = self.tags_mut()?;
        let pre_clean = tags.item_count();
        tags.remove_empty();
        if tags.item_count() < pre_clean {
            self.tagged.save_to_path(&self.path)?;
        }
        Ok(())
    }

    fn write_uuid(&mut self, force: bool) -> Result<(), Error> {
        if self.uuid.is_some() && !force {
            return Ok(());
        }
        let uuid = Uuid::new_v4();
        {
            let tags = self.tags_mut()?;
            tags.re_map(lofty::TagType::Id3v2);
            if !tags.insert_text(lofty::ItemKey::CatalogNumber, uuid.to_string()) {
                return Err(OpenError::WriteTag.at(self.path.clone()));
            }
        }
        self.tagged
            .save_to_path(&self.path)
            .map_err(|e| OpenError::Save(e).at(self.path.clone()))?;
        self.uuid = Some(uuid.into());
        Ok(())
    }
    #[allow(dead_code)]
    fn tags(&self) -> Result<&Tag, Error> {
        self.tagged
            .primary_tag()
            .ok_or(OpenError::UntaggedFile.at(self.path.clone()))
    }
    fn tags_mut(&mut self) -> Result<&mut Tag, Error> {
        self.tagged
            .primary_tag_mut()
            .ok_or(OpenError::UntaggedFile.at(self.path.clone()))
    }
    fn to_db_entry(&self, root: &Path) -> anyhow::Result<DbEntry> {
        let relative_path = RelativePath::new(root, &self.path)?;
        Ok(DbEntry {
            old_path: relative_path,
        })
    }
    pub fn to_map(&self) -> Result<HashMap<String, String>, anyhow::Error> {
        let tags = self.tagged.get_tag(&self.path)?;
        Ok(tags
            .items()
            .filter_map(|i| {
                i.value()
                    .text()
                    .map(|s| (format!("{:?}", i.key()), s.to_owned()))
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
    pub(crate) songs: Vec<Song>,
    pub(crate) playlists: Vec<PathBuf>,
    pub(crate) root: PathBuf,
    pub(crate) db: Database,
}

impl MusicDir {
    pub fn init(root: PathBuf, force: bool) -> Result<Self> {
        let db = Database::init(&root, force)?;
        let songs = Self::find_songs(&root)?;
        let playlists = Self::find_playlists(&root)?;
        let writer = db.0.begin_write()?;
        {
            if force {
                writer.delete_table(SONGTABLE)?;
            }
        }
        writer.commit()?;

        let mut self_ = Self {
            songs,
            playlists,
            root,
            db,
        };
        self_.update(true)?;
        Ok(self_)
    }
    pub fn update(&mut self, write_uuid: bool) -> anyhow::Result<()> {
        if write_uuid {
            self.ensure_uuids()?;
        }
        self.append_songs()?;
        self.clean_old_uuid()?;
        for song in &mut self.songs {
            song.clean_tags()?;
        }
        Ok(())
    }
    fn append_songs(&mut self) -> anyhow::Result<()> {
        let writer = self.db.0.begin_write()?;
        {
            let mut song_tbl = writer.open_table(SONGTABLE)?;
            for song in &self.songs {
                if let Some(uuid) = &song.uuid {
                    if song_tbl.get(uuid)?.is_none() {
                        tracing::info!("adding '{}' to db", song.path.to_string_lossy());
                        song_tbl.insert(uuid, song.to_db_entry(&self.root)?)?;
                    }
                } else {
                    tracing::warn!(
                        "unable to add '{}' to db. Missing uuid",
                        song.path.to_string_lossy()
                    );
                }
            }
        }
        writer.commit()?;
        Ok(())
    }
    fn ensure_uuids(&mut self) -> anyhow::Result<()> {
        for song in self.songs.iter_mut().filter(|s| s.uuid.is_none()) {
            tracing::info!("writing uuid to '{}'", song.path.to_string_lossy());
            song.write_uuid(false)?;
        }
        Ok(())
    }
    fn clean_old_uuid(&mut self) -> anyhow::Result<()> {
        let writer = self.db.0.begin_write()?;
        {
            let mut song_tbl = writer.open_table(SONGTABLE)?;
            let table_uuids = song_tbl
                .iter()?
                .map(|e| e.map(|e| e.0.value()))
                .collect::<Result<Vec<_>, _>>()?;
            let song_uuids = self
                .songs
                .iter()
                .filter_map(|s| (s.uuid.as_ref()))
                .collect::<Vec<_>>();
            for uuid in table_uuids {
                if !song_uuids.contains(&&uuid) {
                    let removed_path = song_tbl
                        .remove(&uuid)?
                        .expect("a uuid from the db is not in the db")
                        .value()
                        .old_path;
                    tracing::info!(
                        "song with uuid '{}' no longer exists.  Removing '{}' from db",
                        &uuid,
                        removed_path.to_string()
                    );
                }
            }
        }
        writer.commit()?;
        Ok(())
    }
    pub fn open(dir: &Path) -> Result<Self> {
        let db = Database::open(dir)?;
        let db_root = db.1.parent().expect("db is both a file and a directory?");
        let songs = Self::find_songs(db_root)?;
        let playlists = Self::find_playlists(db_root)?;
        Ok(Self {
            songs,
            playlists,
            root: db_root.to_path_buf(),
            db,
        })
    }
    pub fn list(&self) {
        for song in &self.songs {
            println!("{}", song.path.to_string_lossy());
        }
    }
    fn find_songs(root: &Path) -> Result<Vec<Song>> {
        walkdir::WalkDir::new(root)
            .max_depth(5)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .collect::<Result<Vec<_>, walkdir::Error>>()?
            .into_iter()
            .filter(|e| is_music_file(e.path()))
            .map(|d| Song::parse(d.into_path()).map_err(Into::into))
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

pub trait GetTags {
    fn get_tag(&self, path: &Path) -> Result<&Tag, Error>;
    fn get_tag_mut(&mut self, path: &Path) -> Result<&mut Tag, Error>;
}
impl GetTags for TaggedFile {
    fn get_tag(&self, path: &Path) -> Result<&Tag, Error> {
        self.primary_tag()
            .ok_or_else(|| OpenError::UntaggedFile.at(path.to_path_buf()))
    }
    fn get_tag_mut(&mut self, path: &Path) -> Result<&mut Tag, Error> {
        self.primary_tag_mut()
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
