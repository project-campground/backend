use appview_schema::models::appview::{Tent, TentCategory};
use campground_lexicon::gg::campground::tent::{
    TentCategoryView, TentType, TentViewBasic, TentViewDetailed,
};

use crate::views::util::serialize_datetime;

pub fn tent_category_view(category: &TentCategory) -> TentCategoryView {
    return TentCategoryView {
        id: category.id.clone(),
        campsite_id: category.campsite_id.clone(),
        bonfire_id: category.bonfire_id.clone(),
        name: category.name.clone(),
        description: category.description.clone(),
        position: category.priority,
        created_by: category.created_by.clone(),
        created_at: serialize_datetime(category.created_at),
        updated_by: category.updated_by.clone(),
        updated_at: serialize_datetime(category.updated_at),
    };
}

pub fn tent_view_basic(tent: &Tent) -> TentViewBasic {
    return TentViewBasic {
        id: tent.id.clone(),
        campsite_id: tent.campsite_id.clone(),
        bonfire_id: tent.bonfire_id.clone(),
        category_id: tent.category_id,
        name: tent.name.clone(),
        description: tent.description.clone(),
        r#type: match tent.r#type {
            _ => TentType::Text,
        },
        view_type: tent.view_type,
        position: tent.priority,
    };
}

pub fn tent_view_detailed(tent: &Tent) -> TentViewDetailed {
    return TentViewDetailed {
        id: tent.id.clone(),
        campsite_id: tent.campsite_id.clone(),
        bonfire_id: tent.bonfire_id.clone(),
        category_id: tent.category_id,
        name: tent.name.clone(),
        description: tent.description.clone(),
        r#type: match tent.r#type {
            _ => TentType::Text,
        },
        view_type: tent.view_type,
        position: tent.priority,
        created_by: tent.created_by.clone(),
        created_at: serialize_datetime(tent.created_at),
        updated_by: tent.updated_by.clone(),
        updated_at: serialize_datetime(tent.updated_at),
    };
}
