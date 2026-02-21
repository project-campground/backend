#![
    allow(dead_code)
]

use std::str::FromStr;

use appview_schema::models::appview::Actor;
use appview_schema::models::appview::Profile as SchemaProfile;
use campground_lexicon::gg::campground::actor::{Profile, ProfileView, ProfileViewBasic, ProfileViewDetailed};
use chrono::DateTime;
use common::get_blob_ref;
use lexicon_cid::CidGeneric;
use rsky_lexicon::com::atproto::repo::Blob;

use crate::helpers::util::parse_datetime;

pub fn profile_view_basic(actor: &Actor, profile: &Profile) -> ProfileViewBasic {
    ProfileViewBasic {
        did: actor.did.clone(),
        handle: match &actor.handle {
            Some(handle) => handle.clone(),
            None => "handle.invalid".to_string()
        },
        tagline: profile.tagline.clone(),
        display_name: profile.display_name.clone(),
        avatar: get_blob_ref(&profile.avatar),
        created_at: parse_datetime(profile.created_at),
        activity: None,
        status: None,
        viewer: None,
        labels: None,
    }
}

pub fn profile_view_basic_from_db(actor: &Actor, profile: &SchemaProfile) -> ProfileViewBasic {
    ProfileViewBasic {
        did: actor.did.clone(),
        handle: match &actor.handle {
            Some(handle) => handle.clone(),
            None => "handle.invalid".to_string()
        },
        tagline: profile.tagline.clone(),
        display_name: profile.display_name.clone(),
        avatar: get_blob_ref(&avatar_from_cid(profile.avatar_cid.clone())),
        created_at: profile.created_at.clone(),
        activity: None,
        status: None,
        viewer: None,
        labels: None,
    }
}

pub fn profile_view_detailed_from_db(actor: &Actor, profile: &SchemaProfile) -> ProfileViewDetailed {
    ProfileViewDetailed {
        did: actor.did.clone(),
        handle: match &actor.handle {
            Some(handle) => handle.clone(),
            None => "handle.invalid".to_string()
        },

        display_name: profile.display_name.clone(),
        avatar: get_blob_ref(&avatar_from_cid(profile.avatar_cid.clone())),
        banner: get_blob_ref(&avatar_from_cid(profile.banner_cid.clone())),

        tagline: profile.tagline.clone(),
        description: profile.description.clone(),
        location: profile.location.clone(),

        labels: vec![],
        social_connections: None,
        status: None,
        viewer: None,
        activities: vec![],

        indexed_at: Some(profile.indexed_at.clone()),
        created_at: profile.created_at.clone(),
    }
}

pub fn profile_view_basic_deleted_actor(did: String) -> ProfileViewBasic {
    ProfileViewBasic {
        did: did.clone(),
        handle: "null".to_string(),
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

pub fn profile_view_basic_deleted_profile(actor: &Actor) -> ProfileViewBasic {
    ProfileViewBasic {
        did: actor.did.clone(),
        handle: actor.handle.clone().unwrap_or("null".to_string()),

        display_name: actor.handle.clone(),
        avatar: None,

        tagline: None,
        activity: None,
        status: None,
        viewer: None,
        labels: None,

        created_at: Some(actor.indexed_at.clone()),
    }
}

pub fn profile_view(actor: &Actor, profile: &Profile) -> ProfileView {
    ProfileView {
        did: actor.did.clone(),
        handle: match &actor.handle {
            Some(handle) => handle.clone(),
            None => "handle.invalid".to_string()
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

pub fn profile_view_detailed(actor: &Actor, profile: &Profile) -> ProfileViewDetailed {
    ProfileViewDetailed {
        did: actor.did.clone(),
        handle: match &actor.handle {
            Some(handle) => handle.clone(),
            None => "handle.invalid".to_string()
        },

        display_name: profile.display_name.clone(),
        avatar: get_blob_ref(&profile.avatar),
        banner: get_blob_ref(&profile.banner),

        tagline: profile.tagline.clone(),
        description: profile.description.clone(),

        activities: vec![],
        labels: vec![],
        status: None,
        viewer: None,
        social_connections: None,
        location: None,

        indexed_at: Some(actor.indexed_at.clone()),
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

pub fn profile_record(profile: SchemaProfile) -> Profile {
    return Profile {
        display_name: profile.display_name,
        description: profile.description,
        avatar: avatar_from_cid(profile.avatar_cid),
        banner: avatar_from_cid(profile.banner_cid),
        tagline: profile.tagline,
        location: profile.location,
        social_connections: None,
        labels: None,
        created_at: match profile.created_at {
            Some(datetime) => DateTime::from_str(&datetime).ok(),
            None => None,
        }
    };
}