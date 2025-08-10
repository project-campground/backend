#![
    allow(dead_code)
]

use appview_schema::models::appview::{Account, Actor};
use campground_lexicon::gg::campground::{activity::Activity, actor::{PrivateProfileView, Profile, ProfileStatus, ProfileView, ProfileViewBasic, ProfileViewDetailed}, socials::SocialConnection};
use chrono::{DateTime, Utc};
use common::get_blob_ref;

pub fn parse_datetime(datetime: Option<DateTime<Utc>>) -> Option<String> {
    match datetime {
        Some(dt) => Some(dt.to_rfc3339()),
        None => None,
    }
}

pub fn profile_view_basic(actor: &Actor, profile: &Profile) -> ProfileViewBasic {
    ProfileViewBasic {
        did: actor.did.clone(),
        handle: match &actor.handle {
            Some(handle) => handle.clone(),
            None => "handle.invalid".to_string()
        },
        display_name: profile.display_name.clone(),
        avatar: get_blob_ref(&profile.avatar),
        created_at: parse_datetime(profile.created_at),
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
            None => "handle.invalid".to_string()
        },
        display_name: profile.display_name.clone(),
        avatar: get_blob_ref(&profile.avatar),
        banner: get_blob_ref(&profile.banner),
        description: profile.description.clone(),
        activities: vec![],
        tagline: profile.tagline.clone(),
        labels: vec![],
        indexed_at: None,
        created_at: parse_datetime(profile.created_at),
        status: None,
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
        description: profile.description.clone(),
        activities: vec![],
        tagline: profile.tagline.clone(),
        labels: vec![],
        indexed_at: None,
        created_at: parse_datetime(profile.created_at),
        status: None,
        viewer: None,
        social_connections: None,
        location: None,
    }
}

pub fn private_profile_view(account: &Account) -> PrivateProfileView {
    PrivateProfileView {
        did: account.did.clone(),
        status: Some(match account.status.as_str() {
            "online" => ProfileStatus::Online,
            "offline" => ProfileStatus::Offline,
            "donotdisturb" => ProfileStatus::DoNotDisturb,
            "idle" => ProfileStatus::Idle,
            _ => ProfileStatus::Offline
        }),
        status_text: account.status_text.clone(),
        status_emoji: account.status_emoji.clone(),
        activities: match serde_json::from_value::<Vec<Activity>>(account.activities.clone()) {
            Ok(activities) => activities,
            Err(_) => vec![]
        },
        social_connections: match serde_json::from_value::<Vec<SocialConnection>>(account.social_connections.clone()) {
            Ok(social_connections) => social_connections,
            Err(_) => vec![]
        },
    }
}