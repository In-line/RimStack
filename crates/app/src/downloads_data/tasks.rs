//! One at a time, in the order asked: two would write the same SteamCMD folder.

use super::{Waiting, qobject};
use crate::changes::Changes;
use crate::qml::values::to_qml;
use crate::workspace::fetch::{Done, SteamAsk};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use rimstack_core::base::sentence::Sentence;
use rimstack_core::mods::library::ModId;
use rimstack_core::say;

impl qobject::DownloadsData {
    fn task(self: Pin<&mut Self>, doing: Sentence, work: impl FnOnce() -> Done + Send + 'static) {
        self.lined_up(doing, None, work);
    }

    fn lined_up(
        mut self: Pin<&mut Self>,
        doing: Sentence,
        updating: Option<i32>,
        work: impl FnOnce() -> Done + Send + 'static,
    ) {
        if self.demo {
            return;
        }
        let waits = Waiting {
            doing: crate::words::put(&doing),
            updating,
            work: Box::new(work),
        };
        match self.task_running {
            true => self.as_mut().rust_mut().get_mut().waiting.push_back(waits),
            false => self.start(waits),
        }
    }

    fn start(mut self: Pin<&mut Self>, waits: Waiting) {
        self.as_mut().set_task_running(true);
        self.as_mut().set_task_note(QString::from(&waits.doing));
        self.as_mut().set_updating(waits.updating.unwrap_or(-1));
        crate::jobs::run(
            "download task",
            self.qt_thread(),
            waits.work,
            |mut me, done| {
                me.as_mut().finish(done);
                if let Some(next) = me.as_mut().rust_mut().get_mut().waiting.pop_front() {
                    me.start(next);
                }
            },
        );
    }

    fn finish(mut self: Pin<&mut Self>, done: Done) {
        let app = self.app();
        if done.wrote {
            app.publish(Changes::MOD_FILES);
        }
        let failed = done.note.is_err();
        let said = match done.note {
            Ok(did) => {
                let said = crate::words::put(&did);
                app.note(did, String::new());
                said
            }
            Err(problem) => crate::words::put(problem.sentence()),
        };
        self.as_mut().set_steamcmd(app.steamcmd().is_some());
        self.as_mut().set_updating(-1);
        if self.cloning >= 0 {
            self.as_mut().set_cloning(-1);
            self.as_mut().show_repositories_again();
        }
        self.as_mut().set_task_running(false);
        self.as_mut().set_task_note(QString::from(&said));
        self.ended(QString::from(&said), failed);
    }

    fn ask_steam(self: Pin<&mut Self>, what: SteamAsk, ids: Vec<String>) {
        if ids.is_empty() {
            return;
        }
        let app = self.app();
        self.task(say!("Asking Steam…"), move || app.ask_steam(what, &ids));
    }

    pub(super) fn update_in_steam(mut self: Pin<&mut Self>) {
        let ids: Vec<String> = self
            .checked
            .items
            .iter()
            .map(|row| row.id.clone())
            .collect();
        self.as_mut()
            .rust_mut()
            .get_mut()
            .asked_of_steam
            .extend(ids.iter().cloned());
        let checked = std::mem::take(&mut self.as_mut().rust_mut().get_mut().checked);
        self.as_mut().show_updates(checked);
        self.ask_steam(SteamAsk::Download, ids);
    }

    pub(super) fn apply(self: Pin<&mut Self>) {
        let ids: Vec<String> = self
            .checked
            .items
            .iter()
            .map(|row| row.id.clone())
            .filter(|id| !self.asked_of_steam.contains(id))
            .collect();
        let checkouts: Vec<ModId> = self
            .checked
            .checkouts
            .iter()
            .filter(|c| c.due() && !c.workshop)
            .map(|c| c.mod_id)
            .collect();
        if ids.is_empty() && checkouts.is_empty() {
            return;
        }
        let app = self.app();
        let any_checkout = !checkouts.is_empty();
        crate::jobs::run(
            "updates applied",
            self.qt_thread(),
            move || {
                let running = any_checkout && app.game_running();
                let asked = !ids.is_empty();
                let took = asked && app.ask_steam(SteamAsk::Download, &ids).note.is_ok();
                (running, asked, took, ids)
            },
            move |mut me, (running, asked, took, ids)| {
                me.as_mut().set_steam_waits(asked && !took);
                if took {
                    me.as_mut().rust_mut().get_mut().asked_of_steam.extend(ids);
                    let checked = std::mem::take(&mut me.as_mut().rust_mut().get_mut().checked);
                    me.as_mut().show_updates(checked);
                }
                if !running {
                    for id in checkouts {
                        me.as_mut().update_from_git(id.to_qml());
                    }
                }
            },
        );
    }

    pub(super) fn subscribe_wanted(self: Pin<&mut Self>) {
        let app = self.app();
        let count = app.listed_ids().len();
        if count == 0 {
            return;
        }
        self.task(say!("Subscribing to {} in Steam…", count), move || {
            app.subscribe_listed()
        });
    }

    pub(super) fn subscribe(self: Pin<&mut Self>, workshop_id: &QString) {
        self.ask_steam(SteamAsk::Subscribe, vec![workshop_id.to_string()]);
    }

    pub(super) fn unsubscribe(self: Pin<&mut Self>, mod_id: i32) {
        let workshop = ModId::from_qml(mod_id).and_then(|id| self.app().workshop_id(id));
        self.ask_steam(SteamAsk::Unsubscribe, workshop.into_iter().collect());
    }

    pub(super) fn download_wanted(self: Pin<&mut Self>) {
        let app = self.app();
        let count = app.listed_ids().len();
        if count == 0 {
            return;
        }
        self.task(say!("Downloading {} with SteamCMD…", count), move || {
            app.download_listed()
        });
    }

    pub(super) fn update_from_git(self: Pin<&mut Self>, mod_id: i32) {
        let Some(id) = ModId::from_qml(mod_id) else {
            return;
        };
        let app = self.app();
        self.lined_up(say!("Updating from Git…"), Some(mod_id), move || {
            app.update_checkout(id)
        });
    }

    fn show_repositories_again(mut self: Pin<&mut Self>) {
        let Ok(queue) = &mut self.as_mut().rust_mut().get_mut().looked_up else {
            return;
        };
        rimstack_core::sources::downloads::recheck(&mut queue.repositories);
        let shown = to_qml(&*queue);
        self.set_queue(shown);
    }

    pub(super) fn clone_repository(mut self: Pin<&mut Self>, at: i32) {
        let link = usize::try_from(at).ok().and_then(|at| {
            let queue = self.looked_up.as_ref().ok()?;
            Some(queue.repositories.get(at)?.link.clone())
        });
        let Some(link) = link else {
            return;
        };
        if !self.demo {
            self.as_mut().set_cloning(at);
        }
        let app = self.app();
        self.task(say!("Cloning {}…", link), move || {
            app.clone_repository(&link)
        });
    }

    pub(super) fn install_steamcmd(self: Pin<&mut Self>) {
        let app = self.app();
        self.task(say!("Installing SteamCMD…"), move || {
            app.install_steamcmd()
        });
    }

    pub(super) fn redownload(self: Pin<&mut Self>, mod_id: i32) {
        let Some(id) = ModId::from_qml(mod_id) else {
            return;
        };
        let app = self.app();
        self.task(say!("Downloading again with SteamCMD…"), move || {
            app.redownload(id)
        });
    }
}
