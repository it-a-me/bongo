use redb::{TableDefinition, TypeName};
use relative_path::RelativePath;
use std::path::{Path, PathBuf};

pub const DBNAME: &str = ".bongo.db";
pub const SONGTABLE: TableDefinition<SongUuid, DbEntry> = TableDefinition::new("song_table");

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to find db in '{0}'")]
    UnableToFindDb(PathBuf),

    #[error(
        "unable to create db in '{target}' because it is a subdirectory of existing db '{existing}'"
    )]
    DbAlreadyExists { existing: PathBuf, target: PathBuf },

    #[error(transparent)]
    Db(#[from] redb::DatabaseError),

    #[error(transparent)]
    Transaction(#[from] redb::TransactionError),

    #[error(transparent)]
    Table(#[from] redb::TableError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub struct Database(pub redb::Database, pub PathBuf);
impl Database {
    pub fn init(dbroot: &Path, force: bool) -> Result<Self, Error> {
        let db_path = dbroot.join(DBNAME);
        if force && db_path.exists() {
            std::fs::remove_file(&db_path)?;
        }
        if let Some(existing) = Self::find_db(dbroot.to_path_buf()) {
            return Err(Error::DbAlreadyExists {
                existing,
                target: dbroot.to_path_buf(),
            });
        }
        if !dbroot.exists() {
            std::fs::create_dir(dbroot)?;
        }
        Ok(Self(redb::Database::create(&db_path)?, db_path))
    }
    pub fn open(dir: &Path) -> Result<Self, Error> {
        let path = Self::find_db(dir.to_path_buf())
            .ok_or_else(|| Error::UnableToFindDb(dir.to_path_buf()))?;
        Ok(Self(redb::Database::open(&path)?, path))
    }
    fn find_db(mut current_dir: PathBuf) -> Option<PathBuf> {
        current_dir.push("");
        let mut db_path;
        while let Some(parent) = current_dir.parent() {
            db_path = current_dir.join(DBNAME);
            if db_path.exists() {
                return Some(db_path);
            }
            current_dir = parent.to_path_buf();
        }
        None
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DbEntry {
    pub old_path: RelativePath,
}
impl redb::RedbValue for DbEntry {
    type SelfType<'a> = Self;
    type AsBytes<'a> = Vec<u8>;
    fn fixed_width() -> Option<usize> {
        None
    }
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        postcard::from_bytes(data).unwrap()
    }
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        postcard::to_allocvec(value).unwrap()
    }
    fn type_name() -> TypeName {
        TypeName::new("song_entry")
    }
}
#[derive(
    derive_more::From, derive_more::Display, Debug, serde::Serialize, serde::Deserialize, PartialEq,
)]
pub struct SongUuid(pub uuid::Uuid);
impl redb::RedbKey for SongUuid {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}
impl redb::RedbValue for SongUuid {
    type SelfType<'a> = Self;
    type AsBytes<'a> = Vec<u8>;
    fn fixed_width() -> Option<usize> {
        None
    }
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        postcard::from_bytes(data).unwrap()
    }
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        postcard::to_allocvec(value).unwrap()
    }
    fn type_name() -> redb::TypeName {
        TypeName::new("song_uuid")
    }
}
