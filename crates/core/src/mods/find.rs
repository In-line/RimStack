//! Searching the text of mods' files.

pub mod inside;
pub mod lines;
pub mod lists;
pub mod preview;
pub mod search;
pub mod types;

pub use inside::inside;
pub use lines::{Line, lines_of};
pub use lists::{FileLists, FolderSettings};
pub use preview::{Preview, preview};
pub use search::{Batch, FileHits, Hit, Progress, Summary, Target, search};
pub use types::FileTypes;

#[cfg(test)]
mod tests;
