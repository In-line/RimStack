mod add;
mod fixture;
mod memory;
mod steam;

pub use add::{Added, Refusal, add_with_requirements};
pub use fixture::{COLLECTION, Fixture, HARMONY};
pub use memory::Memory;
pub use steam::{Canned, Failure, KeyCheck, Steam, Transport, Web, check_key};

use crate::base::error::Error;
use crate::rimworld::workshop::{self, Details};
use crate::rimworld::workshopindex::{Entry, Index};
use crate::rimworld::workshopsearch::{Filters, Page, Query};
use std::sync::Arc;

/// Only Steam with the key sorts; the cache and the fixture rank by relevance, and by newest with
/// no words, whatever is asked.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Sort {
    /// By the words, or by votes with none.
    #[default]
    Relevance,
    /// Steam keeps a window of 1 to 7 days.
    Trending {
        days: u8,
    },
    MostSubscribed,
    MostRecent,
    LastUpdated,
}

#[derive(Debug, Clone, Default)]
pub struct Request {
    pub query: Query,
    pub filters: Filters,
    pub sort: Sort,
    /// Look for collections instead of items; only Steam with the key does.
    pub collections: bool,
    pub offset: usize,
    pub count: usize,
}

pub trait Browse: Send + Sync {
    fn search(&self, request: &Request) -> Result<Page, Error>;

    /// Blocks on the network for the cache. A mod Steam does not know is left out.
    fn details(&self, ids: &[String]) -> Result<Vec<Details>, Error>;

    /// What the source knows of a mod without asking: `requires` is as published, known or not.
    fn entry(&self, id: &str) -> Option<Entry>;

    /// The name the database gives an expansion's Steam app id.
    fn app_name(&self, app_id: &str) -> Option<String>;

    /// The game versions to filter by, newest first; see `Index::offered_versions`.
    fn versions(&self, game: Option<&str>) -> Vec<String>;

    /// Its Workshop requirements that the source knows.
    fn requires(&self, id: &str) -> Result<Vec<String>, Error>;

    fn needed_by(&self, id: &str) -> Result<Vec<String>, Error>;

    fn same_author(&self, id: &str) -> Result<Vec<String>, Error>;

    /// A collection's items, or the id itself for a mod that is not a collection.
    fn collection_items(&self, ids: &[String]) -> Result<Vec<String>, Error>;
}

pub struct Cache {
    index: Arc<Index>,
}

impl Cache {
    pub fn new(index: Arc<Index>) -> Self {
        Cache { index }
    }
}

impl Browse for Cache {
    fn search(&self, request: &Request) -> Result<Page, Error> {
        let Request {
            query,
            filters,
            offset,
            count,
            ..
        } = request;
        Ok(self.index.search(query, filters, *offset, *count))
    }

    fn details(&self, ids: &[String]) -> Result<Vec<Details>, Error> {
        workshop::fetch_details(ids)
    }

    fn entry(&self, id: &str) -> Option<Entry> {
        self.index.get(id)
    }

    fn app_name(&self, app_id: &str) -> Option<String> {
        self.index.app_name(app_id).map(str::to_string)
    }

    fn versions(&self, game: Option<&str>) -> Vec<String> {
        self.index.offered_versions(game)
    }

    fn requires(&self, id: &str) -> Result<Vec<String>, Error> {
        Ok(self.index.requires(id))
    }

    fn needed_by(&self, id: &str) -> Result<Vec<String>, Error> {
        Ok(self.index.needed_by(id))
    }

    fn same_author(&self, id: &str) -> Result<Vec<String>, Error> {
        Ok(self.index.same_author(id))
    }

    fn collection_items(&self, ids: &[String]) -> Result<Vec<String>, Error> {
        workshop::expand_collections(ids)
    }
}
