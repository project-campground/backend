#![
    allow(unused_variables)
]
use appview_schema::{models::appview::{Actor, Campsite, CampsiteMember, Profile, Tent, TentMessage}, schema::appview};
use campground_lexicon::gg::campground::{content::{ContentComponent, SystemMessage}, tent::TentViewBasic};
use chrono::{NaiveDateTime, Utc};
use diesel::{ExpressionMethods, RunQueryDsl};
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{database::{establish_connection, profiles::get_profile_from_actor}, expect_permission, helpers::{api::handle_select_first_error, permissions::{ContentPermissionConsts, GeneralPermissionConsts, has_leveled_perms_or_owner}, tents::{tent_message_view_basic, tent_view_basic}, ws::{event_next_campsite, event_next_tent}}, realtime::data::ReactiveSubject, xrpc::{
    campsite::TentInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdateTentBody {
    name: Option<String>,
    description: Option<String>,
    view_type: Option<i16>,
}

#[post("/xrpc/gg.campground.tent.updateTent?<tent_id>", data = "<body>")]
pub async fn update_tent(auth: TentInfo<'_>, event_subject: &State<ReactiveSubject>, tent_id: &str, body: Json<UpdateTentBody>) -> Result<Json<TentViewBasic>> {    
    let inner_body = &body.into_inner();
    if inner_body.name.is_none() && inner_body.description.is_none() && inner_body.view_type.is_none() {
        return Err(XRPCError::BadRequest("Expected at least one property in the body".to_string()));
    } else if inner_body.name.clone().map_or(false, |x| x.len() < 3 || x.len() > 48) {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string()));
    } else if inner_body.description.clone().map_or(false, |x| x.len() > 200) {
        return Err(XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string()));
    } else if inner_body.view_type.clone().map_or(false, |x| x != 0) {
        return Err(XRPCError::BadRequest("Expected 'view_type' property to be 0".to_string()));
    }

    expect_permission!(
        has_leveled_perms_or_owner(&auth.campsite, &auth.tent.bonfire_id, auth.tent.category_id.clone(), Some(auth.tent.id), &auth.member, GeneralPermissionConsts::MANAGE_TENTS, ContentPermissionConsts::VIEW_CONTENT)
    );

    let (actor, profile) = get_profile_from_actor(auth.auth.client, auth.auth.did_document_storage, auth.actor)
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    let mut conn = establish_connection().unwrap();

    let current_date = Utc::now().naive_utc();
    
    let updated_tents = diesel::update(crate::schema::appview::tent::table)
        .filter(
            appview::tent::id
                .eq(
                    auth.tent.id.clone()
                )
        )
        .set((
            // All the new settings
            appview::tent::name
                .eq(inner_body.name.clone().unwrap_or(auth.tent.name.clone())),
            appview::tent::description
                .eq(inner_body.description.clone().unwrap_or(auth.tent.description.clone())),
            appview::tent::viewtype
                .eq(inner_body.view_type.clone().unwrap_or(auth.tent.view_type)),
            // Mandatory
            appview::tent::updatedat
                .eq(current_date),
            appview::tent::updatedby
                .eq(&actor.did),
        ))
        .load::<Tent>(&mut conn)
        .expect("Error updating tent");

    let updated_tent = updated_tents.first().unwrap();

    event_next_tent(event_subject, &updated_tent, false, "TentUpdated", tent_view_basic(&updated_tent));

    // Add rename message
    if inner_body.name.clone().map_or(false, |x| x != auth.tent.name) {
        create_tent_name_update_message(&auth.campsite, &auth.tent, auth.member, event_subject, updated_tent, &inner_body.name, current_date, actor, profile).await?;
    }

    return Ok(Json(tent_view_basic(updated_tent)));
}

async fn create_tent_name_update_message(auth_campsite: &Campsite, auth_tent: &Tent, auth_member: CampsiteMember, event_subject: &State<ReactiveSubject>, updated_tent: &Tent, new_name: &Option<String>, current_date: NaiveDateTime, actor: Actor, profile: Profile) -> Result<()> {
    let mut conn = establish_connection().unwrap();

    let update_message = &diesel::insert_into(crate::schema::appview::tent_message::table)
        .values(
            TentMessage {
                id: Uuid::new_v4(),
                campsite_id: auth_tent.campsite_id.clone(),
                tent_id: updated_tent.id.clone(),
                content: "".to_string(),
                r#type: 1,
                replying_to: vec![],
                components: vec![
                    serde_json::value::to_value(
                        ContentComponent::System(
                            SystemMessage::TentNameUpdated {
                                previous_name: auth_tent.name.clone(),
                                new_name: new_name.clone().unwrap(),
                            }
                        )
                    ).ok()
                ],
                created_by: actor.did.clone(),
                created_at: current_date,
                updated_at: None,
            }
        )
        .load::<TentMessage>(&mut conn)
        .map_err(handle_select_first_error)?;
    
    let update_message = update_message.first().unwrap();
    event_next_campsite(event_subject, &auth_campsite.id, 0, "MessageCreated", tent_message_view_basic(&updated_tent, &update_message, &Some(actor), &Some(profile), &Some(auth_member)));

    Ok(())
}