use std::path::{Path, PathBuf};

use anyhow::Result;
use redb::{TableDefinition, TypeName};

use crate::song::Song;
pub const SONGTABLE: TableDefinition<SongUuid, DbEntry> = TableDefinition::new("song_table");
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RelativePath(pub Vec<String>);
impl RelativePath {
    pub fn new(path: &Path, root: &Path) -> Result<Self> {
        let not_descendent_err = format!(
            "'{}' is not a subdir of '{}'",
            path.to_string_lossy(),
            root.to_string_lossy()
        );
        let mut path = path
            .into_iter()
            .map(|s| {
                s.to_str()
                    .map(ToString::to_string)
                    .ok_or(anyhow::anyhow!("invalid utf8 path"))
            })
            .collect::<Result<Vec<_>>>()?;
        for dir in root.into_iter().map(|os| {
            os.to_str()
                .map(ToString::to_string)
                .ok_or(anyhow::anyhow!("invalid utf8 path"))
        }) {
            let dir = dir?;
            if &path.remove(0) != &dir {
                anyhow::bail!("{not_descendent_err}");
            }
        }
        Ok(RelativePath(path))
    }
    pub fn rebase(&self, mut root: PathBuf) -> PathBuf {
        for path in &self.0 {
            root.push(path);
        }
        root
    }
}
impl std::fmt::Display for RelativePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut paths = String::from(self.0.get(0).map(ToOwned::to_owned).unwrap_or_default());
        for path in self.0.iter().skip(1) {
            paths.push('/');
            paths.push_str(path);
        }
        write!(f, "{paths}")
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
        bincode::deserialize(data).unwrap()
    }
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        bincode::serialize(value).unwrap()
    }
    fn type_name() -> TypeName {
        TypeName::new("song_entry")
    }
}
#[derive(derive_more::From, Debug, serde::Serialize, serde::Deserialize)]
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
        bincode::deserialize(data).unwrap()
    }
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        bincode::serialize(value).unwrap()
    }
    fn type_name() -> redb::TypeName {
        TypeName::new("song_uuid")
    }
}