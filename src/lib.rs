//! A library for fetching data from The Pirate Bay's API.
//!
//! This library does not aim to be a fully featured client, but contributions
//! are welcome if you feel something is missing. This is *not* a torrent client,
//! it only provides metadata on torrents.
//!
//! This client uses the JSON API and as such currently won't work with most mirrors.
//!
//! # Example
//!
//! ```
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use tpb::{Category, search, torrent};
//!
//! // Get a category by ID to search within that category.
//! let movies = Category::new(201).unwrap();
//! assert_eq!(movies.name(), "Video: Movies");
//!
//! // Search for torrents by name match and category.
//! let torrents = search("Barbie", Some(movies)).await?;
//! println!("Found {} torrents", torrents.len());
//!
//! // Search returns most attributes of a torrent.
//! let first = &torrents[0];
//! println!("First torrent: {}", first.name);
//! println!("Magnet link: {}", first.magnet());
//!
//! // And the missing ones can be fetched by torrent ID.
//! let torrent = torrent(first.id).await?;
//! println!("Description: {}", torrent.descr);
//! # Ok(())
//! # }
//! ```
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
mod deser;
mod scraped;
mod types;

pub use scraped::{CATEGORIES, TRACKERS};
pub use types::*;

const API: &str = "https://apibay.org";
type Result<T> = std::result::Result<T, reqwest::Error>;

thread_local! {
    static CLIENT: reqwest::Client = reqwest::Client::new();
}

/// Search for torrents by name match and optionally category.
///
/// # Errors
///
/// This function returns an error if the request fails or the response is invalid.
pub async fn search(query: &str, category: Option<Category>) -> Result<Vec<PartialTorrent>> {
    let cat = category.map(|cat| cat.0.to_string()).unwrap_or_default();
    let torrents = CLIENT
        .with(|client| {
            client
                .get(format!("{API}/q.php"))
                .query(&[("q", query), ("cat", &cat)])
                .send()
        })
        .await?
        .json()
        .await?;
    Ok(torrents)
}

/// Get the top 100 torrents by category.
///
/// If `last_48h` is true, only torrents uploaded in the last 48 hours are returned.
///
/// # Errors
///
/// This function returns an error if the request fails or the response is invalid.
pub async fn top100(category: Category, last_48h: bool) -> Result<Vec<PartialTorrent>> {
    let specifier = if last_48h { "_48h" } else { "" };
    let torrents = CLIENT
        .with(|client| {
            client
                .get(format!(
                    "{API}/precompiled/data_top100{spec}_{cat}.json",
                    API = API,
                    spec = specifier,
                    cat = category.0,
                ))
                .send()
        })
        .await?
        .json()
        .await?;
    Ok(torrents)
}

/// Get full metadata on a torrent by ID.
///
/// # Errors
///
/// This function returns an error if the request fails or the response is invalid.
pub async fn torrent(id: u64) -> Result<Torrent> {
    let torrent = CLIENT
        .with(|client| {
            client
                .get(format!("{API}/t.php"))
                .query(&[("id", id.to_string())])
                .send()
        })
        .await?
        .json()
        .await?;
    Ok(torrent)
}

/// Get a list of file metadata for a torrent by ID.
///
/// # Errors
///
/// This function returns an error if the request fails or the response is invalid.
pub async fn torrent_files(id: u64) -> Result<Vec<TorrentFile>> {
    let files = CLIENT
        .with(|client| {
            client
                .get(format!("{API}/f.php"))
                .query(&[("id", id.to_string())])
                .send()
        })
        .await?
        .json()
        .await?;
    Ok(files)
}

impl PartialTorrent {
    /// Generate a magnet link for this torrent.
    #[must_use]
    pub fn magnet(&self) -> String {
        // We encode the hash manually because the URL builder escapes the colons.
        format!("magnet:?xt=urn:btih:{}", self.info_hash)
            .parse::<reqwest::Url>()
            .expect("magnet link failed to parse - invalid info hash?")
            .query_pairs_mut()
            // TPB has slightly different escaping rules here but it doesn't seem to be an issue.
            .append_pair("dn", &self.name)
            .extend_pairs(TRACKERS.iter().map(|tracker| ("tr", tracker)))
            .finish()
            .to_string()
    }
}

impl std::ops::Deref for Torrent {
    type Target = PartialTorrent;

    fn deref(&self) -> &Self::Target {
        &self.partial
    }
}

impl Category {
    /// Get a category by category ID.
    ///
    /// Returns [`None`] if the ID is not a valid category ID. See [`CATEGORIES`] for a list of
    /// valid category IDs.
    #[must_use]
    pub fn new(id: u16) -> Option<Self> {
        if CATEGORIES.iter().any(|(category_id, _)| category_id == &id) {
            Some(Self(id))
        } else {
            None
        }
    }

    /// Iterate over all categories.
    pub fn all() -> impl Iterator<Item = Self> {
        CATEGORIES.iter().map(|(id, _)| Self(*id))
    }

    /// Look up the name of this category.
    #[must_use]
    pub fn name(&self) -> &'static str {
        fn lookup_id(id: u16) -> Option<&'static str> {
            CATEGORIES
                .iter()
                .find(|(category_id, _)| category_id == &id)
                .map(|(_, name)| *name)
        }
        lookup_id(self.0)
            .or_else(|| lookup_id(self.0 / 100))
            .unwrap_or("Unknown")
    }

    /// Get the code for this category.
    #[must_use]
    pub const fn code(&self) -> u16 {
        self.0
    }
}
