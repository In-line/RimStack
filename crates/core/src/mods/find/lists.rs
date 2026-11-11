use crate::base::stamp::FolderStamp;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Which files of a mod folder are searched. A folder is left out by its name
/// at any depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FolderSettings {
    pub all_text: bool,
    pub languages: bool,
    pub source: bool,
    pub textures: bool,
    pub git: bool,
}

impl Default for FolderSettings {
    fn default() -> Self {
        Self {
            all_text: false,
            languages: true,
            source: true,
            textures: true,
            git: true,
        }
    }
}

impl FolderSettings {
    fn leaves_out(&self, name: &OsStr) -> bool {
        let named = |left: &str| name.eq_ignore_ascii_case(left);
        (self.languages && named("Languages"))
            || (self.source && named("Source"))
            || (self.textures && named("Textures"))
            || (self.git && named(".git"))
    }

    fn takes(&self, file: &Path) -> bool {
        self.all_text
            || file
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("xml"))
    }

    fn walk(&self, root: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let mut waiting = vec![root.to_path_buf()];
        while let Some(folder) = waiting.pop() {
            let Ok(entries) = std::fs::read_dir(&folder) else {
                continue;
            };
            for entry in entries.flatten() {
                let Ok(kind) = entry.file_type() else {
                    continue;
                };
                let path = entry.path();
                if kind.is_dir() {
                    if !self.leaves_out(&entry.file_name()) {
                        waiting.push(path);
                    }
                } else if self.takes(&path) && (!kind.is_symlink() || path.is_file()) {
                    found.push(path);
                }
            }
        }
        found.sort_unstable();
        found
    }
}

type Kept = (FolderStamp, Arc<Vec<PathBuf>>);

/// The files to search in each mod folder, kept for the session and reused
/// while the folder's stamp is unchanged.
#[derive(Default)]
pub struct FileLists {
    kept: Mutex<HashMap<(PathBuf, FolderSettings), Kept>>,
}

impl FileLists {
    pub fn get(&self, folder: &Path, settings: FolderSettings) -> Arc<Vec<PathBuf>> {
        let stamp = FolderStamp::of(folder, |name| settings.leaves_out(name));
        let key = (folder.to_path_buf(), settings);
        if let Some((kept, list)) = self.kept.lock().unwrap().get(&key)
            && *kept == stamp
        {
            return list.clone();
        }
        let list = Arc::new(settings.walk(folder));
        self.kept.lock().unwrap().insert(key, (stamp, list.clone()));
        list
    }
}
