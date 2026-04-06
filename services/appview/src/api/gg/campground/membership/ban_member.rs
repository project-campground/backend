use appview_schema::{
    models::appview::{Actor, Campsite, CampsiteBan, CampsiteMember, CampsiteRole, Profile},
    schema::appview::{self, campsite_member, campsite_role, profile},
};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::membership::MemberBanView;
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use reqwest::Client;
use rocket::{State, serde::json::Json};
use serde::Deserialize;

use crate::{
    api::gg::campground::membership::remove_member::{
        ensure_user_isnt_higher, remove_campsite_member,
    },
    database::{actors::get_actor, establish_connection},
    expect_permission,
    helpers::{
        api::handle_select_first_error,
        permissions::{GeneralPermissionConsts, has_role_perms_or_owner},
        ws::event_next_campsite,
    },
    realtime::data::ReactiveSubject,
    views::members::member_ban_view,
    xrpc::{
        campsite::CampsiteInfo,
        error::{Result, XRPCError},
    },
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateBanBody {
    reason: Option<String>,
}

#[post(
    "/xrpc/gg.campground.membership.banMember?<campsite_id>&<actor>",
    data = "<body>"
)]
pub async fn ban_member(
    auth: CampsiteInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    client: &State<Client>,
    did_document_storage: &State<LruDidDocumentStorage>,
    campsite_id: &str,
    actor: &str,
    body: Json<CreateBanBody>,
) -> Result<Json<MemberBanView>> {
    if actor == auth.actor.did {
        return Err(XRPCError::Forbidden(
            "Member cannot ban themselves".to_string(),
        ));
    }

    let inner_body = &body.into_inner();
    if inner_body.reason.clone().map_or(false, |x| x.len() > 200) {
        return Err(XRPCError::BadRequest(
            "Expected 'description' property to have a string of up to 200 characters".to_string(),
        ));
    }

    expect_permission!(has_role_perms_or_owner(
        &auth.campsite,
        &auth.member,
        GeneralPermissionConsts::BAN_MEMBERS,
        0
    ));

    let mut conn = establish_connection().unwrap();

    // Make sure the role exists
    let all_roles = campsite_role::table
        .filter(campsite_role::campsiteid.eq(campsite_id))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;

    let target_actor = &get_actor(client, did_document_storage, actor)
        .await
        .map_err(|_| XRPCError::NotFound)?;

    let targets = profile::table
        .filter(profile::creator.eq(&target_actor.did))
        .left_join(
            campsite_member::table.on(profile::creator
                .eq(campsite_member::userid)
                .and(campsite_member::campsiteid.eq(campsite_id))),
        )
        .load::<(Profile, Option<CampsiteMember>)>(&mut conn)
        .map_err(handle_select_first_error)?;

    if targets.len() < 1 {
        return add_ban(
            event_subject,
            &auth.actor,
            &auth.campsite,
            target_actor,
            &inner_body.reason,
            &None,
        );
    }

    let target = &targets.first().unwrap().clone();

    if let Some(member) = target.1.clone() {
        ensure_user_isnt_higher(
            auth.campsite.owner == auth.actor.did,
            &mut all_roles.clone(),
            &member.roles,
            auth.member.roles.clone(),
        )?;
        remove_campsite_member(
            event_subject,
            &auth.campsite.id,
            &member,
            &target.0,
            &target_actor,
            actor,
        )?;
    }

    add_ban(
        event_subject,
        &auth.actor,
        &auth.campsite,
        target_actor,
        &inner_body.reason,
        &Some(target.0.clone()),
    )
}

fn add_ban(
    event_subject: &State<ReactiveSubject>,
    executor: &Actor,
    campsite: &Campsite,
    actor: &Actor,
    reason: &Option<String>,
    profile: &Option<Profile>,
) -> Result<Json<MemberBanView>, XRPCError> {
    let mut conn = establish_connection().unwrap();

    let current_date = Utc::now().naive_utc();
    let ban = &diesel::insert_into(appview::campsite_ban::table)
        .values(CampsiteBan {
            user_id: actor.did.clone(),
            campsite_id: campsite.id.clone(),
            reason: reason.clone(),
            created_at: current_date,
            created_by: executor.did.clone(),
            updated_at: current_date,
            updated_by: executor.did.clone(),
        })
        .load::<CampsiteBan>(&mut conn)
        .map_err(handle_select_first_error)?;

    let ban = ban.first().unwrap();

    event_next_campsite(
        event_subject,
        &campsite.id,
        GeneralPermissionConsts::BAN_MEMBERS,
        "MemberBanCreated",
        member_ban_view(ban, profile, actor),
    );

    Ok(Json(member_ban_view(ban, profile, actor)))
}
