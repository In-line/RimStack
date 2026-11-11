//! Browse: results as a model, the details of one, and the download list; the window never
//! waits for the index or for Steam.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!("rimstack-app/src/workshop_rows.cxxqt.h");
        type WorkshopRows = crate::workshop_rows::qobject::WorkshopRows;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_uncreatable]
        #[qproperty(*mut WorkshopRows, rows, READ, CONSTANT)]
        /// All the results there are, of which `rows` holds some.
        #[qproperty(i32, total)]
        #[qproperty(bool, more)]
        #[qproperty(bool, searching)]
        /// Empty for none.
        #[qproperty(QString, problem)]
        /// False until the Workshop database is downloaded.
        #[qproperty(bool, ready)]
        /// A game version tag such as `1.6`, empty for any.
        #[qproperty(QString, version)]
        /// The versions to filter by, newest first.
        #[qproperty(QVariant, versions)]
        /// `any`, `mod`, `translation` or `scenario`.
        #[qproperty(QString, kind)]
        /// Whether only mods needing no expansion the game lacks are found; set before `search`.
        #[qproperty(bool, owned)]
        /// Searches go through Steam with the player's key, which offers `sorts` and collections.
        #[qproperty(bool, key_mode, cxx_name = "keyMode")]
        /// `[{ id, label }]`, empty without the key.
        #[qproperty(QVariant, sorts)]
        /// A sort's `id`; set before `search`. Relevance without the key.
        #[qproperty(QString, sort)]
        /// Whether collections are searched instead of mods; set before `search`.
        #[qproperty(bool, collections)]
        /// Said once when a search went to the kept list because Steam would not answer; empty
        /// for none, cleared by the next `search`.
        #[qproperty(QString, notice)]
        /// `{ active, which, id, title }`; `which` is `requires`, `neededBy`, `sameAuthor` or
        /// `collection` while `active`.
        #[qproperty(QVariant, related)]
        /// The selected mod's details, or null.
        #[qproperty(QVariant, details)]
        #[qproperty(bool, details_busy, cxx_name = "detailsBusy")]
        /// `{ rows, count, sizeSaid, failures, action }`.
        #[qproperty(QVariant, list)]
        type WorkshopData = super::WorkshopDataRust;
    }

    impl cxx_qt::Threading for WorkshopData {}

    extern "RustQt" {
        /// Tone is "ok" or "err".
        #[qsignal]
        fn told(self: Pin<&mut WorkshopData>, said: QString, tone: QString);

        /// `version`, `kind` and `sort` as the properties are; the first page of the answer.
        #[qinvokable]
        fn search(
            self: Pin<&mut WorkshopData>,
            text: &QString,
            version: &QString,
            kind: &QString,
            sort: &QString,
        );

        /// Searches collections instead of mods, or mods again; the last search is asked again.
        #[qinvokable]
        #[cxx_name = "showCollections"]
        fn show_collections(self: Pin<&mut WorkshopData>, on: bool);

        #[qinvokable]
        #[cxx_name = "showMore"]
        fn show_more(self: Pin<&mut WorkshopData>);

        #[qinvokable]
        #[cxx_name = "showRelated"]
        fn show_related(self: Pin<&mut WorkshopData>, id: &QString, which: &QString);

        #[qinvokable]
        fn back(self: Pin<&mut WorkshopData>);

        /// An empty id closes the details.
        #[qinvokable]
        fn select(self: Pin<&mut WorkshopData>, id: &QString);

        /// An item link selects that item and gives back nothing; another web link is given back
        /// to be opened; anything else gives back nothing.
        #[qinvokable]
        fn followed(self: Pin<&mut WorkshopData>, link: &QString) -> QString;

        #[qinvokable]
        fn add(self: Pin<&mut WorkshopData>, id: &QString, with_requirements: bool);

        #[qinvokable]
        #[cxx_name = "addMissing"]
        fn add_missing(self: Pin<&mut WorkshopData>, workshop_id: &QString);

        #[qinvokable]
        fn remove(self: &WorkshopData, id: &QString);

        /// Forgets the failures on the list.
        #[qinvokable]
        fn retry(self: &WorkshopData);

        #[qinvokable]
        fn empty(self: &WorkshopData);
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/common.h");
        #[cxx_name = "make_unique"]
        #[namespace = "rust::cxxqtlib1"]
        fn make_workshop_data() -> UniquePtr<WorkshopData>;
    }
}

pub(crate) mod view;

use crate::App;
use crate::changes::Changes;
use crate::qml::values::to_qml;
use crate::workshop_rows::Item;
use crate::workspace::browse::{
    Asked, Followed, Results, Row, Shown, SortOffer, chosen, follow, sort_of,
};
use crate::workspace::download_list::ListShown;
use core::pin::Pin;
use cxx::UniquePtr;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QString, QVariant};
use qobject::WorkshopRows;
use rimstack_core::base::error::Error;
use rimstack_core::base::locks::Held;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use view::{DetailsShown, RelatedShown, View, kind_of, related_of};

const PAGE: usize = 30;
const SCREENSHOTS: usize = 8;

pub struct WorkshopDataRust {
    rows: *mut WorkshopRows,
    owned_rows: UniquePtr<WorkshopRows>,
    total: i32,
    more: bool,
    searching: bool,
    problem: QString,
    ready: bool,
    version: QString,
    versions: QVariant,
    kind: QString,
    owned: bool,
    key_mode: bool,
    sorts: QVariant,
    offered: Vec<SortOffer>,
    sort: QString,
    collections: bool,
    notice: QString,
    related: QVariant,
    details: QVariant,
    details_busy: bool,
    list: QVariant,
    app: crate::hub::Given,
    demo: bool,
    asked: Option<Asked>,
    in_related: Option<(crate::workspace::browse::Related, String)>,
    search_loaded: usize,
    current: Vec<Row>,
    previews: HashMap<String, String>,
    settled: HashSet<String>,
    selected: String,
    shown: Option<Shown>,
    /// The preview's address and the file it was put in, empty until then.
    detail_preview: (String, String),
    /// The screenshots' addresses and the files they were put in, empty until then.
    detail_shots: (Vec<String>, Vec<String>),
    seeking: crate::jobs::Lane,
    previewing: crate::jobs::Lane,
    opening: crate::jobs::Lane,
    detail_previewing: crate::jobs::Lane,
    shooting: crate::jobs::Lane,
    listing: crate::jobs::Lane,
    defaulting: crate::jobs::Lane,
}

impl Default for WorkshopDataRust {
    fn default() -> Self {
        Self {
            rows: std::ptr::null_mut(),
            owned_rows: UniquePtr::null(),
            total: 0,
            more: false,
            searching: false,
            problem: QString::default(),
            ready: false,
            version: QString::default(),
            versions: QVariant::default(),
            kind: QString::from("mod"),
            owned: false,
            key_mode: false,
            sorts: QVariant::default(),
            offered: Vec::new(),
            sort: QString::from("relevance"),
            collections: false,
            notice: QString::default(),
            related: QVariant::default(),
            details: QVariant::default(),
            details_busy: false,
            list: QVariant::default(),
            app: crate::hub::Given::default(),
            demo: false,
            asked: None,
            in_related: None,
            search_loaded: 0,
            current: Vec::new(),
            previews: HashMap::new(),
            settled: HashSet::new(),
            selected: String::new(),
            shown: None,
            detail_preview: (String::new(), String::new()),
            detail_shots: (Vec::new(), Vec::new()),
            seeking: crate::jobs::Lane::default(),
            previewing: crate::jobs::Lane::default(),
            opening: crate::jobs::Lane::default(),
            detail_previewing: crate::jobs::Lane::default(),
            shooting: crate::jobs::Lane::default(),
            listing: crate::jobs::Lane::default(),
            defaulting: crate::jobs::Lane::default(),
        }
    }
}

type Found = Result<(Results, String), Error>;

impl qobject::WorkshopData {
    fn app(&self) -> Arc<App> {
        self.app.app()
    }

    fn model(self: Pin<&mut Self>) -> Option<Pin<&mut WorkshopRows>> {
        self.rust_mut().get_mut().owned_rows.as_mut()
    }

    pub(crate) fn connect(mut self: Pin<&mut Self>, app: Arc<App>, demo: bool) {
        let mut rows = crate::workshop_rows::qobject::make_workshop_rows();
        {
            let me = self.as_mut().rust_mut().get_mut();
            me.rows = crate::hub::address(&mut rows);
            me.owned_rows = rows;
            me.app = crate::hub::Given::of(app.clone());
            me.demo = demo;
        }
        self.as_mut().set_list(to_qml(&ListShown::empty()));
        self.as_mut().set_related(to_qml(&RelatedShown::none()));
        if demo {
            return;
        }
        crate::hub::hear(&app, self.qt_thread(), Self::heard);
        self.heard(Changes::COMMUNITY | Changes::DOWNLOAD_LIST);
    }

    fn heard(mut self: Pin<&mut Self>, changes: Changes) {
        if changes.contains(Changes::COMMUNITY | Changes::SOURCE | Changes::KEY) {
            self.as_mut().read_defaults();
        }
        if changes.contains(Changes::KEY) {
            self.as_mut().note_refusal();
        }
        if changes.contains(Changes::LANGUAGE)
            && let Some(model) = self.as_mut().model()
        {
            model.reworded();
        }
        if changes.contains(
            Changes::DOWNLOAD_LIST
                | Changes::LIBRARY
                | Changes::MOD_FILES
                | Changes::COMMUNITY
                | Changes::SOURCE,
        ) {
            self.as_mut().reload();
        }
        if changes.contains(
            Changes::DOWNLOAD_LIST
                | Changes::LIBRARY
                | Changes::MOD_FILES
                | Changes::COMMUNITY
                | Changes::SETTINGS
                | Changes::SOURCE
                | Changes::LANGUAGE,
        ) {
            self.as_mut().read_list();
            self.reopen_details();
        }
    }

    fn read_defaults(self: Pin<&mut Self>) {
        let app = self.app();
        self.defaulting.run(
            "workshop defaults",
            self.qt_thread(),
            move |_| app.browse_defaults(),
            |mut me, defaults| {
                me.as_mut().set_ready(defaults.ready);
                me.as_mut().set_versions(to_qml(&defaults.versions));
                me.as_mut().rust_mut().get_mut().offered = defaults.sorts;
                me.as_mut().show_sorts();
                me.as_mut().set_key_mode(defaults.key_mode);
                if !defaults.key_mode {
                    me.as_mut().no_key();
                }
                if me.version.is_empty()
                    && let Some(version) = defaults.version
                {
                    me.set_version(QString::from(&version));
                }
            },
        );
    }

    /// A refusal noticed by a call that is not a search is said here too.
    fn note_refusal(self: Pin<&mut Self>) {
        let app = self.app();
        crate::jobs::run(
            "steam key state for browse",
            self.qt_thread(),
            move || app.key_state(),
            |mut me, state| {
                if let Some(note) = state.note.filter(|_| state.refused) {
                    me.as_mut().set_notice(crate::words::said(&note));
                }
            },
        );
    }

    fn show_sorts(mut self: Pin<&mut Self>) {
        let marked = chosen(&self.offered, &self.sort.to_string());
        self.as_mut().set_sorts(to_qml(&marked));
    }

    /// Sorts and collections are Steam's with the key; the search stands as the cache answers it.
    fn no_key(mut self: Pin<&mut Self>) {
        self.as_mut().set_sort(QString::from("relevance"));
        self.as_mut().show_sorts();
        self.as_mut().set_collections(false);
        if let Some(asked) = self.as_mut().rust_mut().get_mut().asked.as_mut() {
            asked.sort = sort_of("relevance");
            asked.collections = false;
        }
    }

    fn read_list(self: Pin<&mut Self>) {
        let app = self.app();
        self.listing.run(
            "download list",
            self.qt_thread(),
            move |_| app.list_shown(),
            |mut me, shown| me.as_mut().set_list(to_qml(&shown)),
        );
    }

    fn current_view(&self) -> Option<View> {
        match &self.in_related {
            Some((which, id)) => Some(View::Related(*which, id.clone())),
            None => self.asked.clone().map(View::Search),
        }
    }

    /// Shows the same results again, for badges that changed.
    fn reload(self: Pin<&mut Self>) {
        let Some(view) = self.current_view() else {
            return;
        };
        let count = self.current.len().max(PAGE);
        self.load(view, 0, count, false);
    }

    fn load(mut self: Pin<&mut Self>, view: View, offset: usize, count: usize, append: bool) {
        self.as_mut().set_searching(true);
        let app = self.app();
        self.seeking.run(
            "workshop results",
            self.qt_thread(),
            move |_| -> Found {
                match view {
                    View::Search(asked) => {
                        let found = app.browse_search(&asked, offset, count);
                        found.map(|results| (results, String::new()))
                    }
                    View::Related(which, id) => {
                        let found = app.browse_related(&id, which);
                        found.map(|results| (results, app.browse_title(&id)))
                    }
                }
            },
            move |me, found| me.arrived(found, append),
        );
    }

    fn arrived(mut self: Pin<&mut Self>, found: Found, append: bool) {
        self.as_mut().set_searching(false);
        match found {
            Ok((results, title)) => {
                self.as_mut().set_problem(QString::default());
                if let Some(notice) = &results.notice {
                    self.as_mut().set_notice(crate::words::said(notice));
                    self.as_mut().read_defaults();
                }
                let (total, more) = (results.total, results.more);
                let me = self.as_mut().rust_mut().get_mut();
                if append {
                    let present: HashSet<String> =
                        me.current.iter().map(|r| r.id.clone()).collect();
                    let fresh = results
                        .rows
                        .into_iter()
                        .filter(|r| !present.contains(&r.id));
                    me.current.extend(fresh);
                } else {
                    me.current = results.rows;
                }
                if let Some((which, id)) = &me.in_related {
                    let related = to_qml(&RelatedShown::of(*which, id, &title));
                    self.as_mut().set_related(related);
                }
                self.as_mut()
                    .set_total(i32::try_from(total).unwrap_or(i32::MAX));
                self.as_mut().set_more(more);
            }
            Err(problem) => {
                self.as_mut().rust_mut().get_mut().current.clear();
                self.as_mut().set_problem(crate::words::said_of(&problem));
                self.as_mut().set_total(0);
                self.as_mut().set_more(false);
            }
        }
        self.as_mut().show_rows();
        self.fetch_previews();
    }

    fn show_rows(mut self: Pin<&mut Self>) {
        let items: Vec<Item> = self
            .current
            .iter()
            .map(|row| Item {
                row: row.clone(),
                preview: self.previews.get(&row.id).cloned().unwrap_or_default(),
            })
            .collect();
        if let Some(model) = self.as_mut().model() {
            model.show(items);
        }
    }

    fn fetch_previews(self: Pin<&mut Self>) {
        let ids: Vec<String> = self
            .current
            .iter()
            .filter(|row| !self.settled.contains(&row.id))
            .map(|row| row.id.clone())
            .collect();
        if ids.is_empty() {
            return;
        }
        let app = self.app();
        let landing = Mutex::new(self.qt_thread());
        self.previewing.run(
            "workshop previews",
            self.qt_thread(),
            move |turn| {
                app.browse_previews(&ids, &|id, path| {
                    if !turn.current() {
                        return false;
                    }
                    let id = id.to_string();
                    let path = path.map(|path| path.to_string_lossy().into_owned());
                    let told = landing.held().queue(move |me| me.preview_came(&id, path));
                    told.is_ok()
                });
            },
            |_, ()| {},
        );
    }

    fn preview_came(mut self: Pin<&mut Self>, id: &str, path: Option<String>) {
        let me = self.as_mut().rust_mut().get_mut();
        me.settled.insert(id.to_string());
        let Some(path) = path else {
            return;
        };
        me.previews.insert(id.to_string(), path.clone());
        if let Some(model) = self.model() {
            model.set_preview(id, &path);
        }
    }

    fn search(
        mut self: Pin<&mut Self>,
        text: &QString,
        version: &QString,
        kind: &QString,
        sort: &QString,
    ) {
        self.as_mut().set_version(version.clone());
        self.as_mut().set_kind(kind.clone());
        self.as_mut().set_sort(sort.clone());
        self.as_mut().show_sorts();
        self.as_mut().set_notice(QString::default());
        let version = version.to_string();
        let asked = Asked {
            text: text.to_string(),
            version: Some(version).filter(|v| !v.is_empty()),
            kind: kind_of(&kind.to_string()),
            owned: self.owned,
            sort: sort_of(&sort.to_string()),
            collections: self.collections,
        };
        let me = self.as_mut().rust_mut().get_mut();
        me.asked = Some(asked.clone());
        me.in_related = None;
        me.search_loaded = 0;
        self.as_mut().set_related(to_qml(&RelatedShown::none()));
        if self.app().browse_opened() {
            self.as_mut().read_defaults();
        }
        self.load(View::Search(asked), 0, PAGE, false);
    }

    fn show_collections(mut self: Pin<&mut Self>, on: bool) {
        self.as_mut().set_collections(on);
        let Some(mut asked) = self.asked.clone() else {
            return;
        };
        asked.collections = on;
        let me = self.as_mut().rust_mut().get_mut();
        me.asked = Some(asked.clone());
        me.in_related = None;
        me.search_loaded = 0;
        self.as_mut().set_related(to_qml(&RelatedShown::none()));
        self.load(View::Search(asked), 0, PAGE, false);
    }

    fn show_more(self: Pin<&mut Self>) {
        let Some(asked) = self
            .asked
            .clone()
            .filter(|_| self.more && self.in_related.is_none())
        else {
            return;
        };
        let offset = self.current.len();
        self.load(View::Search(asked), offset, PAGE, true);
    }

    fn show_related(mut self: Pin<&mut Self>, id: &QString, which: &QString) {
        let Some(which) = related_of(&which.to_string()) else {
            return;
        };
        let id = id.to_string();
        let me = self.as_mut().rust_mut().get_mut();
        if me.in_related.is_none() {
            me.search_loaded = me.current.len();
        }
        me.in_related = Some((which, id.clone()));
        self.as_mut()
            .set_related(to_qml(&RelatedShown::of(which, &id, "")));
        self.load(View::Related(which, id), 0, PAGE, false);
    }

    fn back(mut self: Pin<&mut Self>) {
        if self.in_related.is_none() {
            return;
        }
        let version = self.version.to_string();
        let kind = self.kind.to_string();
        let owned = self.owned;
        let me = self.as_mut().rust_mut().get_mut();
        me.in_related = None;
        let count = me.search_loaded.max(PAGE);
        let asked = me.asked.get_or_insert_with(|| Asked {
            version: Some(version).filter(|v| !v.is_empty()),
            kind: kind_of(&kind),
            owned,
            ..Asked::default()
        });
        let view = View::Search(asked.clone());
        self.as_mut().set_related(to_qml(&RelatedShown::none()));
        self.load(view, 0, count, false);
    }

    fn select(mut self: Pin<&mut Self>, id: &QString) {
        let id = id.to_string();
        let me = self.as_mut().rust_mut().get_mut();
        me.selected = id.clone();
        if id.is_empty() {
            me.shown = None;
            self.opening.cancel();
            self.detail_previewing.cancel();
            self.as_mut().set_details_busy(false);
            self.set_details(QVariant::default());
            return;
        }
        self.as_mut().set_details_busy(true);
        self.open_details(id);
    }

    fn followed(mut self: Pin<&mut Self>, link: &QString) -> QString {
        match follow(&link.to_string()) {
            Followed::Item(id) => {
                self.as_mut().select(&QString::from(id));
                QString::default()
            }
            Followed::Outside(url) => QString::from(url),
            Followed::Nothing => QString::default(),
        }
    }

    fn reopen_details(self: Pin<&mut Self>) {
        let id = self.selected.clone();
        if !id.is_empty() {
            self.open_details(id);
        }
    }

    fn open_details(self: Pin<&mut Self>, id: String) {
        let app = self.app();
        self.opening.run(
            "workshop details",
            self.qt_thread(),
            move |_| app.browse_details(&id),
            |mut me, shown| {
                me.as_mut().set_details_busy(false);
                me.show_details(shown);
            },
        );
    }

    fn show_details(mut self: Pin<&mut Self>, shown: Option<Shown>) {
        let url = shown.as_ref().map(|s| s.preview_url.clone());
        let shots = shown.as_ref().map(|s| s.screenshot_urls.clone());
        let me = self.as_mut().rust_mut().get_mut();
        me.shown = shown;
        if let Some(url) = url.filter(|url| *url != me.detail_preview.0) {
            me.detail_preview = (url.clone(), String::new());
            self.as_mut().fetch_detail_preview(url);
        }
        if let Some(urls) = shots.filter(|urls| *urls != self.detail_shots.0) {
            self.as_mut().rust_mut().get_mut().detail_shots = (urls.clone(), Vec::new());
            self.as_mut().fetch_screenshots(urls);
        }
        self.render_details();
    }

    fn fetch_detail_preview(self: Pin<&mut Self>, url: String) {
        let app = self.app();
        let asked = url.clone();
        self.detail_previewing.run(
            "workshop preview",
            self.qt_thread(),
            move |_| app.browse_preview(&asked),
            move |mut me, found| match found {
                Ok(Some(path)) if me.detail_preview.0 == url => {
                    me.as_mut().rust_mut().get_mut().detail_preview.1 =
                        path.to_string_lossy().into_owned();
                    me.render_details();
                }
                Ok(_) => {}
                Err(problem) => rimstack_core::problem!("{problem}"),
            },
        );
    }

    fn fetch_screenshots(self: Pin<&mut Self>, urls: Vec<String>) {
        if urls.is_empty() {
            self.shooting.cancel();
            return;
        }
        let app = self.app();
        let asked = urls.clone();
        self.shooting.run(
            "workshop screenshots",
            self.qt_thread(),
            move |turn| {
                let mut paths = Vec::new();
                for url in asked.iter().take(SCREENSHOTS) {
                    if !turn.current() {
                        break;
                    }
                    match app.browse_preview(url) {
                        Ok(found) => paths.extend(found),
                        Err(problem) => rimstack_core::problem!("{problem}"),
                    }
                }
                paths
            },
            move |mut me, paths| {
                if me.detail_shots.0 == urls {
                    me.as_mut().rust_mut().get_mut().detail_shots.1 = paths
                        .iter()
                        .map(|path| path.to_string_lossy().into_owned())
                        .collect();
                    me.render_details();
                }
            },
        );
    }

    fn render_details(mut self: Pin<&mut Self>) {
        let value = match &self.shown {
            Some(shown) => {
                let (url, path) = &self.detail_preview;
                let preview_path = if *url == shown.preview_url { path } else { "" };
                let (urls, files) = &self.detail_shots;
                let screenshots = if *urls == shown.screenshot_urls {
                    files.as_slice()
                } else {
                    &[]
                };
                to_qml(&DetailsShown {
                    shown,
                    preview_path,
                    screenshots,
                })
            }
            None => QVariant::default(),
        };
        self.as_mut().set_details(value);
    }

    fn add(self: Pin<&mut Self>, id: &QString, with_requirements: bool) {
        let (app, id) = (self.app(), id.to_string());
        crate::jobs::run(
            "add to the download list",
            self.qt_thread(),
            move || app.add_to_list(&id, with_requirements).note(),
            |me, note| me.tell(note),
        );
    }

    fn add_missing(self: Pin<&mut Self>, workshop_id: &QString) {
        let (app, id) = (self.app(), workshop_id.to_string());
        crate::jobs::run(
            "add a missing mod to the download list",
            self.qt_thread(),
            move || app.add_missing_to_list(&id).note(),
            |me, note| me.tell(note),
        );
    }

    fn tell(self: Pin<&mut Self>, note: Option<(rimstack_core::base::sentence::Sentence, bool)>) {
        if let Some((said, failed)) = note {
            let tone = if failed { "err" } else { "ok" };
            self.told(crate::words::said(&said), QString::from(tone));
        }
    }

    fn remove(&self, id: &QString) {
        self.app().remove_from_list(&id.to_string());
    }

    fn retry(&self) {
        self.app().clear_list_failures();
    }

    fn empty(&self) {
        self.app().empty_list();
    }
}
