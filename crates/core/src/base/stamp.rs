use std::collections::hash_map::DefaultHasher;
use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// The modification times of a folder and all its subfolders, which change
/// when a file or folder is added, removed or renamed inside one, and not when
/// a file is edited in place. So it vouches for lists of names, never contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolderStamp {
    folders: usize,
    sum: u64,
}

impl FolderStamp {
    /// Only folders are read, links to folders are not entered, and a folder
    /// named by `skip` is left out. Whatever cannot be read counts as nothing.
    pub fn of(root: &Path, skip: impl Fn(&OsStr) -> bool) -> Self {
        let mut stamp = Self { folders: 0, sum: 0 };
        let mut waiting = vec![root.to_path_buf()];
        while let Some(folder) = waiting.pop() {
            let mut hasher = DefaultHasher::new();
            folder.hash(&mut hasher);
            if let Ok(changed) = std::fs::metadata(&folder).and_then(|m| m.modified()) {
                changed.hash(&mut hasher);
            }
            stamp.folders += 1;
            stamp.sum = stamp.sum.wrapping_add(hasher.finish());
            let Ok(entries) = std::fs::read_dir(&folder) else {
                continue;
            };
            for entry in entries.flatten() {
                let is_dir = entry.file_type().is_ok_and(|kind| kind.is_dir());
                if is_dir && !skip(&entry.file_name()) {
                    waiting.push(entry.path());
                }
            }
        }
        stamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    pub fn touch_dir(dir: &Path, later: u64) {
        let file = std::fs::File::open(dir).unwrap();
        file.set_modified(SystemTime::now() + Duration::from_secs(later))
            .unwrap();
    }

    #[test]
    fn a_change_in_a_subfolder_changes_the_stamp_and_a_skipped_one_does_not() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("a/b")).unwrap();
        std::fs::create_dir_all(tmp.path().join("Textures")).unwrap();
        let skip = |name: &OsStr| name == "Textures";
        let first = FolderStamp::of(tmp.path(), skip);
        assert_eq!(first, FolderStamp::of(tmp.path(), skip));
        touch_dir(&tmp.path().join("Textures"), 100);
        assert_eq!(first, FolderStamp::of(tmp.path(), skip));
        touch_dir(&tmp.path().join("a/b"), 200);
        assert_ne!(first, FolderStamp::of(tmp.path(), skip));
    }
}
