use super::filter::tagged;
use crate::base::sentence::Sentence;
use crate::{say, say_n};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Offer {
    pub term: String,
    pub about: Sentence,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct Offers {
    pub offers: Vec<Offer>,
    /// The longest term offered, which their column is measured on.
    pub widest: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Counted {
    pub errors: usize,
    pub warnings: usize,
    pub conflicts: usize,
    /// Active mods with an update; 0 before Steam is asked too.
    pub updated: usize,
}

type About = fn(&Counted) -> Sentence;

const TERMS: &[(&str, About)] = &[
    ("is:error", |c| say_n!(c.errors, "{} mod", "{} mods")),
    ("is:warning", |c| say_n!(c.warnings, "{} mod", "{} mods")),
    ("is:conflict", |c| {
        say_n!(
            c.conflicts,
            "{} mod sharing textures",
            "{} mods sharing textures"
        )
    }),
    ("is:changed", |_| {
        say!("moved or activated since the last save")
    }),
    ("is:new", |_| {
        say!("installed since the game was last started")
    }),
    ("is:updated", |c| match c.updated {
        0 => say!("after checking for updates"),
        n => say!("{} with an update", n),
    }),
    ("is:active", |_| {
        say!("the load order only; the inactive list is dimmed or hidden")
    }),
    ("is:inactive", |_| {
        say!("the inactive list only; the load order is dimmed or hidden")
    }),
    ("is:workshop", |_| say!("from the Workshop")),
    ("is:local", |_| say!("in the game's Mods folder")),
    ("is:git", |_| say!("a git checkout")),
    ("is:notinsave", |_| {
        say!("active, and not in the newest save")
    }),
    ("is:recent", |_| say!("updated by Steam in the last week")),
    ("is:duplicate", |_| say!("installed more than once")),
    ("in:search", |_| {
        say!("has matches in the last search of Find")
    }),
    ("has:code", |_| say!("ships compiled code")),
    ("has:defs", |_| say!("has XML defs")),
    ("has:patches", |_| say!("has XML patches")),
    ("has:textures", |_| say!("has textures")),
    ("has:sounds", |_| say!("has sounds")),
    ("has:languages", |_| say!("has translations")),
    ("defines:", |_| say!("defines this defName")),
    ("patchesdef:", |_| say!("patches this defName")),
    ("patches:", |_| say!("patches this method of the game")),
    ("tag:", |_| say!("has this tag")),
    ("author:", |_| say!("by this author")),
];

const MOST_TAGS: usize = 12;

/// What the filter offers for `typed`, the word being written, beside `text`, the whole filter;
/// `tags` holds each mod's own. A term already in `text` is not offered again.
pub fn offers<'a>(
    typed: &str,
    text: &str,
    counted: &Counted,
    tags: impl IntoIterator<Item = &'a [String]>,
) -> Offers {
    let typed = typed.to_lowercase();
    let taken: HashSet<String> = text.split_whitespace().map(str::to_lowercase).collect();
    let fits = |term: &str| {
        let term = term.to_lowercase();
        term.starts_with(&typed) && term != typed && !taken.contains(&term)
    };
    let (sign, bare) = match typed.strip_prefix('!') {
        Some(rest) => ("!", rest),
        None => ("", typed.as_str()),
    };
    let offers: Vec<Offer> = match bare.starts_with("tag:") {
        true => tag_offers(sign, tags, fits),
        false => TERMS
            .iter()
            .filter(|(term, _)| sign.is_empty() || !matches!(*term, "is:active" | "is:inactive"))
            .map(|(term, about)| (format!("{sign}{term}"), about))
            .filter(|(term, _)| fits(term))
            .map(|(term, about)| Offer {
                term,
                about: about(counted),
            })
            .collect(),
    };
    let widest = offers
        .iter()
        .map(|offer| &offer.term)
        .max_by_key(|term| term.chars().count())
        .cloned()
        .unwrap_or_default();
    Offers { offers, widest }
}

/// A term holds one word, and `tag:` matches the start of a tag, so a tag is offered by its
/// first word.
fn tag_offers<'a>(
    sign: &str,
    tags: impl IntoIterator<Item = &'a [String]>,
    fits: impl Fn(&str) -> bool,
) -> Vec<Offer> {
    let tags: Vec<&[String]> = tags.into_iter().collect();
    let mut first_words: BTreeMap<String, String> = BTreeMap::new();
    for tag in tags.iter().flat_map(|own| own.iter()) {
        if let Some(word) = tag.split_whitespace().next() {
            let term = format!("{sign}tag:{word}");
            if !first_words.contains_key(&word.to_lowercase()) && fits(&term) {
                first_words.insert(word.to_lowercase(), term);
            }
        }
    }
    let mut counted: Vec<(usize, String, String)> = first_words
        .into_iter()
        .map(|(lower, term)| {
            let mods = tags.iter().filter(|own| tagged(own, &lower)).count();
            (mods, lower, term)
        })
        .collect();
    counted.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    counted
        .into_iter()
        .take(MOST_TAGS)
        .map(|(mods, _, term)| Offer {
            term,
            about: say_n!(mods, "{} mod", "{} mods"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terms(typed: &str, text: &str, tags: &[Vec<String>]) -> Vec<String> {
        let found = offers(
            typed,
            text,
            &Counted::default(),
            tags.iter().map(Vec::as_slice),
        );
        found.offers.into_iter().map(|offer| offer.term).collect()
    }

    fn tagged(each: &[&[&str]]) -> Vec<Vec<String>> {
        let own = |tags: &&[&str]| tags.iter().map(|t| t.to_string()).collect();
        each.iter().map(own).collect()
    }

    #[test]
    fn an_empty_word_offers_every_term_in_order() {
        let all = terms("", "", &[]);
        assert_eq!(all.len(), TERMS.len());
        assert_eq!(all[0], "is:error");
    }

    #[test]
    fn a_bang_offers_each_term_left_out_but_the_two_lists() {
        let all = terms("!", "", &[]);
        assert_eq!(all.len(), TERMS.len() - 2);
        assert!(all.iter().all(|t| t.starts_with('!')));
        assert!(!all.iter().any(|t| t == "!is:active" || t == "!is:inactive"));
        assert_eq!(terms("!has:t", "", &[]), ["!has:textures"]);
    }

    #[test]
    fn a_term_written_or_already_taken_is_not_offered() {
        assert_eq!(terms("is:e", "", &[]), ["is:error"]);
        assert!(terms("is:error", "", &[]).is_empty());
        assert!(terms("is:e", "is:error is:e", &[]).is_empty());
        assert_eq!(terms("IS:E", "", &[]), ["is:error"]);
    }

    #[test]
    fn tags_are_offered_after_tag_or_its_leaving_out() {
        let tags = tagged(&[&["HUD", "Quality of life"], &["hud"], &["Medieval"], &[]]);
        assert_eq!(
            terms("tag:", "", &tags),
            ["tag:HUD", "tag:Medieval", "tag:Quality"]
        );
        assert_eq!(terms("!tag:h", "", &tags), ["!tag:HUD"]);
        assert_eq!(terms("tag:q", "tag:medieval", &tags), ["tag:Quality"]);
        assert!(terms("tag:hud", "", &tags).is_empty());
        assert!(
            terms("is:", "", &tags)
                .iter()
                .all(|t| !t.starts_with("tag:"))
        );
    }

    #[test]
    fn a_tag_counts_every_mod_its_term_matches() {
        let tags = tagged(&[&["HUD"], &["hud", "HUDs"], &["Huds"]]);
        let found = offers(
            "tag:hud",
            "",
            &Counted::default(),
            tags.iter().map(Vec::as_slice),
        );
        let about: Vec<String> = found.offers.iter().map(|o| o.about.to_string()).collect();
        assert_eq!(about, ["2 mods"]);
        let found = offers(
            "tag:",
            "",
            &Counted::default(),
            tags.iter().map(Vec::as_slice),
        );
        let all: Vec<(String, String)> = found
            .offers
            .into_iter()
            .map(|o| (o.term, o.about.to_string()))
            .collect();
        assert_eq!(
            all,
            [
                ("tag:HUD".into(), "3 mods".into()),
                ("tag:HUDs".into(), "2 mods".into())
            ]
        );
    }

    #[test]
    fn tags_offered_are_few_and_the_widest_is_named() {
        let many: Vec<Vec<String>> = (0..40).map(|n| vec![format!("tag{n:02}")]).collect();
        let found = offers(
            "tag:",
            "",
            &Counted::default(),
            many.iter().map(Vec::as_slice),
        );
        assert_eq!(found.offers.len(), MOST_TAGS);
        let widest = offers("has:", "", &Counted::default(), std::iter::empty()).widest;
        assert_eq!(widest, "has:languages");
        assert_eq!(
            offers("zzz", "", &Counted::default(), std::iter::empty()).widest,
            ""
        );
    }
}
