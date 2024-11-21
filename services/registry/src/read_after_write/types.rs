/**
 * Implementation from https://github.com/blacksky-algorithms/rsky
 * Modified to work with our own DB
 * License: https://github.com/blacksky-algorithms/rsky/blob/main/LICENSE
 */
use libipld::Cid;
use rsky_lexicon::app::bsky::actor::Profile;
use rsky_lexicon::app::bsky::feed::Post;
use rsky_syntax::aturi::AtUri;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LocalRecords {
    pub count: i64,
    pub profile: Option<RecordDescript<Profile>>,
    pub posts: Vec<RecordDescript<Post>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RecordDescript<T> {
    pub uri: AtUri,
    pub cid: Cid,
    #[serde(rename = "indexedAt")]
    pub indexed_at: String,
    pub record: T,
}