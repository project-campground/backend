use appview_schema::models::appview::{Bonfire, Tent, TentMessage};
use campground_lexicon::gg::campground::{content::{ContentComponent, SystemMessage}, tent::TentViewBasic};
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{database::{establish_connection, profiles::get_profile_from_actor}, helpers::{api::handle_select_first_error, permissions::{CampsitePermissionConsts, has_tent_perms_or_owner}, tents::{tent_message_view_basic, tent_view_basic}, ws::{event_next_campsite, event_next_tent}}, realtime::data::ReactiveSubject, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct CreateTentBody {
    name: String,
    description: String,
    category_id: Option<String>,
    r#type: i16,
    view_type: i16,
    priority: i32,
}

#[post("/xrpc/gg.campground.tent.createTent?<campsite_id>&<bonfire_id>", data = "<body>")]
pub async fn create_tent(auth: CampsiteInfo<'_>, event_subject: &State<ReactiveSubject>, campsite_id: &str, bonfire_id: &str, body: Json<CreateTentBody>) -> Result<Json<TentViewBasic>> {    
    let inner_body = &body.into_inner();
    if inner_body.name.len() < 3 || inner_body.name.len() > 48 {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string()));
    } else if inner_body.description.len() > 200 {
        return Err(XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string()));
    } else if inner_body.r#type != 0 {
        return Err(XRPCError::BadRequest("Invalid type. Expected only 0, as other types do not exist yet.".to_string()))
    } else if inner_body.view_type != 0 {
        return Err(XRPCError::BadRequest("Invalid view type. Expected only 0, as for this type other view types do not exist.".to_string()));
    }

    let category_id =
        if inner_body.category_id.is_none() {
            None
        } else {
            Some(Uuid::try_parse(inner_body.category_id.clone().unwrap().as_str())
                .map_err(|_| XRPCError::BadRequest("Invalid category_id UUID format".to_string()))?)
        };

    let mut conn = establish_connection().unwrap();

    crate::schema::appview::bonfire::table
        .filter(
            crate::schema::appview::bonfire::id
                .eq(bonfire_id)
                .and(
                    crate::schema::appview::bonfire::campsiteid
                        .eq(campsite_id)
                )
        )
        .first::<Bonfire>(&mut conn)
        .map_err(handle_select_first_error)?;

    if !has_tent_perms_or_owner(&auth.campsite, &bonfire_id, category_id.clone(), None, &auth.member, CampsitePermissionConsts::MANAGE_TENTS, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let existing_tent_count = crate::schema::appview::tent::table
        .filter(
            crate::schema::appview::tent::campsiteid
                .eq(campsite_id)
        )
        .count()
        .first::<i64>(&mut conn)
        .expect("Error loading bonfires");
    
    if existing_tent_count >= 500 {
        return Err(XRPCError::Forbidden("Cannot create more than 500 tents in a campsite".to_string()));
    }

    let current_date = Utc::now().naive_utc();

    let tents = &diesel::insert_into(crate::schema::appview::tent::table)
        .values(
            Tent {
                id: Uuid::new_v4(),
                campsite_id: campsite_id.to_string(),
                bonfire_id: bonfire_id.to_string(),
                category_id: category_id,
                name: inner_body.name.clone(),
                description: inner_body.description.clone(),
                r#type: inner_body.r#type,
                view_type: inner_body.view_type,
                priority: inner_body.priority,
                created_by: auth.actor.did.clone(),
                created_at: current_date,
                updated_by: auth.actor.did.clone(),
                updated_at: current_date,
            }
        )
        .load::<Tent>(&mut conn)
        .map_err(handle_select_first_error)?;
    
    let first_tent = tents.first().unwrap();
    
    let first_message = &diesel::insert_into(crate::schema::appview::tent_message::table)
        .values(
            TentMessage {
                id: Uuid::new_v4(),
                campsite_id: campsite_id.to_string(),
                tent_id: first_tent.id.clone(),
                content: "".to_string(),
                r#type: 1,
                replying_to: vec![],
                components: vec![
                    serde_json::value::to_value(
                        ContentComponent::System(
                            SystemMessage::TentCreated {
                                tent_name: inner_body.name.clone(),
                            }
                        )
                    ).ok()
                ],
                created_by: auth.actor.did.clone(),
                created_at: current_date,
                updated_at: None,
            }
        )
        .load::<TentMessage>(&mut conn)
        .map_err(handle_select_first_error)?;
    let first_message = first_message.first().unwrap();

    let (_, profile) = get_profile_from_actor(&auth.auth.client, &auth.auth.did_document_storage, auth.actor.clone())
        .await
        .map_err(|_| XRPCError::InternalServerError)?;

    event_next_tent(event_subject, &auth.campsite.id, &first_tent.bonfire_id, first_tent.category_id, first_tent.id, false, "TentCreated", tent_view_basic(&first_tent));
    event_next_campsite(event_subject, &auth.campsite.id, "MessageCreated", tent_message_view_basic(&first_tent, &first_message, &Some(auth.actor), &Some(profile), &Some(auth.member)));

    let tent_view = tent_view_basic(&first_tent);

    return Ok(Json(tent_view));
}