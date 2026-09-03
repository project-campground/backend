use appview_schema::models::appview::TentMessage;
use campground_lexicon::gg::campground::{content::ContentComponent, message::MessageType};
use chrono::NaiveDateTime;
use uuid::Uuid;

pub fn system_message(
    campsite_id: &str,
    tent_id: &Uuid,
    author_did: &str,
    created_at: NaiveDateTime,
    component: ContentComponent,
) -> TentMessage {
    TentMessage {
        id: Uuid::new_v4(),
        campsite_id: campsite_id.to_string(),
        tent_id: tent_id.clone(),

        r#type: MessageType::Default as i16,

        components: vec![serde_json::value::to_value(component).ok()],
        // Most of content the should be empty
        content: "".to_string(),
        replying_to: vec![],

        created_by: author_did.to_string(),
        created_at,
        updated_at: None,
    }
}
