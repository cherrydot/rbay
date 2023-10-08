//! Types returned by the API, all supporting serde deserialisation.
use crate::deser::{empty_as_none, parse_timestamp, u16_from_str, u64_from_str, unit_array};
use serde::Deserialize;

/// Full details on a torrent.
///
/// The API only returns this type when requesting a single torrent - for lists of
/// torrents, it returns a [`PartialTorrent`]. This type wraps a [`PartialTorrent`]
/// with a few extra attributes, and implements [`std::ops::Deref`] to allow access
/// to the underlying [`PartialTorrent`].
#[derive(Debug, Deserialize)]
#[non_exhaustive]
pub struct Torrent {
    /// The description.
    pub descr: String,
    // Not sure what these numbers represent.
    pub language: Option<usize>,
    pub textlanguage: Option<usize>,
    /// The rest of the details (these are also returned when listing multiple torrents).
    ///
    /// You will often not need to access this directly, as [`Torrent`] derefs to this type.
    #[serde(flatten)]
    pub partial: PartialTorrent,
}

/// A torrent with only the details returned by listing endpoints.
///
/// This is usually all you need, but to get the rest of the attributes, you can
/// call [`torrent`](crate::torrent) with the ID.
#[derive(Debug, Deserialize)]
#[non_exhaustive]
pub struct PartialTorrent {
    #[serde(deserialize_with = "u64_from_str")]
    pub id: u64,
    pub name: String,
    pub info_hash: String,
    #[serde(deserialize_with = "u64_from_str")]
    pub leechers: u64,
    #[serde(deserialize_with = "u64_from_str")]
    pub seeders: u64,
    #[serde(deserialize_with = "u64_from_str")]
    pub num_files: u64,
    #[serde(deserialize_with = "u64_from_str")]
    pub size: u64,
    pub username: String,
    #[serde(deserialize_with = "parse_timestamp")]
    pub added: chrono::DateTime<chrono::Utc>,
    pub status: UserStatus,
    pub category: Category,
    #[serde(deserialize_with = "empty_as_none")]
    pub imdb: Option<String>,
}

/// The trust status of an uploader.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum UserStatus {
    Member,
    Trusted,
    Helper,
    Vip,
    Moderator,
    SuperMod,
    Admin,
}

/// A media category code.
#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct Category(#[serde(deserialize_with = "u16_from_str")] pub(crate) u16);

/// Metadata on a file in a torrent.
#[derive(Debug, Deserialize)]
#[non_exhaustive]
pub struct TorrentFile {
    #[serde(deserialize_with = "unit_array")]
    pub name: String,
    #[serde(deserialize_with = "unit_array")]
    pub size: u64,
}
