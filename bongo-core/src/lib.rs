#![warn(
    clippy::complexity,
    clippy::correctness,
    clippy::perf,
    clippy::pedantic,
    clippy::style
)]
#![allow(clippy::module_name_repetitions)]
pub mod db;
mod error;
pub use error::Error;
pub mod song;
mod sort;
pub mod rexports {
    pub use redb;
}
