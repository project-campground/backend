use appview_schema::models::appview::{Actor, Profile, Tent, TentCategory, TentMessage};
use campground_lexicon::gg::campground::{actor::ProfileViewBasic, tent::{TentCategoryView, TentMessageViewBasic, TentMessageViewWithReplies, TentType, TentViewBasic, TentViewDetailed}};
use uuid::Uuid;

use crate::helpers::{util::serialize_datetime, views::{profile_view_basic_deleted_actor, profile_view_basic_deleted_profile, profile_view_basic_from_db}};

pub fn tent_category_view(category: &TentCategory) -> TentCategoryView {
    return TentCategoryView {
        id: category.id.clone(),
        campsite_id: category.campsite_id.clone(),
        bonfire_id: category.bonfire_id.clone(),
        name: category.name.clone(),
        description: category.description.clone(),
        priority: category.priority,
        created_by: category.created_by.clone(),
        created_at: serialize_datetime(category.created_at),
        updated_by: category.updated_by.clone(),
        updated_at: serialize_datetime(category.updated_at)
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
        priority: tent.priority,
        created_by: tent.created_by.clone(),
        created_at: serialize_datetime(tent.created_at),
        updated_by: tent.updated_by.clone(),
        updated_at: serialize_datetime(tent.updated_at)
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
        priority: tent.priority,
        created_by: tent.created_by.clone(),
        created_at: serialize_datetime(tent.created_at),
        updated_by: tent.updated_by.clone(),
        updated_at: serialize_datetime(tent.updated_at)
    };
}

fn tent_message_created_by(created_by: &String, actor: &Option<Actor>, profile: &Option<Profile>) -> ProfileViewBasic {
    match actor {
        None => profile_view_basic_deleted_actor(created_by.clone()),
        Some(x) => match profile {
            None => profile_view_basic_deleted_profile(x),
            Some(y) => profile_view_basic_from_db(x, y),
        }
    }
}

pub fn tent_message_view_basic(tent: &Tent, message: &TentMessage, actor: &Option<Actor>, profile: &Option<Profile>) -> TentMessageViewBasic {
    return TentMessageViewBasic {
        id: message.id,
        campsite_id: message.campsite_id.clone(),
        bonfire_id: tent.bonfire_id.clone(),
        tent_id: message.tent_id,
        content: message.content.clone(),
        created_by: tent_message_created_by(&message.created_by, actor, profile),
        created_at: serialize_datetime(message.created_at),
        updated_at: message.updated_at.map(|x| serialize_datetime(x)),
        replying_to: message.replying_to.iter().filter_map(|x| x.clone()).collect::<Vec<Uuid>>(),
    };
}

pub fn tent_message_view_with_replies(tent: &Tent, message: &TentMessage, replies: Vec<TentMessageViewBasic>, actor: &Option<Actor>, profile: &Option<Profile>) -> TentMessageViewWithReplies {
    return TentMessageViewWithReplies {
        id: message.id,
        campsite_id: message.campsite_id.clone(),
        bonfire_id: tent.bonfire_id.clone(),
        tent_id: message.tent_id,
        content: message.content.clone(),
        created_by: tent_message_created_by(&message.created_by, actor, profile),
        created_at: serialize_datetime(message.created_at),
        updated_at: message.updated_at.map(|x| serialize_datetime(x)),
        replying_to: replies,
        replying_to_count: message.replying_to.len()
    };
}
