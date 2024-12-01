pub mod like;

use super::actor::ProfileView;
use crate::app::bsky::actor::{ProfileViewBasic, ViewerState};
use crate::app::bsky::embed::{EmbedViews, Embeds};
use crate::app::bsky::richtext::Facet;
use crate::com::atproto::label::{Label, SelfLabels};
use crate::com::atproto::repo::StrongRef;
use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
#[serde(rename = "app.bsky.feed.post")]
#[serde(rename_all = "camelCase")]
pub struct Post {
    /// Client-declared timestamp when this post was originally created.
    pub created_at: DateTime<Utc>,
    /// The primary post content. Might be an empty string, if there are embeds.
    pub text: String,
    /// DEPRECATED: replaced by app.bsky.richtext.facet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entities: Option<Vec<EntityRef>>,
    /// Annotations of text (mentions, URLs, hashtags, .etc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<Facet>>,
    /// Indicates human language of post primary text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub langs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<PostLabels>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<Embeds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<ReplyRef>,
    /// Additional hashtags, in addition to any included in post text and facets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
pub enum PostLabels {
    #[serde(rename = "com.atproto.label.defs#selfLabels")]
    SelfLabels(SelfLabels),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
#[serde(rename = "app.bsky.feed.defs#postView")]
#[serde(rename_all = "camelCase")]
pub struct PostView {
    pub uri: String,
    pub cid: String,
    pub author: ProfileViewBasic,
    pub record: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<EmbedViews>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repost_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub like_count: Option<usize>,
    pub indexed_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<ViewerState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<Label>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasonRepost {
    pub by: ProfileViewBasic,
    pub indexed_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedViewPost {
    pub post: PostView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<ReplyRefView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<ReasonRepost>,
    /// Context provided by feed generator that may be passed back alongside interactions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_context: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AuthorFeed {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    pub feed: Vec<FeedViewPost>,
}

///like from app.bsky.feed.getLikes
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLikesLike {
    pub created_at: DateTime<Utc>,
    pub indexed_at: DateTime<Utc>,
    pub actor: ProfileView,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repost {
    pub created_at: DateTime<Utc>,
    pub subject: StrongRef,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplyRefView {
    pub root: ReplyRefUnion,
    pub parent: ReplyRefUnion,
    /// When parent is a reply to another post, this is the author of that post.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grandparent_author: Option<ProfileViewBasic>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ReplyRef {
    pub root: StrongRef,
    pub parent: StrongRef,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ReplyRefUnion {
    #[serde(rename = "#postView")]
    PostView(PostView),
    #[serde(rename = "#notFoundPost")]
    NotFoundPost(NotFoundPost),
    #[serde(rename = "#blockedPost")]
    BlockedPost(BlockedPost),
}

impl ReplyRefUnion {
    pub fn uri(&self) -> &str {
        match self {
            ReplyRefUnion::PostView(post) => post.uri.as_str(),
            ReplyRefUnion::NotFoundPost(post) => post.uri.as_str(),
            ReplyRefUnion::BlockedPost(post) => post.uri.as_str(),
        }
    }
}

/// Deprecated: use facets instead.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct EntityRef {
    pub index: TextSlice,
    /// Expected values are 'mention' and 'link'.
    #[serde(rename = "type")]
    pub r#type: StrongRef,
    pub value: String,
}

/// Deprecated. Use app.bsky.richtext instead -- A text segment. Start is inclusive, end is exclusive.
/// Indices are for utf16-encoded strings.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TextSlice {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GetLikes {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GetLikesOutput {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
    pub likes: Vec<GetLikesLike>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ThreadViewPost {
    pub post: PostView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Box<ThreadViewPostEnum>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replies: Option<Vec<Box<ThreadViewPostEnum>>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotFoundPost {
    pub uri: String,
    pub not_found: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BlockedPost {
    pub uri: String,
    pub blocked: bool,
    pub author: BlockedAuthor,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BlockedAuthor {
    pub did: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<ViewerState>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
#[serde(rename = "app.bsky.feed.defs#generatorView")]
#[serde(rename_all = "camelCase")]
pub struct GeneratorView {
    pub uri: String,
    pub cid: String,
    pub did: String,
    pub creator: ProfileView,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_facets: Option<Vec<Facet>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub like_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepts_interactions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<Label>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<GeneratorViewState>,
    pub indexed_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GeneratorViewState {
    pub like: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
pub enum ThreadViewPostEnum {
    #[serde(rename = "app.bsky.feed.defs#threadViewPost")]
    ThreadViewPost(ThreadViewPost),
    #[serde(rename = "app.bsky.feed.defs#notFoundPost")]
    NotFoundPost(NotFoundPost),
    #[serde(rename = "app.bsky.feed.defs#blockedPost")]
    BlockedPost(BlockedPost),
}

///api.bsky.feed.getPostThread
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GetPostThread {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<usize>,
}
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GetPostThreadOutput {
    pub thread: ThreadViewPostEnum,
}
