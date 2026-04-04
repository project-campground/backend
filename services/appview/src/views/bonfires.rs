use appview_schema::models::appview::Bonfire;
use campground_lexicon::gg::campground::{
    bonfire::{BonfireViewBasic, BonfireViewDetailed},
    tent::{TentCategoryView, TentViewBasic},
};

use crate::views::util::serialize_datetime;

pub fn bonfire_view_basic(bonfire: &Bonfire) -> BonfireViewBasic {
    return BonfireViewBasic {
        id: bonfire.id.clone(),
        campsite_id: bonfire.campsite_id.clone(),
        name: bonfire.name.clone(),
        description: bonfire.description.clone(),
        avatar_uri: bonfire.avatar_uri.clone(),
        banner_uri: bonfire.banner_uri.clone(),
        position: bonfire.priority,
        home: bonfire.home,
    };
}
pub fn bonfire_view_detailed(
    bonfire: &Bonfire,
    tents: Vec<TentViewBasic>,
    categories: Vec<TentCategoryView>,
) -> BonfireViewDetailed {
    return BonfireViewDetailed {
        id: bonfire.id.clone(),
        campsite_id: bonfire.campsite_id.clone(),
        name: bonfire.name.clone(),
        description: bonfire.description.clone(),
        avatar_uri: bonfire.avatar_uri.clone(),
        banner_uri: bonfire.banner_uri.clone(),
        priority: bonfire.priority,
        home: bonfire.home,
        created_by: bonfire.created_by.clone(),
        created_at: serialize_datetime(bonfire.created_at),
        updated_by: bonfire.updated_by.clone(),
        updated_at: serialize_datetime(bonfire.updated_at),
        tents,
        categories,
    };
}
