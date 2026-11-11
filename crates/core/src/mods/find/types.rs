use crate::base::error::Error;
use crate::said;
use std::path::Path;

/// The files a search reads, by extension: `xml, cs; json`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileTypes {
    /// Every file that is text, not only those with a given extension.
    Any,
    /// Lowercase, in order, each once.
    Only(Vec<String>),
}

impl Default for FileTypes {
    fn default() -> Self {
        FileTypes::Only(vec!["xml".into()])
    }
}

fn is_extension(token: &str) -> bool {
    !token.is_empty()
        && token
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

impl FileTypes {
    /// Split by comma, semicolon, bar or space; `.` (or `*`) alone is every text file.
    pub fn parse(text: &str) -> Result<FileTypes, Error> {
        let mut types: Vec<String> = Vec::new();
        for token in text
            .split([',', ';', '|', ' ', '\t'])
            .filter(|t| !t.is_empty())
        {
            if matches!(token, "." | "*" | "*.*") {
                return Ok(FileTypes::Any);
            }
            let extension = token.trim_start_matches('*').trim_start_matches('.');
            if !is_extension(extension) {
                return Err(Error::refused(said!(
                    "\"{}\" is not a file type; write extensions like xml, cs or json, or . for every text file",
                    token
                )));
            }
            let extension = extension.to_lowercase();
            if !types.contains(&extension) {
                types.push(extension);
            }
        }
        match types.is_empty() {
            true => Err(Error::refused(said!(
                "name at least one file type, like xml, or . for every text file"
            ))),
            false => Ok(FileTypes::Only(types)),
        }
    }

    pub fn reads_any_text(&self) -> bool {
        matches!(self, FileTypes::Any)
    }

    pub fn takes(&self, file: &Path) -> bool {
        match self {
            FileTypes::Any => true,
            FileTypes::Only(types) => file
                .extension()
                .is_some_and(|x| types.iter().any(|t| x.eq_ignore_ascii_case(t))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extensions_are_split_by_any_of_the_separators_and_kept_once_in_lowercase() {
        let parsed = |text| FileTypes::parse(text).unwrap();
        let only = |types: &[&str]| FileTypes::Only(types.iter().map(|t| t.to_string()).collect());
        assert_eq!(parsed("xml, cs; json"), only(&["xml", "cs", "json"]));
        assert_eq!(parsed("*.XML|.xml  txt"), only(&["xml", "txt"]));
        assert_eq!(parsed("xml"), FileTypes::default());
        assert_eq!(parsed("."), FileTypes::Any);
        assert_eq!(parsed("xml, ."), FileTypes::Any);
    }

    #[test]
    fn a_bad_entry_or_none_is_refused_with_a_sentence() {
        let said = |text| FileTypes::parse(text).unwrap_err().sentence().to_string();
        assert!(said("xml, a/b").starts_with("\"a/b\" is not a file type"));
        assert!(said(" , ;").starts_with("name at least one file type"));
        assert!(said("").starts_with("name at least one file type"));
    }

    #[test]
    fn a_file_is_taken_by_its_extension_in_any_case() {
        let types = FileTypes::parse("xml, cs").unwrap();
        assert!(types.takes(Path::new("a/B.XML")) && types.takes(Path::new("c.cs")));
        assert!(!types.takes(Path::new("c.json")) && !types.takes(Path::new("noext")));
        assert!(FileTypes::Any.takes(Path::new("noext")));
    }
}
