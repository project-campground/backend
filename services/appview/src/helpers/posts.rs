#![
    allow(dead_code)
]

use std::str::FromStr;

use appview_schema::models::appview::{Actor, ProfilePost};
use campground_lexicon::gg::campground::actor::Profile;
use campground_lexicon::gg::campground::profile::{ProfilePostViewBasic, ProfilePostViewDetailed};
use chrono::{DateTime, Utc};

use crate::helpers::views::profile_view_basic;

pub fn parse_datetime(datetime: Option<DateTime<Utc>>) -> Option<String> {
    match datetime {
        Some(dt) => Some(dt.to_rfc3339()),
        None => None,
    }
}

pub fn profile_post_view_basic(actor: &Actor, profile: &Profile, profile_post: &ProfilePost) -> ProfilePostViewBasic {
    return ProfilePostViewBasic {
        cid: profile_post.cid.clone(),
        uri: profile_post.uri.clone(),
        content: profile_post.content.clone(),
        author: profile_view_basic(actor, profile),
        tags: profile_post.tags.clone(),
        reply_count: profile_post.replies.len(),
        indexed_at: parse_datetime(DateTime::from_str(&profile_post.indexed_at).ok()),
        created_at: parse_datetime(DateTime::from_str(&profile_post.created_at).ok()),
        updated_at: match profile_post.updated_at.clone() {
            Some(datetime) => parse_datetime(DateTime::from_str(&datetime).ok()),
            None => None,
        }
    };
}

pub fn profile_post_view_detailed(actor: &Actor, profile: &Profile, profile_post: &ProfilePost, replies: Vec<ProfilePostViewBasic>) -> ProfilePostViewDetailed {
    return ProfilePostViewDetailed {
        cid: profile_post.cid.clone(),
        uri: profile_post.uri.clone(),
        content: profile_post.content.clone(),
        author: profile_view_basic(actor, profile),
        tags: profile_post.tags.iter().filter(|x| x.is_some()).map(|x| x.clone().unwrap()).collect(),
        replies: replies,
        indexed_at: parse_datetime(DateTime::from_str(&profile_post.indexed_at).ok()),
        created_at: parse_datetime(DateTime::from_str(&profile_post.created_at).ok()),
        updated_at: match profile_post.updated_at.clone() {
            Some(datetime) => parse_datetime(DateTime::from_str(&datetime).ok()),
            None => None,
        }
    };
}
