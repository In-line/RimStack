use super::App;
use crate::changes::Changes;
use rimstack_core::base::error::Error;
use rimstack_core::base::pattern::{Pattern, Switches};
use rimstack_core::base::sentence::Sentence;
use rimstack_core::kept::searches::{Search, Searches};
use rimstack_core::mods::find::{
    Batch, FileLists, FolderSettings, Progress, Summary, Target, preview, search,
};
use rimstack_core::mods::library::{Library, ModId};
use rimstack_core::rimworld::paths::Source;
use rimstack_core::{say, say_n};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

mod results;
pub use results::{Cell, Folded, MOST_HITS, Op, Part, ROLES, Results, RowRef};

/// Where a search looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    Active,
    Inactive,
    Installed,
    Selected,
    Local,
    Workshop,
    Settings,
}

impl Place {
    const ALL: [Place; 7] = [
        Place::Active,
        Place::Inactive,
        Place::Installed,
        Place::Selected,
        Place::Local,
        Place::Workshop,
        Place::Settings,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Place::Active => "active",
            Place::Inactive => "inactive",
            Place::Installed => "installed",
            Place::Selected => "selected",
            Place::Local => "local",
            Place::Workshop => "workshop",
            Place::Settings => "settings",
        }
    }

    fn of(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|place| place.id() == id)
    }

    fn label(self) -> Sentence {
        match self {
            Place::Active => say!("Active mods"),
            Place::Inactive => say!("Inactive mods"),
            Place::Installed => say!("All installed mods"),
            Place::Selected => say!("The selected mod"),
            Place::Local => say!("Local mods"),
            Place::Workshop => say!("Workshop mods"),
            Place::Settings => say!("Mod settings"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub text: String,
    pub switches: Switches,
    pub place: Place,
    pub selected: Option<ModId>,
    pub folders: FolderSettings,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            text: String::new(),
            switches: Switches::default(),
            place: Place::Active,
            selected: None,
            folders: FolderSettings::default(),
        }
    }
}

impl Request {
    fn kept(&self) -> Search {
        Search {
            text: self.text.clone(),
            case: self.switches.case,
            regex: self.switches.regex,
            whole_word: self.switches.whole_word,
            place: self.place.id().to_string(),
            all_text: self.folders.all_text,
            languages: self.folders.languages,
            source: self.folders.source,
            textures: self.folders.textures,
            git: self.folders.git,
        }
    }

    /// Where it looked is the selected mod only while one is.
    fn from_kept(search: &Search, selected: Option<ModId>) -> Self {
        let place = Place::of(&search.place).unwrap_or(Place::Active);
        let place = match (place, selected) {
            (Place::Selected, None) => Place::Active,
            (place, _) => place,
        };
        Self {
            text: search.text.clone(),
            switches: Switches {
                case: search.case,
                regex: search.regex,
                whole_word: search.whole_word,
            },
            place,
            selected,
            folders: FolderSettings {
                all_text: search.all_text,
                languages: search.languages,
                source: search.source,
                textures: search.textures,
                git: search.git,
            },
        }
    }

    pub fn flip(&mut self, id: &str) {
        let (s, f) = (&mut self.switches, &mut self.folders);
        match id {
            "case" => s.case = !s.case,
            "regex" => s.regex = !s.regex,
            "word" => s.whole_word = !s.whole_word,
            "languages" => f.languages = !f.languages,
            "source" => f.source = !f.source,
            "textures" => f.textures = !f.textures,
            "git" => f.git = !f.git,
            _ => {}
        }
    }

    /// `group` is "where" or "files".
    pub fn choose(&mut self, group: &str, id: &str) {
        match group {
            "where" => {
                if let Some(place) = Place::of(id) {
                    self.place = place;
                }
            }
            "files" => self.folders.all_text = id == "text",
            _ => {}
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Choice {
    pub id: &'static str,
    pub label: Sentence,
    pub chosen: bool,
    pub available: bool,
}

#[derive(Debug, Serialize)]
pub struct Options {
    pub switches: Vec<Choice>,
    #[serde(rename = "where")]
    pub places: Vec<Choice>,
    pub files: Vec<Choice>,
    pub leave: Vec<Choice>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recent {
    pub text: String,
    pub place: Sentence,
    pub case: bool,
    pub regex: bool,
    pub word: bool,
    pub all_text: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewLine {
    pub number: usize,
    pub text: String,
    pub marked: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewShown {
    pub path: String,
    pub line: usize,
    pub lines: Vec<PreviewLine>,
    pub problem: Option<Sentence>,
}

/// Valid for the search that started it; a newer search makes it stale.
pub struct Started {
    pub generation: u64,
    cancel: Arc<AtomicBool>,
    pattern: Pattern,
    targets: Vec<Target<Option<ModId>>>,
    folders: FolderSettings,
}

pub struct Arrived {
    pub ops: Vec<Op>,
    pub progress: Progress,
    pub mods: usize,
    pub capped: bool,
}

#[derive(Default)]
struct State {
    generation: u64,
    cancel: Option<Arc<AtomicBool>>,
    results: Results,
}

#[derive(Default)]
pub struct Find {
    lists: FileLists,
    state: Mutex<State>,
    hit_mods: Mutex<Arc<HashSet<ModId>>>,
}

fn matching(request: &Request) -> Result<Pattern, Error> {
    Pattern::new(&request.text, request.switches)
}

/// The mod a settings file `Mod_<folder>_<Class>.xml` belongs to: the longest folder name
/// that starts what follows `Mod_`.
fn mod_of_settings(name: &str, folders: &HashMap<String, ModId>) -> Option<ModId> {
    let lower = name.to_lowercase();
    let rest = lower.strip_prefix("mod_")?.strip_suffix(".xml")?;
    let ends = rest.match_indices('_').map(|(at, _)| at);
    ends.rev()
        .find_map(|end| folders.get(&rest[..end]))
        .copied()
}

fn targets_of(
    library: &Library,
    active: &HashSet<ModId>,
    request: &Request,
    config: Option<PathBuf>,
) -> Vec<Target<Option<ModId>>> {
    let mod_target =
        |(id, installed): (ModId, &rimstack_core::mods::library::Installed)| Target::Mod {
            key: Some(id),
            folder: installed.path.clone(),
        };
    let ids = library.ids();
    match request.place {
        Place::Active => ids
            .filter(|(id, _)| active.contains(id))
            .map(mod_target)
            .collect(),
        Place::Inactive => ids
            .filter(|(id, _)| !active.contains(id))
            .map(mod_target)
            .collect(),
        Place::Installed => ids.map(mod_target).collect(),
        Place::Local => ids
            .filter(|(_, m)| m.source == Source::Local)
            .map(mod_target)
            .collect(),
        Place::Workshop => ids
            .filter(|(_, m)| m.source == Source::Workshop)
            .map(mod_target)
            .collect(),
        Place::Selected => request
            .selected
            .and_then(|id| Some((id, library.get(id)?)))
            .map(mod_target)
            .into_iter()
            .collect(),
        Place::Settings => {
            let folders: HashMap<String, ModId> =
                ids.map(|(id, m)| (m.folder.to_lowercase(), id)).collect();
            config
                .map(|folder| Target::Settings {
                    folder,
                    key_of: Box::new(move |name| mod_of_settings(name, &folders)),
                })
                .into_iter()
                .collect()
        }
    }
}

impl App {
    fn searches_file(&self) -> PathBuf {
        self.data_dir().join("searches.json")
    }

    /// Nothing is wrong with a box left empty.
    pub fn find_check(&self, text: &str, switches: Switches) -> Result<(), Error> {
        match text.is_empty() {
            true => Ok(()),
            false => Pattern::new(text, switches).map(drop),
        }
    }

    /// Ends the search before, forgets its results, and takes the places to look.
    pub fn find_start(&self, request: &Request) -> Result<Started, Error> {
        let pattern = matching(request)?;
        let loaded = self.library();
        let (library, _) = loaded.as_ref().as_ref().map_err(Clone::clone)?;
        let active = self.active_now(library);
        let config = self.paths().config.as_ref().map(|c| c.join("Config"));
        let targets = targets_of(library, &active, request, config);
        let cancel = Arc::new(AtomicBool::new(false));
        let mut state = self.find.state.lock().unwrap();
        if let Some(older) = state.cancel.replace(cancel.clone()) {
            older.store(true, Ordering::Relaxed);
        }
        state.generation += 1;
        state.results = Results::default();
        Ok(Started {
            generation: state.generation,
            cancel,
            pattern,
            targets,
            folders: request.folders,
        })
    }

    /// Blocks until the search is done or stopped; each batch goes to `on_batch` as found.
    pub fn find_search(
        &self,
        started: &Started,
        on_batch: impl Fn(Batch<Option<ModId>>) + Sync,
    ) -> Summary {
        search(
            &started.targets,
            &started.pattern,
            started.folders,
            &self.find.lists,
            &started.cancel,
            MOST_HITS,
            on_batch,
        )
    }

    pub fn find_stop(&self) {
        if let Some(cancel) = &self.find.state.lock().unwrap().cancel {
            cancel.store(true, Ordering::Relaxed);
        }
    }

    /// `None` for a batch of a search that was replaced.
    pub fn find_add(&self, generation: u64, batch: Batch<Option<ModId>>) -> Option<Arrived> {
        let loaded = self.library();
        let library = loaded.as_ref().as_ref().ok().map(|(library, _)| library);
        let mut state = self.find.state.lock().unwrap();
        if state.generation != generation {
            return None;
        }
        let mut ops = Vec::new();
        for found in batch.found {
            let installed = found.key.zip(library).and_then(|(id, l)| l.get(id));
            let shown = match installed.and_then(|m| found.file.strip_prefix(&m.path).ok()) {
                Some(inside) => inside.to_string_lossy().into_owned(),
                None => found
                    .file
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            };
            let name = || match installed {
                Some(m) => Sentence::Plain(m.display_name().to_string()),
                None => say!("Settings of mods not installed"),
            };
            ops.extend(state.results.add(found, shown, name));
        }
        Some(Arrived {
            ops,
            progress: batch.progress,
            mods: state.results.mods_found(),
            capped: state.results.capped(),
        })
    }

    /// Makes the mods of the results those `in:search` keeps; the sentence says what was found.
    pub fn find_finish(&self, generation: u64, summary: Summary) -> Option<Sentence> {
        let state = self.find.state.lock().unwrap();
        if state.generation != generation {
            return None;
        }
        let results = &state.results;
        let skipped = summary.too_large + summary.binary;
        let mut parts = match summary.matches {
            0 => vec![say!("No matches")],
            n => vec![
                say_n!(n, "{} match", "{} matches"),
                say_n!(results.files_found(), "{} file", "{} files"),
                say_n!(results.mods_found(), "{} mod", "{} mods"),
            ],
        };
        if skipped > 0 {
            parts.push(say_n!(skipped, "{} file skipped", "{} files skipped"));
        }
        if results.capped() {
            parts.push(say!("showing the first {}", MOST_HITS));
        }
        if summary.cancelled {
            parts.push(say!("stopped"));
        }
        let mods = results.mods().clone();
        drop(state);
        self.find_keep_mods(mods);
        Some(Sentence::listed(parts))
    }

    fn find_keep_mods(&self, mods: HashSet<ModId>) {
        let mut kept = self.find.hit_mods.lock().unwrap();
        if **kept != mods {
            *kept = Arc::new(mods);
            drop(kept);
            self.publish(Changes::FIND);
        }
    }

    /// What `in:search` keeps.
    pub fn find_mods(&self) -> Arc<HashSet<ModId>> {
        self.find.hit_mods.lock().unwrap().clone()
    }

    /// Mods with hits so far, for the Mods list; the number of them.
    pub fn find_show_in_mods(&self) -> usize {
        let mods = self.find.state.lock().unwrap().results.mods().clone();
        let count = mods.len();
        self.find_keep_mods(mods);
        count
    }

    pub fn find_results<T>(&self, read: impl FnOnce(&Results) -> T) -> T {
        read(&self.find.state.lock().unwrap().results)
    }

    pub fn find_change<T>(&self, change: impl FnOnce(&mut Results) -> T) -> T {
        change(&mut self.find.state.lock().unwrap().results)
    }

    pub fn find_file(&self, at: RowRef) -> Option<PathBuf> {
        self.find_results(|r| r.locate(at)).map(|l| l.file)
    }

    pub fn find_folder(&self, at: RowRef) -> Option<PathBuf> {
        let file = self.find_file(at)?;
        file.parent().map(PathBuf::from)
    }

    pub fn find_line(&self, at: RowRef) -> Option<String> {
        self.find_results(|r| r.locate(at)).map(|l| l.text)
    }

    /// Reads the file: off the window's thread.
    pub fn find_preview(&self, at: RowRef) -> Option<PreviewShown> {
        let located = self.find_results(|r| r.locate(at))?;
        let path = located.file.to_string_lossy().into_owned();
        Some(match preview(&located.file, located.line) {
            Ok(read) => PreviewShown {
                path,
                line: located.line,
                lines: read
                    .lines
                    .into_iter()
                    .enumerate()
                    .map(|(at, text)| PreviewLine {
                        number: read.first_line + at,
                        marked: read.marked == Some(at),
                        text,
                    })
                    .collect(),
                problem: None,
            },
            Err(problem) => PreviewShown {
                path,
                line: located.line,
                lines: Vec::new(),
                problem: Some(problem.sentence().clone()),
            },
        })
    }

    pub fn find_options(&self, request: &Request) -> Options {
        let choice = |id, label, chosen| Choice {
            id,
            label,
            chosen,
            available: true,
        };
        let (s, f) = (request.switches, request.folders);
        let selected = request.selected.and_then(|id| {
            let loaded = self.library();
            let (library, _) = loaded.as_ref().as_ref().ok()?;
            Some(library.get(id)?.display_name().to_string())
        });
        let places = Place::ALL.map(|place| {
            let mut shown = choice(place.id(), place.label(), place == request.place);
            if place == Place::Selected {
                shown.available = selected.is_some();
                if let Some(name) = &selected {
                    shown.label = say!("Only {}", name);
                }
            }
            shown
        });
        Options {
            switches: vec![
                choice("case", say!("Match case"), s.case),
                choice("word", say!("Whole word"), s.whole_word),
                choice("regex", say!("Regular expression"), s.regex),
            ],
            places: places.into(),
            files: vec![
                choice("xml", say!("XML files"), !f.all_text),
                choice("text", say!("All text files"), f.all_text),
            ],
            leave: vec![
                choice("languages", say!("Languages"), f.languages),
                choice("source", say!("Source"), f.source),
                choice("textures", say!("Textures"), f.textures),
                choice("git", Sentence::Plain(".git".into()), f.git),
            ],
        }
    }

    pub fn find_remember(&self, request: &Request) {
        let file = self.searches_file();
        let kept = Searches::read(&file).with(request.kept());
        if let Err(problem) = kept.write(&file) {
            rimstack_core::problem!("{problem}");
        }
    }

    pub fn find_recent(&self) -> Vec<Recent> {
        let kept = Searches::read(&self.searches_file());
        kept.all()
            .iter()
            .map(|s| Recent {
                text: s.text.clone(),
                place: Place::of(&s.place).unwrap_or(Place::Active).label(),
                case: s.case,
                regex: s.regex,
                word: s.whole_word,
                all_text: s.all_text,
            })
            .collect()
    }

    pub fn find_recall(&self, at: usize, selected: Option<ModId>) -> Option<Request> {
        let kept = Searches::read(&self.searches_file());
        Some(Request::from_kept(kept.all().get(at)?, selected))
    }
}

#[cfg(test)]
mod tests;
