use campground_lexicon::gg::campground::actor::{Profile, ProfileView};
use rsky_identity::types::DidDocument;

use super::{did::get_handle, repo::get_blob_ref};

pub fn profile_view(did_doc: &DidDocument, profile: &Profile) -> ProfileView {
    ProfileView {
        did: did_doc.id.clone(),
        handle: get_handle(did_doc).unwrap(),
        display_name: profile.display_name.clone(),
        avatar: get_blob_ref(&profile.avatar),
        description: profile.description.clone(),
        activities: vec![],
        tagline: profile.tagline.clone(),
        labels: vec![],
        indexed_at: None,
        status: None,
    }
}