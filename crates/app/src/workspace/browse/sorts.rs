use rimstack_core::base::sentence::Sentence;
use rimstack_core::say;
use rimstack_core::sources::browse::Sort;
use serde::Serialize;

/// Steam keeps Trending for a window of 1 to 7 days.
const SORTS: [(&str, Sort); 6] = [
    ("relevance", Sort::Relevance),
    ("trending1", Sort::Trending { days: 1 }),
    ("trending7", Sort::Trending { days: 7 }),
    ("subscribed", Sort::MostSubscribed),
    ("recent", Sort::MostRecent),
    ("updated", Sort::LastUpdated),
];

pub fn sort_of(id: &str) -> Sort {
    SORTS
        .iter()
        .find(|(known, _)| *known == id)
        .map_or(Sort::Relevance, |(_, sort)| *sort)
}

pub fn id_of(sort: Sort) -> &'static str {
    SORTS
        .iter()
        .find(|(_, known)| *known == sort)
        .map_or("relevance", |(id, _)| id)
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SortOffer {
    pub id: &'static str,
    pub label: Sentence,
    /// Whether it is the sort the results are in.
    pub chosen: bool,
}

fn label(sort: Sort) -> Sentence {
    match sort {
        Sort::Relevance => say!("Best match"),
        Sort::Trending { days: 1 } => say!("Trending today"),
        Sort::Trending { .. } => say!("Trending this week"),
        Sort::MostSubscribed => say!("Most subscribed"),
        Sort::MostRecent => say!("Newest"),
        Sort::LastUpdated => say!("Recently updated"),
    }
}

pub fn offers() -> Vec<SortOffer> {
    SORTS
        .iter()
        .map(|(id, sort)| SortOffer {
            id,
            label: label(*sort),
            chosen: false,
        })
        .collect()
}

/// The offers with the one named `id` chosen.
pub fn chosen(offers: &[SortOffer], id: &str) -> Vec<SortOffer> {
    offers
        .iter()
        .map(|offer| SortOffer {
            chosen: offer.id == id,
            ..offer.clone()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_offered_sort_is_read_back_from_its_id_and_an_unknown_one_is_relevance() {
        for offer in offers() {
            assert_eq!(id_of(sort_of(offer.id)), offer.id);
        }
        assert_eq!(sort_of("nonsense"), Sort::Relevance);
        assert_eq!(sort_of("trending7"), Sort::Trending { days: 7 });
    }

    #[test]
    fn one_offer_is_chosen_by_its_id() {
        let marked = chosen(&offers(), "subscribed");
        let ids: Vec<&str> = marked.iter().filter(|o| o.chosen).map(|o| o.id).collect();
        assert_eq!(ids, ["subscribed"]);
        assert!(chosen(&offers(), "nonsense").iter().all(|o| !o.chosen));
    }
}
