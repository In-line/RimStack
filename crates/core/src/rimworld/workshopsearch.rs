use super::workshop::ids_in;
use super::workshopindex::Index;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Kind {
    #[default]
    Any,
    Mod,
    Translation,
    Scenario,
}

impl Kind {
    fn tag(self) -> Option<&'static str> {
        match self {
            Kind::Any => None,
            Kind::Mod => Some("mod"),
            Kind::Translation => Some("translation"),
            Kind::Scenario => Some("scenario"),
        }
    }
}

impl Kind {
    /// The tag as Steam writes it.
    pub fn steam_tag(self) -> Option<&'static str> {
        match self {
            Kind::Any => None,
            Kind::Mod => Some("Mod"),
            Kind::Translation => Some("Translation"),
            Kind::Scenario => Some("Scenario"),
        }
    }
}

/// Beside the search text. They do not apply to a search for given Workshop ids.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filters {
    /// A version tag as Steam writes it, such as `1.6`.
    pub version: Option<String>,
    pub kind: Kind,
    /// App ids of the DLC the player has: a mod needing another is left out. `None` leaves none out.
    pub owned_dlc: Option<Vec<String>>,
}

/// Words and values are lowercase.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query {
    pub words: Vec<String>,
    /// `w:<id>` and pasted links: when there are any, only these mods are found.
    pub ids: Vec<String>,
    pub tags: Vec<String>,
    pub dlc: Vec<String>,
    pub requires: Vec<String>,
    pub authors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub ids: Vec<String>,
    pub total: usize,
}

fn pieces(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let (mut current, mut quoted) = (String::new(), false);
    for c in text.chars() {
        match c {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => found.push(std::mem::take(&mut current)),
            c => current.push(c),
        }
    }
    found.push(current);
    found.retain(|piece| !piece.is_empty());
    found
}

impl Query {
    pub fn parse(text: &str) -> Query {
        let mut query = Query::default();
        for piece in pieces(text) {
            let keyed = piece.split_once(':').filter(|(_, v)| !v.is_empty());
            let digits = |v: &str| v.chars().all(|c| c.is_ascii_digit());
            match keyed.map(|(k, v)| (k.to_lowercase(), v)) {
                Some((k, v)) if k == "t" => query.tags.push(v.to_lowercase()),
                Some((k, v)) if k == "a" => query.authors.push(v.to_lowercase()),
                Some((k, v)) if k == "w" && digits(v) => query.ids.push(v.into()),
                Some((k, v)) if k == "d" && digits(v) => query.dlc.push(v.into()),
                Some((k, v)) if k == "m" && digits(v) => query.requires.push(v.into()),
                _ => {
                    let links = ids_in(&piece);
                    if links.is_empty() {
                        query.words.push(piece.to_lowercase());
                    }
                    for id in links {
                        if !query.ids.contains(&id) {
                            query.ids.push(id);
                        }
                    }
                }
            }
        }
        query
    }
}

impl Index {
    /// Best first: the title is the words, starts with them, has each, or only author or package id do.
    /// With no words, the newest first: ids grow with time and the index keeps them in order.
    pub fn search(&self, query: &Query, filters: &Filters, offset: usize, count: usize) -> Page {
        let phrase = query.words.join(" ");
        let numbers = |texts: &[String]| -> Option<Vec<u64>> {
            texts.iter().map(|t| t.parse().ok()).collect()
        };
        let (Some(given), Some(dlc), Some(requires)) = (
            numbers(&query.ids),
            numbers(&query.dlc),
            numbers(&query.requires),
        ) else {
            return Page {
                ids: Vec::new(),
                total: 0,
            };
        };
        let by_ids = !query.ids.is_empty();
        let tags: Vec<Vec<u32>> = query.tags.iter().map(|t| self.tags_called(t)).collect();
        let version = filters
            .version
            .as_deref()
            .map(|v| self.tags_called(&v.to_lowercase()));
        let kind = filters.kind.tag().map(|k| self.tags_called(k));
        let owned: Option<Vec<u64>> = filters
            .owned_dlc
            .as_deref()
            .map(|o| o.iter().filter_map(|d| d.parse().ok()).collect());
        let mut hits: Vec<(u8, usize)> = Vec::new();
        for at in 0..self.len() {
            let (title, author) = (self.title_key(at), self.author_key(at));
            let admitted = if by_ids {
                given.contains(&self.id_at(at))
            } else {
                version.as_ref().is_none_or(|v| self.has_any_tag(at, v))
                    && kind.as_ref().is_none_or(|k| self.has_any_tag(at, k))
                    && owned.as_ref().is_none_or(|owned| {
                        self.dlc_of(at).iter().all(|needed| owned.contains(needed))
                    })
            };
            let wanted = admitted
                && tags.iter().all(|t| self.has_any_tag(at, t))
                && dlc.iter().all(|d| self.dlc_of(at).contains(d))
                && requires.iter().all(|m| self.requires_of(at).contains(m))
                && query.authors.iter().all(|a| author.contains(a.as_str()))
                && query.words.iter().all(|w| {
                    [title, author, self.package_key(at)]
                        .iter()
                        .any(|s| s.contains(w.as_str()))
                });
            if !wanted {
                continue;
            }
            let tier = if query.words.is_empty() || title == phrase {
                0
            } else if title.starts_with(&phrase) {
                1
            } else if query.words.iter().all(|w| title.contains(w.as_str())) {
                2
            } else {
                3
            };
            hits.push((tier, at));
        }
        let total = hits.len();
        if query.words.is_empty() {
            hits.reverse();
        } else {
            let order = |&(tier, at): &(u8, usize)| (tier, self.title_key(at), at);
            hits.sort_unstable_by_key(order);
        }
        let ids = hits
            .iter()
            .skip(offset)
            .take(count)
            .map(|&(_, at)| self.id_at(at).to_string())
            .collect();
        Page { ids, total }
    }
}

#[cfg(test)]
mod tests {
    use super::super::workshopindex::{Entry, entry};
    use super::*;

    fn index() -> Index {
        let mut with_pkg = entry("600000", "Unrelated", Some("Someone"), &["Mod", "1.5"]);
        with_pkg.package_id = Some("vanillaexpanded.harmony".into());
        let mut needs = entry("700000", "Needs Stuff", Some("Ann Lee"), &["Mod", "1.6"]);
        needs.requires = vec!["100000".into()];
        needs.dlc = vec!["1826140".into()];
        let all: Vec<Entry> = vec![
            entry("100000", "Harmony", Some("Brrainz"), &["Mod", "1.6", "1.5"]),
            entry("200000", "Harmony Patches Extra", None, &["Mod", "1.6"]),
            entry("300000", "More Harmony", None, &["Translation", "1.6"]),
            entry("400000", "Zeta Harmony tweaks", None, &["Scenario"]),
            entry("500000", "Éclair", Some("Brrainz"), &["Mod"]),
            with_pkg,
            needs,
        ];
        Index::new(all)
    }

    fn found(text: &str, filters: &Filters) -> Vec<String> {
        index().search(&Query::parse(text), filters, 0, 100).ids
    }

    #[test]
    fn the_box_is_split_into_words_and_keys() {
        let q = Query::parse(
            r#"Big "Red Mod" t:Mod d:1826140 m:100000 A:"Ann Lee" w:42 https://steamcommunity.com/sharedfiles/filedetails/?id=2009463077 818773962 w:abc d:x"#,
        );
        assert_eq!(q.words, ["big", "red mod", "w:abc", "d:x"]);
        assert_eq!(
            (q.tags, q.dlc, q.requires),
            (
                vec!["mod".to_string()],
                vec!["1826140".to_string()],
                vec!["100000".to_string()]
            )
        );
        assert_eq!(q.authors, ["ann lee"]);
        assert_eq!(q.ids, ["42", "2009463077", "818773962"]);
        assert_eq!(Query::parse("  "), Query::default());
    }

    #[test]
    fn ranking_puts_the_title_first_then_its_start_then_inside_then_the_rest() {
        let got = found("harmony", &Filters::default());
        assert_eq!(got, ["100000", "200000", "300000", "400000", "600000"]);
        assert_eq!(found("harmony patches", &Filters::default()), ["200000"]);
        assert_eq!(found("HARMONY", &Filters::default())[0], "100000");
    }

    #[test]
    fn the_author_and_package_id_are_searched_after_the_title() {
        assert_eq!(found("brrainz", &Filters::default()), ["100000", "500000"]);
        assert_eq!(found("vanillaexpanded", &Filters::default()), ["600000"]);
        assert_eq!(found("éCLAIR", &Filters::default()), ["500000"]);
    }

    #[test]
    fn keys_narrow_the_search() {
        let any = Filters::default();
        assert_eq!(found("t:translation", &any), ["300000"]);
        assert_eq!(found("m:100000", &any), ["700000"]);
        assert_eq!(found("d:1826140 harmony", &any), Vec::<String>::new());
        assert_eq!(found("d:1826140", &any), ["700000"]);
        assert_eq!(found("a:ann", &any), ["700000"]);
        assert_eq!(found("w:300000 harmony", &any), ["300000"]);
        assert_eq!(
            found(
                "https://steamcommunity.com/sharedfiles/filedetails/?id=200000",
                &any
            ),
            ["200000"]
        );
    }

    #[test]
    fn filters_go_by_steams_tags_and_leave_given_ids_alone() {
        let v16 = Filters {
            version: Some("1.6".into()),
            kind: Kind::Mod,
            ..Default::default()
        };
        assert_eq!(found("harmony", &v16), ["100000", "200000"]);
        assert_eq!(found("w:300000", &v16), ["300000"]);
        let scenario = Filters {
            kind: Kind::Scenario,
            ..Default::default()
        };
        assert_eq!(found("", &scenario), ["400000"]);
        let none_owned = Filters {
            owned_dlc: Some(vec![]),
            ..Default::default()
        };
        assert!(!found("", &none_owned).contains(&"700000".to_string()));
        let owned = Filters {
            owned_dlc: Some(vec!["1826140".into()]),
            ..Default::default()
        };
        assert!(found("", &owned).contains(&"700000".to_string()));
    }

    #[test]
    fn with_no_words_the_newest_come_first_and_with_words_the_best() {
        let all = index().search(&Query::default(), &Filters::default(), 0, 10);
        assert_eq!(all.ids.first().map(String::as_str), Some("700000"));
        assert!(all.ids.windows(2).all(|w| w[0] > w[1]), "{:?}", all.ids);
        assert_eq!(found("harmony", &Filters::default())[0], "100000");
    }

    #[test]
    fn a_page_is_a_slice_of_the_whole() {
        let page = index().search(&Query::parse("harmony"), &Filters::default(), 1, 2);
        assert_eq!(
            (page.total, page.ids),
            (5, vec!["200000".to_string(), "300000".to_string()])
        );
        let past = index().search(&Query::default(), &Filters::default(), 50, 10);
        assert_eq!((past.total, past.ids.len()), (7, 0));
    }

    #[test]
    fn a_title_padded_with_spaces_sorts_and_matches_by_its_text() {
        let index = Index::new(vec![
            entry("100000", "  Zebra", None, &["Mod"]),
            entry("200000", "Apple", None, &["Mod"]),
        ]);
        let all = index.search(&Query::parse("a"), &Filters::default(), 0, 10);
        assert_eq!(all.ids, ["200000", "100000"]);
        assert_eq!(index.get("100000").unwrap().title, "Zebra");
    }
}
