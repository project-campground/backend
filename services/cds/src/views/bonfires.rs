use campground_lexicon::chat::campground::bonfire::{BonfireViewBasic, BonfireViewDetailed};
use cds_schema::models::cds::Bonfire;

use crate::views::util::serialize_datetime;

pub fn bonfire_view_basic(bonfire: &Bonfire) -> BonfireViewBasic {
    return BonfireViewBasic {
        id: bonfire.id.clone(),
        campsite_id: bonfire.campsite.clone(),
        name: bonfire.name.clone(),
        description: bonfire.description.clone(),
        avatar_uri: bonfire.avatar.clone(),
        banner_uri: bonfire.banner.clone(),
        position: bonfire.priority,
        home: bonfire.home,
    };
}
pub fn bonfire_view_detailed(bonfire: &Bonfire) -> BonfireViewDetailed {
    return BonfireViewDetailed {
        id: bonfire.id.clone(),
        campsite_id: bonfire.campsite.clone(),
        name: bonfire.name.clone(),
        description: bonfire.description.clone(),
        avatar_uri: bonfire.avatar.clone(),
        banner_uri: bonfire.banner.clone(),
        priority: bonfire.priority,
        home: bonfire.home,
        created_by: bonfire.created_by.clone(),
        created_at: serialize_datetime(bonfire.created_at),
        updated_by: bonfire.updated_by.clone(),
        updated_at: serialize_datetime(bonfire.updated_at),
    };
}
