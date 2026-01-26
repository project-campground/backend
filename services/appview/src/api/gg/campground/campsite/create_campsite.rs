use appview_schema::{models::appview::{Bonfire, Campsite, CampsiteMember, CampsiteRole, Tent}, schema::appview::actor};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::campsite::CreateCampsiteOutput;
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{serde::json::Json,State};
use reqwest::Client;
use rsky_common::tid::Ticker;
use serde::Deserialize;
use uuid::Uuid;

use crate::{database::{establish_connection, profiles::get_profile}, helpers::{campsites::{bonfire_view_basic, campsite_member_view_basic, campsite_role_view_basic, campsite_view_detailed}, roles::CampsiteRoleFlag, tents::tent_view_basic}, xrpc::{
    auth::Authorization,
    error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct CreateCampsiteBody {
    name: String,
    description: String,
    vanity_url: Option<String>,
    tags: Option<Vec<String>>,
}

#[post("/xrpc/gg.campground.campsite.createCampsite", data = "<body>")]
pub async fn create_campsite(auth: Authorization<'_>, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, body: Json<CreateCampsiteBody>) -> Result<Json<CreateCampsiteOutput>> {    
    let inner_body = &body.into_inner();
    let vanity_url = &inner_body.vanity_url.clone();
    if inner_body.name.len() < 3 || inner_body.name.len() > 48 {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string()));
    } else if inner_body.description.len() > 200 {
        return Err(XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string()));
    } else if vanity_url.clone().map_or(false, |x| x.len() < 1 || x.len() > 32) {
        return Err(XRPCError::BadRequest("Expected 'vanity_url' property to have a string of length 3 to 32 characters".to_string()));
    } else if inner_body.tags.clone().map_or(false, |x| x.len() > 10 || x.iter().any(|y| y.len() > 20)) {
        return Err(XRPCError::BadRequest("Expected 'tags' property to have up to 10 values and value to be a string of length up to 20 characters".to_string()));
    }

    let mut conn = establish_connection().unwrap();
    let (actor, profile) = &get_profile(client, did_document_storage, auth.actor_did.clone().as_str())
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    let existing_campsite_count = crate::schema::appview::campsite::table
        .filter(crate::schema::appview::campsite::owner.eq(actor.did.clone()))
        .execute(&mut conn)
        .expect("Error loading owner's campsites");
    
    if existing_campsite_count >= 20 {
        return Err(XRPCError::Forbidden("Cannot create more than 20 campsites".to_string()));
    }

    if vanity_url.clone().map_or(false, |x| {
        let existing_vanity_count = crate::schema::appview::campsite::table
            .filter(crate::schema::appview::campsite::vanityurl.eq(x))
            .execute(&mut conn)
            .expect("Error loading other campsites");
        existing_vanity_count > 0
    }) {
        return Err(XRPCError::Forbidden("Cannot use that 'vanity_url', as a campsite is already using it".to_string()));
    }

    let mut ticker = Ticker::new();
    let current_date = Utc::now().naive_utc();
    let campsite_id = ticker.next(None);

    let campsite = &diesel::insert_into(crate::schema::appview::campsite::table)
        .values(
            Campsite {
                id: campsite_id.to_string(),
                name: inner_body.name.clone(),
                vanity_url: vanity_url.clone(),
                description: inner_body.description.clone(),
                avatar_uri: None,
                banner_uri: None,
                tags: inner_body.tags.clone().map_or(vec![], |x| x.iter().map(|y| Some(y.clone())).collect()),
                member_dids: vec![Some(actor.did.clone())],
                owner: actor.did.clone(),
                created_by: actor.did.clone(),
                created_at: current_date,
                updated_by: actor.did.clone(),
                updated_at: current_date,
            }
        )
        .get_result::<Campsite>(&mut conn)
        .expect("Error inserting campsite");
    let default_role = &diesel::insert_into(crate::schema::appview::campsite_role::table)
        .values(
            CampsiteRole {
                id: Uuid::new_v4(),
                campsite_id: campsite.id.clone(),
                name: "Member".to_string(),
                display_separately: false,
                mentionable: false,
                color: 0,
                color_secondary: 0,
                campsite_permissions: 0b00,
                tent_permissions: 0b11,
                priority: 0,
                created_by: actor.did.clone(),
                created_at: current_date,
                updated_by: actor.did.clone(),
                updated_at: current_date,
                flags: CampsiteRoleFlag::DEFAULT_ROLE,
                members: vec![Some(actor.did.clone())]
            }
        )
        .get_result::<CampsiteRole>(&mut conn)
        .expect("Error inserting default role");
    let owner_member = &diesel::insert_into(crate::schema::appview::campsite_member::table)
        .values(
            CampsiteMember {
                user_id: actor.did.clone(),
                campsite_id: campsite_id.to_string(),
                joined_at: current_date,
                nickname: None,
                used_invite_id: None,
                roles: vec![Some(default_role.id)],
            }
        )
        .get_result::<CampsiteMember>(&mut conn)
        .expect("Error inserting owner membership");

    // To make sure the user sees the new campsite they created
    let mut new_campsite_list = actor.campsites.clone();
    new_campsite_list.push(Some(campsite.id.clone()));

    diesel::update(actor::table)
        .filter(
            actor::did.eq(auth.actor_did)
        )
        .set(
            actor::campsites.eq(new_campsite_list)
        )
        .execute(&mut conn)
        .expect("Error updating actor campsite list");

    let home_bonfire = &diesel::insert_into(crate::schema::appview::bonfire::table)
        .values(
            Bonfire {
                id: ticker.next(Some(campsite_id.clone())).to_string(),
                campsite_id: campsite_id.to_string(),
                name: campsite.name.clone(),
                description: "".to_string(),
                avatar_uri: None,
                banner_uri: None,
                priority: 0,
                created_by: actor.did.clone(),
                created_at: current_date,
                updated_by: actor.did.clone(),
                updated_at: current_date
            }
        )
        .get_result::<Bonfire>(&mut conn)
        .expect("Error inserting default bonfire");
    let general_tent = &diesel::insert_into(crate::schema::appview::tent::table)
        .values(
            Tent {
                id: Uuid::new_v4(),
                campsite_id: campsite.id.clone(),
                bonfire_id: home_bonfire.id.clone(),
                category_id: None,
                name: "General".to_string(),
                r#type: 0,
                view_type: 0,
                description: home_bonfire.description.clone(),
                priority: 0,
                created_by: actor.did.clone(),
                created_at: current_date,
                updated_by: actor.did.clone(),
                updated_at: current_date,
            }
        )
        .get_result::<Tent>(&mut conn)
        .expect("Error inserting default tent");

    let campsite_view = campsite_view_detailed(campsite, vec![ bonfire_view_basic(home_bonfire) ], vec![ campsite_role_view_basic(default_role) ], campsite_member_view_basic(owner_member, profile, actor));

    return Ok(Json(CreateCampsiteOutput { campsite: campsite_view, default_tent: tent_view_basic(general_tent) }));
}