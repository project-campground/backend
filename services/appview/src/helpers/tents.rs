use appview_schema::models::appview::{Actor, CampsiteMember, Profile, Tent, TentCategory, TentMessage};
use campground_lexicon::gg::campground::{content::ContentComponent, membership::CampsiteMemberViewAuthor, tent::{TentCategoryView, TentMessageViewBasic, TentMessageViewWithReplies, TentType, TentViewBasic, TentViewDetailed}};
use serde_json::from_value;
use uuid::Uuid;

use crate::helpers::{campsites::campsite_member_view_author, util::serialize_datetime, views::{profile_view_basic_deleted_actor, profile_view_basic_deleted_profile, profile_view_basic_from_db}};

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
        updated_at: serialize_datetime(tent.updated_at),
    };
}

fn created_by_view(created_by: &String, actor: &Option<Actor>, profile: &Option<Profile>, member: &Option<CampsiteMember>) -> CampsiteMemberViewAuthor {
    let profile_view = match actor {
        None => profile_view_basic_deleted_actor(created_by.clone()),
        Some(x) => match profile {
            None => profile_view_basic_deleted_profile(x),
            Some(y) => profile_view_basic_from_db(x, y),
        }
    };

    campsite_member_view_author(member, profile_view)
}

pub fn tent_message_view_basic(tent: &Tent, message: &TentMessage, actor: &Option<Actor>, profile: &Option<Profile>, member: &Option<CampsiteMember>) -> TentMessageViewBasic {
    return TentMessageViewBasic {
        id: message.id,
        campsite_id: message.campsite_id.clone(),
        bonfire_id: tent.bonfire_id.clone(),
        tent_id: message.tent_id,
    
        r#type: match message.r#type {
            1 => Some(campground_lexicon::gg::campground::tent::MessageType::System),
            _ => None,
        },
        content: message.content.clone(),
        components:
            if message.components.len() < 1 { None }
            else {
                Some(
                    message.components
                        .clone()
                        .iter()
                        .filter_map(|x|
                            x.clone().map(|value| from_value::<ContentComponent>(value).ok()).flatten()
                        )
                        .collect::<Vec<ContentComponent>>()
                )
            },
        replying_to: message.replying_to.iter().filter_map(|x| x.clone()).collect::<Vec<Uuid>>(),

        created_by: created_by_view(&message.created_by, actor, profile, member),
        created_at: serialize_datetime(message.created_at),
        updated_at: message.updated_at.map(|x| serialize_datetime(x)),
    };
}

pub fn tent_message_view_with_replies(tent: &Tent, message: &TentMessage, replies: Vec<TentMessageViewBasic>, actor: &Option<Actor>, profile: &Option<Profile>, member: &Option<CampsiteMember>) -> TentMessageViewWithReplies {
    return TentMessageViewWithReplies {
        id: message.id,
        campsite_id: message.campsite_id.clone(),
        bonfire_id: tent.bonfire_id.clone(),
        tent_id: message.tent_id,
        r#type: match message.r#type {
            1 => Some(campground_lexicon::gg::campground::tent::MessageType::System),
            _ => None,
        },
        content: message.content.clone(),
        replying_to: replies,
        replying_to_count: message.replying_to.len(),
        components:
            if message.components.len() < 1 { None }
            else {
                Some(
                    message.components
                        .clone()
                        .iter()
                        .filter_map(|x|
                            x.clone().map(|value| from_value::<ContentComponent>(value).ok()).flatten()
                        )
                        .collect::<Vec<ContentComponent>>()
                )
            },
        created_by: created_by_view(&message.created_by, actor, profile, member),
        created_at: serialize_datetime(message.created_at),
        updated_at: message.updated_at.map(|x| serialize_datetime(x)),
    };
}
