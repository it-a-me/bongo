#![warn(
    clippy::complexity,
    clippy::correctness,
    clippy::nursery,
    clippy::pedantic
)]

use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

#[derive(thiserror::Error, Debug)]
///primary error type of relative-path
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("target is not a descendent of root.\ntarget:'{}'\nroot:'{}'", root.to_string_lossy(), target.to_string_lossy())]
    NotDescendent { root: PathBuf, target: PathBuf },
    #[error("invalid utf8 path ~~ '{0}'")]
    InvalidUtf8(PathBuf),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, derive_more::From, Eq, PartialEq)]
#[repr(transparent)]
///a path that can be split or rejoined to a root path
pub struct RelativePath(pub VecDeque<String>);
impl RelativePath {
    ///split a subpath off of a root path into a relative-path
    /// # Errors
    ///   [`Error::Io`]
    ///
    ///   [`Error::InvalidUtf8`]
    ///
    ///   [`Error::NotDescendent`]
    pub fn new(root: &Path, target: &Path) -> Result<Self, Error> {
        let non_descendent_err = || Error::NotDescendent {
            root: root.to_path_buf(),
            target: target.to_path_buf(),
        };
        //ensure that both paths are canontical
        let root = root.canonicalize()?;
        let target = target.canonicalize()?;
        //collect target into a vector to ease
        let mut target_vec = target.iter().collect::<VecDeque<_>>();
        for path in root.iter() {
            if path != target_vec.pop_front().ok_or_else(non_descendent_err)? {
                return Err(non_descendent_err());
            }
        }
        let relative_dir = target_vec
            .iter()
            .map(|s| {
                s.to_str()
                    .map(ToString::to_string)
                    .ok_or_else(|| Error::InvalidUtf8(target.clone()))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self(relative_dir))
    }
    #[must_use]
    pub fn rebase(&self, mut root: PathBuf) -> PathBuf {
        for path in &self.0 {
            root.push(path);
        }
        root
    }
}
impl std::fmt::Display for RelativePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut paths = self.0.get(0).map(ToOwned::to_owned).unwrap_or_default();
        for path in self.0.iter().skip(1) {
            paths.push('/');
            paths.push_str(path);
        }
        write!(f, "{paths}")
    }
}

impl FromIterator<String> for RelativePath {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
