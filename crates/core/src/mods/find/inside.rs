use crate::base::error::Error;
use std::path::Path;

/// More lines than this with a match are not listed.
const MOST_LINES: usize = 5000;

/// The lines of the file, from 1, that hold `text` in any case, each once; none for no text.
pub fn inside(file: &Path, text: &str) -> Result<Vec<usize>, Error> {
    let bytes = std::fs::read(file).map_err(|e| Error::file(file, e))?;
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let wanted = text.to_lowercase();
    let content = String::from_utf8_lossy(&bytes);
    Ok(content
        .lines()
        .enumerate()
        .filter(|(_, line)| line.to_lowercase().contains(&wanted))
        .map(|(at, _)| at + 1)
        .take(MOST_LINES)
        .collect())
}
