#![allow(dead_code)]

use std::str::FromStr;

use appview_schema::models::appview::Actor;
use appview_schema::models::appview::Profile as SchemaProfile;
use campground_lexicon::gg::campground::actor::ProfileViewDetailed;
use campground_lexicon::gg::campground::actor::{Profile, ProfileView, ProfileViewBasic};
use common::get_blob_ref;
use lexicon_cid::CidGeneric;
use rsky_lexicon::com::atproto::repo::Blob;

use crate::views::util::parse_datetime;

pub fn profile_view_basic(actor: &Actor, profile: Option<&SchemaProfile>) -> ProfileViewBasic {
    match profile {
        Some(profile) => ProfileViewBasic {
            did: actor.did.clone(),
            handle: match &actor.handle {
                Some(handle) => handle.clone(),
                None => "handle.invalid".to_string(),
            },
            tagline: profile.tagline.clone(),
            display_name: profile.display_name.clone(),
            avatar: get_blob_ref(&avatar_from_cid(profile.avatar_cid.clone())),
            created_at: profile.created_at.clone(),
            activity: None,
            status: None,
            viewer: None,
            labels: None,
        },
        None => ProfileViewBasic {
            did: actor.did.clone(),
            handle: actor.handle.clone().unwrap_or("handle.invalid".to_string()),

            display_name: None,
            avatar: None,

            tagline: None,
            activity: None,
            status: None,
            viewer: None,
            labels: None,

            created_at: Some(actor.indexed_at.clone()),
        },
    }
}
pub fn profile_view_detailed(
    actor: &Actor,
    profile: Option<&SchemaProfile>,
) -> ProfileViewDetailed {
    match profile {
        Some(profile) => ProfileViewDetailed {
            did: actor.did.clone(),
            handle: match &actor.handle {
                Some(handle) => handle.clone(),
                None => "handle.invalid".to_string(),
            },
            display_name: profile.display_name.clone(),
            avatar: get_blob_ref(&avatar_from_cid(profile.avatar_cid.clone())),
            banner: get_blob_ref(&avatar_from_cid(profile.banner_cid.clone())),

            tagline: profile.tagline.clone(),
            description: profile.description.clone(),
            location: profile.location.clone(),

            status: None,
            viewer: None,
            labels: vec![],
            social_connections: None,

            created_at: Some(actor.indexed_at.clone()),
            indexed_at: Some(actor.indexed_at.clone()),
            activities: vec![],
        },
        None => ProfileViewDetailed {
            did: actor.did.clone(),
            handle: actor.handle.clone().unwrap_or("handle.invalid".to_string()),

            display_name: None,
            avatar: None,
            banner: None,

            description: None,
            location: None,

            tagline: None,
            status: None,
            viewer: None,
            labels: vec![],
            social_connections: None,

            created_at: Some(actor.indexed_at.clone()),
            indexed_at: Some(actor.indexed_at.clone()),
            activities: vec![],
        },
    }
}

pub fn profile_view_basic_deleted_actor(did: String) -> ProfileViewBasic {
    ProfileViewBasic {
        did: did.clone(),
        handle: "handle.invalid".to_string(),
        display_name: None,
        tagline: None,
        avatar: None,
        created_at: None,
        activity: None,
        status: None,
        viewer: None,
        labels: None,
    }
}

pub fn profile_view(actor: &Actor, profile: &Profile) -> ProfileView {
    ProfileView {
        did: actor.did.clone(),
        handle: match &actor.handle {
            Some(handle) => handle.clone(),
            None => "handle.invalid".to_string(),
        },

        display_name: profile.display_name.clone(),
        avatar: get_blob_ref(&profile.avatar),
        banner: get_blob_ref(&profile.banner),

        tagline: profile.tagline.clone(),
        description: profile.description.clone(),
        status: None,

        activities: vec![],
        labels: vec![],
        indexed_at: None,
        created_at: parse_datetime(profile.created_at),
    }
}

fn avatar_from_cid(cid: Option<String>) -> Option<Blob> {
    match cid {
        Some(cid) => Some(Blob {
            r#type: None,
            r#ref: Some(CidGeneric::from_str(&cid).unwrap()),
            cid: None,
            mime_type: "image".to_string(),
            size: None,
            original: None,
        }),
        None => None,
    }
}
