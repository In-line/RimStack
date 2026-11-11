#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("rimstack-app/src/mod_rows.cxxqt.h");
        type QAbstractListModel = crate::mod_rows::qobject::QAbstractListModel;
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qml_uncreatable]
        #[qproperty(QString, text)]
        #[qproperty(bool, running)]
        #[qproperty(i32, done)]
        #[qproperty(i32, total)]
        #[qproperty(QString, progress)]
        #[qproperty(QString, summary)]
        #[qproperty(QString, problem)]
        #[qproperty(bool, capped)]
        /// Mods with hits.
        #[qproperty(i32, count)]
        /// A mod id, -1 for none.
        #[qproperty(i32, selected_mod, cxx_name = "selectedMod")]
        /// The row chosen, -1 for none; its file is in `preview`.
        #[qproperty(i32, current)]
        #[qproperty(QVariant, recent)]
        #[qproperty(QVariant, preview)]
        #[qproperty(QVariant, options)]
        type FindData = super::FindDataRust;
    }

    impl cxx_qt::Threading for FindData {}

    unsafe extern "C++" {
        include!("cxx-qt-lib/common.h");
        #[cxx_name = "make_unique"]
        #[namespace = "rust::cxxqtlib1"]
        fn make_find_data() -> UniquePtr<FindData>;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &FindData, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &FindData, parent: &QModelIndex) -> i32;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &FindData) -> QHash_i32_QByteArray;
    }

    extern "RustQt" {
        /// Looks as the box and the options say; a newer search ends the one before.
        #[qinvokable]
        fn search(self: Pin<&mut FindData>);

        #[qinvokable]
        fn stop(self: &FindData);

        /// A hit, or a file (its first hit), is chosen and shown in `preview`.
        #[qinvokable]
        fn select(self: Pin<&mut FindData>, row: i32);

        /// The row `by` file or line rows from `row` (a mod's row and a gap are passed over);
        /// `row` itself when there is none.
        #[qinvokable]
        fn step(self: &FindData, row: i32, by: i32) -> i32;

        /// Folds the file of `row` (`how` -1), unfolds it (1) or flips it (0); gives the row to choose then.
        #[qinvokable]
        fn fold(self: Pin<&mut FindData>, row: i32, how: i32) -> i32;

        /// The path to give `Qt.openUrlExternally(Theme.fileUrl(…))`; empty for a mod's row.
        #[qinvokable]
        #[cxx_name = "openFile"]
        fn open_file(self: &FindData, row: i32) -> QString;

        #[qinvokable]
        #[cxx_name = "openFolder"]
        fn open_folder(self: &FindData, row: i32) -> QString;

        #[qinvokable]
        #[cxx_name = "copyPath"]
        fn copy_path(self: &FindData, row: i32) -> QString;

        #[qinvokable]
        #[cxx_name = "copyLine"]
        fn copy_line(self: &FindData, row: i32) -> QString;

        /// Folds or unfolds a mod's or a file's rows.
        #[qinvokable]
        fn toggle(self: Pin<&mut FindData>, row: i32);

        /// Makes `in:search` keep the mods found; gives the filter text to put in the Mods list.
        #[qinvokable]
        #[cxx_name = "showInMods"]
        fn show_in_mods(self: Pin<&mut FindData>) -> QString;

        /// Fills the box and the options as that search had them, and searches.
        #[qinvokable]
        #[cxx_name = "useRecent"]
        fn use_recent(self: Pin<&mut FindData>, at: i32);

        /// Keeps the results with the text in their line, file or mod.
        #[qinvokable]
        #[cxx_name = "filterResults"]
        fn filter_results(self: Pin<&mut FindData>, text: &QString);

        /// A switch: "case", "word", "regex", "languages", "source", "textures" or "git".
        #[qinvokable]
        fn flip(self: Pin<&mut FindData>, id: &QString);

        /// `group` is "where" or "files"; `id` is one of `options.where` or `options.files`.
        #[qinvokable]
        fn choose(self: Pin<&mut FindData>, group: &QString, id: &QString);
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(
            self: Pin<&mut FindData>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QVector_i32,
        );

        #[inherit]
        fn index(self: &FindData, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;
    }

    extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginInsertRows"]
        unsafe fn begin_insert_rows(
            self: Pin<&mut FindData>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        #[inherit]
        #[cxx_name = "endInsertRows"]
        unsafe fn end_insert_rows(self: Pin<&mut FindData>);
        #[inherit]
        #[cxx_name = "beginRemoveRows"]
        unsafe fn begin_remove_rows(
            self: Pin<&mut FindData>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        #[inherit]
        #[cxx_name = "endRemoveRows"]
        unsafe fn end_remove_rows(self: Pin<&mut FindData>);
        #[inherit]
        #[cxx_name = "beginMoveRows"]
        unsafe fn begin_move_rows(
            self: Pin<&mut FindData>,
            source_parent: &QModelIndex,
            source_first: i32,
            source_last: i32,
            destination_parent: &QModelIndex,
            destination_child: i32,
        ) -> bool;
        #[inherit]
        #[cxx_name = "endMoveRows"]
        unsafe fn end_move_rows(self: Pin<&mut FindData>);
        #[inherit]
        #[cxx_name = "beginResetModel"]
        unsafe fn begin_reset_model(self: Pin<&mut FindData>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        unsafe fn end_reset_model(self: Pin<&mut FindData>);
    }
}

use crate::App;
use crate::changes::Changes;
use crate::qml::diff::{Step, steps};
use crate::qml::values::to_qml;
use crate::workspace::find::{Cell, Folded, Op, ROLES, Request, RowRef};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{
    QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant, QVector,
};
use rimstack_core::base::locks::Held;
use rimstack_core::base::sentence::Sentence;
use rimstack_core::mods::find::{Batch, Summary};
use rimstack_core::mods::library::ModId;
use rimstack_core::say;
use std::sync::{Arc, Mutex};

const FIRST_ROLE: i32 = 257;
const IN_SEARCH: &str = "in:search";

pub struct FindDataRust {
    text: QString,
    running: bool,
    done: i32,
    total: i32,
    progress: QString,
    summary: QString,
    problem: QString,
    capped: bool,
    count: i32,
    selected_mod: i32,
    current: i32,
    recent: QVariant,
    preview: QVariant,
    options: QVariant,
    app: crate::hub::Given,
    request: Request,
    generation: u64,
    shown: Vec<RowRef>,
    said: Option<Sentence>,
    working: Option<Sentence>,
    runs: crate::jobs::Lane,
    previews: crate::jobs::Lane,
}

impl Default for FindDataRust {
    fn default() -> Self {
        Self {
            text: QString::default(),
            running: false,
            done: 0,
            total: 0,
            progress: QString::default(),
            summary: QString::default(),
            problem: QString::default(),
            capped: false,
            count: 0,
            selected_mod: -1,
            current: -1,
            recent: QVariant::default(),
            preview: QVariant::default(),
            options: QVariant::default(),
            app: crate::hub::Given::default(),
            request: Request::default(),
            generation: 0,
            shown: Vec::new(),
            said: None,
            working: None,
            runs: crate::jobs::Lane::default(),
            previews: crate::jobs::Lane::default(),
        }
    }
}

fn int(n: impl TryInto<i32>) -> i32 {
    n.try_into().unwrap_or(i32::MAX)
}

fn variant(cell: Cell<'_>) -> QVariant {
    match cell {
        Cell::Flag(b) => QVariant::from(&b),
        Cell::Number(n) => QVariant::from(&n),
        Cell::Text(t) => QVariant::from(&QString::from(t.as_ref())),
        Cell::Said(s) => QVariant::from(&crate::words::said(s)),
        Cell::Parts(parts) => to_qml(&parts),
    }
}

impl qobject::FindData {
    fn app(&self) -> Arc<App> {
        self.app.app()
    }

    pub(crate) fn connect(mut self: Pin<&mut Self>, app: Arc<App>) {
        crate::hub::hear(&app, self.qt_thread(), Self::heard);
        self.as_mut().rust_mut().get_mut().app = crate::hub::Given::of(app.clone());
        self.as_mut().on_text_changed(|me| me.check()).release();
        self.as_mut()
            .on_selected_mod_changed(|me| me.show_options())
            .release();
        self.as_mut().show_options();
        crate::jobs::run(
            "find recent",
            self.qt_thread(),
            move || app.find_recent(),
            |mut me, recent| me.as_mut().set_recent(to_qml(&recent)),
        );
    }

    fn heard(mut self: Pin<&mut Self>, changes: Changes) {
        if !changes.contains(Changes::LANGUAGE) {
            return;
        }
        self.as_mut().show_options();
        self.as_mut().show_texts();
        let recent = self.app().find_recent();
        self.as_mut().set_recent(to_qml(&recent));
        self.changed_all();
    }

    fn changed_all(self: Pin<&mut Self>) {
        let last = int(self.shown.len()) - 1;
        if last < 0 {
            return;
        }
        let root = QModelIndex::default();
        let (top, bottom) = (self.index(0, 0, &root), self.index(last, 0, &root));
        self.data_changed(&top, &bottom, &QVector::default());
    }

    fn show_texts(mut self: Pin<&mut Self>) {
        let said = |s: &Option<Sentence>| s.as_ref().map(crate::words::said).unwrap_or_default();
        let (summary, progress) = (said(&self.said), said(&self.working));
        self.as_mut().set_summary(summary);
        self.set_progress(progress);
    }

    fn request_now(&self) -> Request {
        Request {
            text: self.text.to_string(),
            selected: ModId::from_qml(self.selected_mod),
            ..self.request.clone()
        }
    }

    fn show_options(self: Pin<&mut Self>) {
        let request = self.request_now();
        let options = to_qml(&self.app().find_options(&request));
        self.set_options(options);
    }

    fn check(self: Pin<&mut Self>) {
        let (text, switches) = (self.text.to_string(), self.request.switches);
        let checked = self.app().find_check(&text, switches);
        self.set_problem(crate::words::why_not(&checked));
    }

    fn flip(mut self: Pin<&mut Self>, id: &QString) {
        self.as_mut()
            .rust_mut()
            .get_mut()
            .request
            .flip(&id.to_string());
        self.as_mut().show_options();
        self.check();
    }

    fn choose(mut self: Pin<&mut Self>, group: &QString, id: &QString) {
        let (group, id) = (group.to_string(), id.to_string());
        self.as_mut()
            .rust_mut()
            .get_mut()
            .request
            .choose(&group, &id);
        self.show_options();
    }

    fn search(mut self: Pin<&mut Self>) {
        let app = self.app();
        let request = self.request_now();
        let started = match app.find_start(&request) {
            Ok(started) => started,
            Err(problem) => return self.set_problem(crate::words::said_of(&problem)),
        };
        let generation = started.generation;
        self.as_mut().set_problem(QString::default());
        self.as_mut().renew(Vec::new());
        let me = self.as_mut().rust_mut().get_mut();
        me.generation = generation;
        me.said = None;
        me.working = Some(say!("Searching"));
        self.as_mut().set_preview(QVariant::default());
        self.as_mut().set_capped(false);
        self.as_mut().set_count(0);
        self.as_mut().set_done(0);
        self.as_mut().set_total(0);
        self.as_mut().set_running(true);
        self.as_mut().show_texts();
        let remember = app.clone();
        crate::jobs::run(
            "find remember",
            self.qt_thread(),
            move || {
                remember.find_remember(&request);
                remember.find_recent()
            },
            |mut me, recent| me.as_mut().set_recent(to_qml(&recent)),
        );
        let batches = Mutex::new(self.qt_thread());
        self.runs.run(
            "find",
            self.qt_thread(),
            move |_| {
                app.find_search(&started, |batch| {
                    let _ = batches.held().queue(move |me| me.arrive(generation, batch));
                })
            },
            move |me, summary| me.finished(generation, summary),
        );
    }

    fn arrive(mut self: Pin<&mut Self>, generation: u64, batch: Batch<Option<ModId>>) {
        if generation != self.generation {
            return;
        }
        let Some(arrived) = self.app().find_add(generation, batch) else {
            return;
        };
        for op in arrived.ops {
            self.as_mut().apply(op);
        }
        let (done, total) = (arrived.progress.done, arrived.progress.total);
        self.as_mut().set_done(int(done));
        self.as_mut().set_total(int(total));
        self.as_mut().set_count(int(arrived.mods));
        self.as_mut().set_capped(arrived.capped);
        self.as_mut().rust_mut().get_mut().working =
            Some(say!("Mods searched: {} of {}", done, total));
        self.show_texts();
    }

    fn finished(mut self: Pin<&mut Self>, generation: u64, summary: Summary) {
        if generation != self.generation {
            return;
        }
        let said = self.app().find_finish(generation, summary);
        let me = self.as_mut().rust_mut().get_mut();
        me.said = said;
        me.working = None;
        self.as_mut().set_running(false);
        self.show_texts();
    }

    fn stop(&self) {
        self.app().find_stop();
    }

    fn apply(self: Pin<&mut Self>, op: Op) {
        let root = QModelIndex::default();
        match op {
            Op::Insert { at, rows } => self.insert(at, rows),
            Op::Changed { at } => {
                let index = self.index(int(at), 0, &root);
                self.data_changed(&index, &index, &QVector::default());
            }
        }
    }

    fn insert(mut self: Pin<&mut Self>, at: usize, rows: Vec<RowRef>) {
        if rows.is_empty() || at > self.shown.len() {
            return;
        }
        let coming = rows.len();
        let root = QModelIndex::default();
        // SAFETY: the insert ends right after the rows are in.
        unsafe {
            self.as_mut()
                .begin_insert_rows(&root, int(at), int(at + rows.len() - 1));
            self.as_mut()
                .rust_mut()
                .get_mut()
                .shown
                .splice(at..at, rows);
            self.as_mut().end_insert_rows();
        }
        let current = self.current;
        if current >= int(at) {
            self.set_current(current + int(coming));
        }
    }

    fn remove(mut self: Pin<&mut Self>, first: usize, last: usize) {
        let root = QModelIndex::default();
        // SAFETY: the removal ends right after the rows are out.
        unsafe {
            self.as_mut()
                .begin_remove_rows(&root, int(first), int(last));
            self.as_mut().rust_mut().get_mut().shown.drain(first..=last);
            self.as_mut().end_remove_rows();
        }
        let (current, gone) = (self.current, int(last - first + 1));
        if current >= int(first) {
            let kept = (current > int(last)).then(|| current - gone);
            self.set_current(kept.unwrap_or(-1));
        }
    }

    fn renew(mut self: Pin<&mut Self>, rows: Vec<RowRef>) {
        // SAFETY: the reset ends below, after the rows changed.
        unsafe { self.as_mut().begin_reset_model() };
        self.as_mut().rust_mut().get_mut().shown = rows;
        // SAFETY: ends the reset begun above.
        unsafe { self.as_mut().end_reset_model() };
        self.set_current(-1);
    }

    fn reach(mut self: Pin<&mut Self>, rows: Vec<RowRef>) {
        match steps(&self.shown, &rows) {
            Some(steps) => {
                for step in steps {
                    match step {
                        Step::Remove { first, last } => self.as_mut().remove(first, last),
                        Step::Insert { first, last } => {
                            let coming = rows[first..=last].to_vec();
                            self.as_mut().insert(first, coming);
                        }
                        Step::Move { .. } => return self.renew(rows),
                    }
                }
            }
            None => self.renew(rows),
        }
    }

    fn row(&self, row: i32) -> Option<RowRef> {
        self.shown.get(usize::try_from(row).ok()?).copied()
    }

    fn select(mut self: Pin<&mut Self>, row: i32) {
        let Some(at) = self.row(row).filter(|at| !at.is_mod()) else {
            self.previews.cancel();
            self.as_mut().set_current(-1);
            return self.set_preview(QVariant::default());
        };
        self.as_mut().set_current(row);
        let app = self.app();
        self.previews.run(
            "find preview",
            self.qt_thread(),
            move |_| app.find_preview(at),
            |mut me, shown| me.as_mut().set_preview(to_qml(&shown)),
        );
    }

    fn step(&self, row: i32, by: i32) -> i32 {
        let moved = self
            .app()
            .find_results(|r| r.step(&self.shown, usize::try_from(row).ok(), by));
        moved.map_or(row, int)
    }

    fn fold(self: Pin<&mut Self>, row: i32, how: i32) -> i32 {
        let Some(at) = self.row(row).filter(|at| !at.is_mod()) else {
            return row;
        };
        let file_row = match at.is_file() {
            true => row,
            false => {
                let before = &self.shown[..usize::try_from(row).unwrap_or_default()];
                before
                    .iter()
                    .rposition(|other| other.same_file(at) && other.is_file())
                    .map_or(row, int)
            }
        };
        let folded = self.app().find_results(|r| r.is_folded(at));
        if how != 0 && folded == (how < 0) {
            return row;
        }
        self.toggle(file_row);
        file_row
    }

    fn path_text(
        &self,
        row: i32,
        of: impl FnOnce(&App, RowRef) -> Option<std::path::PathBuf>,
    ) -> QString {
        let path = self.row(row).and_then(|at| of(&self.app(), at));
        QString::from(
            path.map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default()
                .as_str(),
        )
    }

    fn open_file(&self, row: i32) -> QString {
        self.path_text(row, App::find_file)
    }

    fn open_folder(&self, row: i32) -> QString {
        self.path_text(row, App::find_folder)
    }

    fn copy_path(&self, row: i32) -> QString {
        self.path_text(row, App::find_file)
    }

    fn copy_line(&self, row: i32) -> QString {
        let line = self.row(row).and_then(|at| self.app().find_line(at));
        QString::from(line.unwrap_or_default().as_str())
    }

    fn toggle(mut self: Pin<&mut Self>, row: i32) {
        let Some(at) = self.row(row) else {
            return;
        };
        let Some(folded) = self.app().find_change(|results| results.toggle(at)) else {
            return;
        };
        let first = usize::try_from(row).unwrap_or_default();
        match folded {
            Folded::Hid => {
                let inside = |other: &&RowRef| match at.is_mod() {
                    true => other.same_mod(at),
                    false => other.same_file(at),
                };
                let below = self.shown[first + 1..].iter().take_while(inside).count();
                if below > 0 {
                    self.as_mut().remove(first + 1, first + below);
                }
            }
            Folded::Showed(rows) => self.as_mut().insert(first + 1, rows),
        }
        let index = self.index(row, 0, &QModelIndex::default());
        self.data_changed(&index, &index, &QVector::default());
    }

    fn show_in_mods(self: Pin<&mut Self>) -> QString {
        let count = self.app().find_show_in_mods();
        self.set_count(int(count));
        QString::from(IN_SEARCH)
    }

    fn use_recent(mut self: Pin<&mut Self>, at: i32) {
        let selected = ModId::from_qml(self.selected_mod);
        let Some(request) = usize::try_from(at)
            .ok()
            .and_then(|at| self.app().find_recall(at, selected))
        else {
            return;
        };
        let text = QString::from(request.text.as_str());
        self.as_mut().rust_mut().get_mut().request = request;
        self.as_mut().set_text(text);
        self.as_mut().show_options();
        self.search();
    }

    fn filter_results(self: Pin<&mut Self>, text: &QString) {
        let text = text.to_string();
        let rows = self.app().find_change(|results| results.set_filter(&text));
        self.reach(rows);
    }

    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let field = usize::try_from(role - FIRST_ROLE).ok();
        let at = self.row(index.row()).zip(field);
        let cell =
            at.and_then(|(at, field)| self.app().find_results(|r| r.cell(at, field).map(variant)));
        cell.unwrap_or_default()
    }

    fn row_count(&self, _parent: &QModelIndex) -> i32 {
        int(self.shown.len())
    }

    fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut names = QHash::<QHashPair_i32_QByteArray>::default();
        for (at, name) in ROLES.iter().enumerate() {
            names.insert(FIRST_ROLE + int(at), QByteArray::from(*name));
        }
        names
    }
}
