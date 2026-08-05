use appview_schema::models::appview::{Actor, CampsiteMember, Profile, Tent, TentMessage};
use campground_lexicon::gg::campground::{
    content::ContentComponent,
    message::{MessageViewBasic, MessageViewWithReplies},
};
use serde_json::from_value;
use uuid::Uuid;

use crate::views::{members::created_by_view, util::serialize_datetime};

pub fn message_view_basic(
    tent: &Tent,
    message: &TentMessage,
    actor: Option<&Actor>,
    profile: Option<&Profile>,
    member: Option<&CampsiteMember>,
) -> MessageViewBasic {
    return MessageViewBasic {
        id: message.id,
        campsite_id: message.campsite_id.clone(),
        bonfire_id: tent.bonfire_id.clone(),
        tent_id: message.tent_id,

        r#type: match message.r#type {
            1 => Some(campground_lexicon::gg::campground::message::MessageType::System),
            _ => None,
        },
        content: message.content.clone(),
        components: if message.components.len() < 1 {
            None
        } else {
            Some(
                message
                    .components
                    .clone()
                    .iter()
                    .filter_map(|x| {
                        x.clone()
                            .map(|value| from_value::<ContentComponent>(value).ok())
                            .flatten()
                    })
                    .collect::<Vec<ContentComponent>>(),
            )
        },
        replying_to: message
            .replying_to
            .iter()
            .filter_map(|x| x.clone())
            .collect::<Vec<Uuid>>(),

        created_by: created_by_view(&message.created_by, actor, profile, member),
        created_at: serialize_datetime(message.created_at),
        updated_at: message.updated_at.map(|x| serialize_datetime(x)),
    };
}

pub fn message_view_with_replies(
    tent: &Tent,
    message: &TentMessage,
    replies: Vec<MessageViewBasic>,
    actor: Option<&Actor>,
    profile: Option<&Profile>,
    member: Option<&CampsiteMember>,
) -> MessageViewWithReplies {
    return MessageViewWithReplies {
        id: message.id,
        campsite_id: message.campsite_id.clone(),
        bonfire_id: tent.bonfire_id.clone(),
        tent_id: message.tent_id,
        r#type: match message.r#type {
            1 => Some(campground_lexicon::gg::campground::message::MessageType::System),
            _ => None,
        },
        content: message.content.clone(),
        replying_to: replies,
        replying_to_count: message.replying_to.len(),
        components: if message.components.len() < 1 {
            None
        } else {
            Some(
                message
                    .components
                    .clone()
                    .iter()
                    .filter_map(|x| {
                        x.clone()
                            .map(|value| from_value::<ContentComponent>(value).ok())
                            .flatten()
                    })
                    .collect::<Vec<ContentComponent>>(),
            )
        },
        created_by: created_by_view(&message.created_by, actor, profile, member),
        created_at: serialize_datetime(message.created_at),
        updated_at: message.updated_at.map(|x| serialize_datetime(x)),
    };
}
