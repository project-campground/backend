#![
    allow(dead_code)
]

use std::str::FromStr;

use appview_schema::models::appview::{Actor, ProfilePost};
use campground_lexicon::gg::campground::actor::Profile;
use campground_lexicon::gg::campground::profile::{ProfilePostViewBasic, ProfilePostViewDetailed, ProfilePostViewParented};
use chrono::DateTime;

use crate::helpers::util::parse_datetime;
use crate::helpers::views::profile_view_basic;

pub fn profile_post_view_basic(actor: &Actor, profile: &Profile, profile_post: &ProfilePost) -> ProfilePostViewBasic {
    return ProfilePostViewBasic {
        cid: profile_post.cid.clone(),
        uri: profile_post.uri.clone(),
        parent_uri: profile_post.parent_uri.clone(),
        content: profile_post.content.clone(),
        author: profile_view_basic(actor, profile),
        reply_count: profile_post.replies.len(),
        indexed_at: parse_datetime(DateTime::from_str(&profile_post.indexed_at).ok()),
        created_at: parse_datetime(DateTime::from_str(&profile_post.created_at).ok()),
        updated_at: match profile_post.updated_at.clone() {
            Some(datetime) => parse_datetime(DateTime::from_str(&datetime).ok()),
            None => None,
        }
    };
}

pub fn profile_post_view_parented(actor: &Actor, profile: &Profile, profile_post: &ProfilePost, parent: &Option<ProfilePostViewBasic>) -> ProfilePostViewParented {
    return ProfilePostViewParented {
        cid: profile_post.cid.clone(),
        uri: profile_post.uri.clone(),
        parent_uri: profile_post.parent_uri.clone(),
        content: profile_post.content.clone(),
        author: profile_view_basic(actor, profile),
        reply_count: profile_post.replies.len(),
        indexed_at: parse_datetime(DateTime::from_str(&profile_post.indexed_at).ok()),
        created_at: parse_datetime(DateTime::from_str(&profile_post.created_at).ok()),
        updated_at: match profile_post.updated_at.clone() {
            Some(datetime) => parse_datetime(DateTime::from_str(&datetime).ok()),
            None => None,
        },
        parent: parent.clone(),
    };
}

pub fn profile_post_view_detailed(actor: &Actor, profile: &Profile, profile_post: &ProfilePost, replies: Vec<ProfilePostViewBasic>, parent: &Option<ProfilePostViewBasic>) -> ProfilePostViewDetailed {
    return ProfilePostViewDetailed {
        cid: profile_post.cid.clone(),
        uri: profile_post.uri.clone(),
        parent_uri: profile_post.parent_uri.clone(),
        content: profile_post.content.clone(),
        author: profile_view_basic(actor, profile),
        replies: replies,
        indexed_at: parse_datetime(DateTime::from_str(&profile_post.indexed_at).ok()),
        created_at: parse_datetime(DateTime::from_str(&profile_post.created_at).ok()),
        updated_at: match profile_post.updated_at.clone() {
            Some(datetime) => parse_datetime(DateTime::from_str(&datetime).ok()),
            None => None,
        },
        parent: parent.clone(),
    };
}
