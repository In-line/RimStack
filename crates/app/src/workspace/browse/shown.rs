use rimstack_core::base::{elapsed::ago, sentence::Sentence, sizes};
use rimstack_core::rimworld::workshop::{Details, Votes};
use rimstack_core::rimworld::workshopindex::Entry;
use rimstack_core::sources::browse::Browse;
use rimstack_core::{say, say_n};
use serde::Serialize;
use std::collections::HashSet;

/// Asked once per answer.
pub struct Marks {
    pub installed: HashSet<String>,
    pub listed: HashSet<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub versions: Vec<String>,
    pub installed: bool,
    pub on_list: bool,
    pub badge: Option<Sentence>,
    pub can_add: bool,
}

impl Marks {
    pub fn row(&self, entry: &Entry) -> Row {
        let installed = self.installed.contains(&entry.id);
        let on_list = self.listed.contains(&entry.id);
        Row {
            id: entry.id.clone(),
            title: entry.title.clone(),
            author: entry.author.clone(),
            versions: entry
                .tags
                .iter()
                .filter(|tag| tag.starts_with(|c: char| c.is_ascii_digit()))
                .cloned()
                .collect(),
            installed,
            on_list,
            badge: match (installed, on_list) {
                (true, _) => Some(say!("Installed")),
                (false, true) => Some(say!("On the list")),
                _ => None,
            },
            can_add: !installed && !on_list,
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Results {
    pub rows: Vec<Row>,
    pub total: usize,
    pub more: bool,
    /// Said once, when the search went to the kept list because Steam would not answer.
    pub notice: Option<Sentence>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Dlc {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Shown {
    #[serde(flatten)]
    pub row: Row,
    /// False when Steam could not be asked: only what the database holds is filled.
    pub loaded: bool,
    pub problem: Option<Sentence>,
    /// Qt's rich text.
    pub description: String,
    pub size: u64,
    pub size_said: Option<Sentence>,
    pub updated: u64,
    pub updated_said: Option<Sentence>,
    pub subscribers: u64,
    pub favorites: u64,
    pub banned: bool,
    pub preview_url: String,
    /// Only with the key.
    pub votes_said: Option<Sentence>,
    /// Only with the key; fetched into files by the bridge.
    #[serde(skip)]
    pub screenshot_urls: Vec<String>,
    pub requires: Vec<Row>,
    pub unknown_requirements: Vec<String>,
    pub unknown_said: Option<Sentence>,
    pub dlc: Vec<Dlc>,
}

fn votes_said(votes: Votes) -> Option<Sentence> {
    let total = usize::try_from(votes.up + votes.down).unwrap_or(usize::MAX);
    let liked = (votes.score * 100.0).round() as u64;
    (total > 0).then(|| say_n!(total, "{} vote, {}% up", "{} votes, {}% up", liked.min(100)))
}

impl Shown {
    pub fn of(
        row: Row,
        entry: &Entry,
        source: &dyn Browse,
        details: Option<&Details>,
        problem: Option<Sentence>,
        requires: Vec<Row>,
        now: u64,
    ) -> Self {
        let known: HashSet<&str> = requires.iter().map(|r| r.id.as_str()).collect();
        let unknown_requirements: Vec<String> = entry
            .requires
            .iter()
            .filter(|id| !known.contains(id.as_str()))
            .cloned()
            .collect();
        let unknown_said = (!unknown_requirements.is_empty()).then(|| {
            rimstack_core::say_n!(
                unknown_requirements.len(),
                "{} requirement is not in the database, so it is not added",
                "{} requirements are not in the database, so they are not added",
                unknown_requirements.len()
            )
        });
        let size = details.map_or(0, |d| d.size);
        let updated = details.map_or(0, |d| d.updated);
        Shown {
            row,
            loaded: details.is_some(),
            problem,
            description: details.map_or_else(String::new, |d| {
                rimstack_core::base::markup::rich(&d.description)
            }),
            size,
            size_said: (size > 0).then(|| sizes::of_bytes(size)),
            updated,
            updated_said: (updated > 0)
                .then(|| say!("Updated {}", ago(now.saturating_sub(updated)).inside())),
            subscribers: details.map_or(0, |d| d.subscriptions),
            favorites: details.map_or(0, |d| d.favorited),
            banned: details.is_some_and(|d| d.banned),
            preview_url: details.map_or_else(String::new, |d| d.preview_url.clone()),
            votes_said: details.and_then(|d| d.votes).and_then(votes_said),
            screenshot_urls: details.map_or_else(Vec::new, |d| d.screenshots.clone()),
            requires,
            unknown_requirements,
            unknown_said,
            dlc: entry
                .dlc
                .iter()
                .map(|id| Dlc {
                    id: id.clone(),
                    name: source.app_name(id),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn votes_are_said_with_their_count_and_the_share_that_liked() {
        let said = |up, down, score| votes_said(Votes { up, down, score }).map(|s| s.to_string());
        assert_eq!(said(90, 10, 0.9).as_deref(), Some("100 votes, 90% up"));
        assert_eq!(said(1, 0, 1.0).as_deref(), Some("1 vote, 100% up"));
        assert_eq!(said(0, 0, 0.0), None);
    }
}
