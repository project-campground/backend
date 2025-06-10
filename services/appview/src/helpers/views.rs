use atproto_identity::model::Document;
use campground_lexicon::gg::campground::actor::{Profile, ProfileView, ProfileViewBasic, ProfileViewDetailed};
use chrono::{DateTime, Utc};
use common::{get_handle, get_blob_ref};

pub fn parse_datetime(datetime: Option<DateTime<Utc>>) -> Option<String> {
    match datetime {
        Some(dt) => Some(dt.to_rfc3339()),
        None => None,
    }
}

pub fn profile_view_basic(did_doc: &Document, profile: &Profile) -> ProfileViewBasic {
    ProfileViewBasic {
        did: did_doc.id.clone(),
        handle: get_handle(did_doc).unwrap(),
        display_name: profile.display_name.clone(),
        avatar: get_blob_ref(&profile.avatar),
        created_at: parse_datetime(profile.created_at),
        activity: None,
        status: None,
        viewer: None,
        labels: None,
    }
}

pub fn profile_view(did_doc: &Document, profile: &Profile) -> ProfileView {
    ProfileView {
        did: did_doc.id.clone(),
        handle: get_handle(did_doc).unwrap(),
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

pub fn profile_view_detailed(did_doc: &Document, profile: &Profile) -> ProfileViewDetailed {
    ProfileViewDetailed {
        did: did_doc.id.clone(),
        handle: get_handle(did_doc).unwrap(),
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