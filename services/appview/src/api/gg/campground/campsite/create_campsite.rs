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

use crate::{database::{establish_connection, profiles::get_profile}, helpers::{api::handle_all_db_errors, campsites::{bonfire_view_basic, campsite_member_view_basic, campsite_role_view_basic, campsite_view_basic, campsite_view_detailed}, roles::CampsiteRoleFlag, tents::tent_view_basic, ws::event_next}, realtime::data::{ReactiveSubject, ReactiveSubjectData}, util::params::{OptionValidity, ensure_valid_set_uri}, xrpc::{
    auth::Authorization,
    error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct CreateCampsiteBody<'a> {
    name: String,
    description: Option<String>,
    vanity_url: Option<String>,
    tags: Option<Vec<String>>,
    avatar_uri: Option<&'a str>,
    banner_uri: Option<&'a str>,
}

#[post("/xrpc/gg.campground.campsite.createCampsite", data = "<body>")]
pub async fn create_campsite(auth: Authorization<'_>, event_subject: &State<ReactiveSubject>, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, body: Json<CreateCampsiteBody<'_>>) -> Result<Json<CreateCampsiteOutput>> {    
    let CreateCampsiteBody { name, description, vanity_url, tags, avatar_uri, banner_uri } = &body.into_inner();
    if name.len() < 3 || name.len() > 48 {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string()));

    } else if description.clone().map_or(false, |x| x.len() > 200) {
        return Err(XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string()));
    }

    let vanity_url = &vanity_url
        .clone()
        .ensure_validity(|x| x.len() <= 32)
        .map_err(|_|
            XRPCError::BadRequest("Expected 'vanity_url' property to have a string of up to 32 characters".to_string())
        )?;
    let tags = &tags
        .clone()
        .ensure_validity(|x| x.len() <= 10 && !x.iter().any(|y| y.len() > 20))
        .map_err(|_|
            XRPCError::BadRequest("Expected 'tags' property to have up to 10 values and value to be a string of length up to 20 characters".to_string())
        )?
        .map(|x|
            x
                .iter()
                .map(|x| Some(x.to_string()))
                .collect::<Vec<Option<String>>>()
        );
    let avatar_uri = &ensure_valid_set_uri(avatar_uri)
        .map_err(|x|
            XRPCError::BadRequest(x.to_string())
        )?
        .map(|x| x.to_string());
    let banner_uri = &ensure_valid_set_uri(banner_uri)
        .map_err(|x|
            XRPCError::BadRequest(x.to_string())
        )?
        .map(|x| x.to_string());

    let mut conn = establish_connection().unwrap();
    let (actor, profile) = &get_profile(client, did_document_storage, &auth.actor_did)
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    let existing_campsite_count = crate::schema::appview::campsite::table
        .filter(crate::schema::appview::campsite::owner.eq(&actor.did))
        .count()
        .first::<i64>(&mut conn)
        .expect("Error loading owner's campsites");

    if existing_campsite_count >= 20 {
        return Err(XRPCError::Forbidden("Cannot create more than 20 campsites".to_string()));
    }

    if vanity_url.clone().map_or(false, |x| {
        let existing_vanity_count = crate::schema::appview::campsite::table
            .filter(crate::schema::appview::campsite::vanityurl.eq(x))
            .count()
            .first::<i64>(&mut conn)
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
                name: name.clone(),
                vanity_url: vanity_url.clone(),
                description: description.clone().unwrap_or("".to_string()),
                avatar_uri: avatar_uri.clone(),
                banner_uri: banner_uri.clone(),
                tags: tags.clone().unwrap_or(vec![]),
                member_dids: vec![Some(actor.did.clone())],
                owner: actor.did.clone(),
                created_by: actor.did.clone(),
                created_at: current_date,
                updated_by: actor.did.clone(),
                updated_at: current_date,
            }
        )
        .get_result::<Campsite>(&mut conn)
        .map_err(handle_all_db_errors)?;
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
                general_permissions: 0b00,
                content_permissions: 0b11,
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
        .map_err(handle_all_db_errors)?;
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
        .map_err(handle_all_db_errors)?;

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
        .map_err(handle_all_db_errors)?;

    let home_bonfire = &diesel::insert_into(crate::schema::appview::bonfire::table)
        .values(
            Bonfire {
                id: ticker.next(Some(campsite_id.clone())).to_string(),
                campsite_id: campsite_id.to_string(),
                name: campsite.name.clone(),
                description: "".to_string(),
                avatar_uri: avatar_uri.clone(),
                banner_uri: banner_uri.clone(),
                priority: 0,
                created_by: actor.did.clone(),
                created_at: current_date,
                updated_by: actor.did.clone(),
                updated_at: current_date
            }
        )
        .get_result::<Bonfire>(&mut conn)
        .map_err(handle_all_db_errors)?;
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
        .map_err(handle_all_db_errors)?;

    let campsite_view = campsite_view_detailed(campsite, vec![ bonfire_view_basic(home_bonfire) ], vec![ campsite_role_view_basic(default_role) ], campsite_member_view_basic(owner_member, profile, actor));

    event_next(event_subject, "CampsiteJoined", campsite_view_basic(&campsite), |binary|
        ReactiveSubjectData::CampsiteAdded {
            campsite_id: campsite.id.clone(),
            to_actor: actor.did.clone(),
            binary,
        }
    );
    return Ok(Json(CreateCampsiteOutput { campsite: campsite_view, default_tent: tent_view_basic(general_tent) }));
}