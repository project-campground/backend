use appview_schema::{
    models::appview::{Actor, CampsiteMember, CampsiteRole, Profile},
    schema::appview::{self, campsite_member, campsite_role, profile},
};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::membership::CampsiteLeftOutput;
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use diesel::{Connection, pg::expression::dsl::array_remove};
use reqwest::Client;
use rocket::State;
use uuid::Uuid;

use crate::{
    database::{DbConnection, establish_connection, profiles::get_profile_from_actor},
    expect_permission,
    helpers::{
        api::{handle_all_db_errors, handle_select_first_error},
        permissions::{GeneralPermissionConsts, has_role_perms_or_owner},
        ws::{event_next, event_next_campsite},
    },
    realtime::data::{ReactiveSubject, ReactiveSubjectData},
    views::members::member_view_basic,
    xrpc::{
        campsite::CampsiteInfo,
        error::{Result, XRPCError},
    },
};

#[post("/xrpc/gg.campground.membership.removeMember?<campsite_id>&<actor>")]
pub async fn remove_member(
    auth: CampsiteInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    client: &State<Client>,
    did_document_storage: &State<LruDidDocumentStorage>,
    campsite_id: &str,
    actor: &str,
) -> Result<()> {
    if actor == "" || actor == auth.actor.did {
        return remove_self(
            auth,
            event_subject,
            client,
            did_document_storage,
            campsite_id,
        )
        .await;
    }

    expect_permission!(has_role_perms_or_owner(
        &auth.campsite,
        &auth.member,
        GeneralPermissionConsts::KICK_MEMBERS,
        0
    ));

    let mut conn = establish_connection().unwrap();

    // Make sure the role exists
    let all_roles = campsite_role::table
        .filter(campsite_role::campsiteid.eq(campsite_id))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;

    let target = campsite_member::table
        .filter(
            campsite_member::campsiteid
                .eq(campsite_id)
                .and(campsite_member::userid.eq(&auth.actor.did)),
        )
        .inner_join(
            crate::schema::appview::actor::table
                .on(crate::schema::appview::actor::did.eq(campsite_member::userid)),
        )
        .left_join(profile::table.on(profile::creator.eq(campsite_member::userid)))
        .first::<(CampsiteMember, Actor, Option<Profile>)>(&mut conn)
        .map_err(handle_select_first_error)?;

    ensure_user_isnt_higher(
        auth.campsite.owner == auth.actor.did,
        &mut all_roles.clone(),
        &target.0.roles,
        auth.member.roles.clone(),
    )?;

    conn.transaction(|conn| {
        remove_campsite_member(
            conn,
            event_subject,
            campsite_id,
            &target.0,
            target.2.as_ref(),
            &target.1,
            actor,
        )
    })
    .map_err(handle_all_db_errors)?;

    Ok(())
}

#[post("/xrpc/gg.campground.membership.removeMember?<campsite_id>")]
pub async fn remove_self(
    auth: CampsiteInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    client: &State<Client>,
    did_document_storage: &State<LruDidDocumentStorage>,
    campsite_id: &str,
) -> Result<()> {
    let (actor, profile) = get_profile_from_actor(client, did_document_storage, auth.actor)
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    if auth.campsite.owner == actor.did {
        return Err(XRPCError::Forbidden(
            "Owner cannot leave the server without deleting it".to_string(),
        ));
    }

    let mut conn = establish_connection().unwrap();

    conn.transaction(|conn| {
        remove_campsite_member(
            conn,
            event_subject,
            campsite_id,
            &auth.member,
            profile.as_ref(),
            &actor,
            &actor.did,
        )
    })
    .map_err(handle_all_db_errors)?;

    Ok(())
}

pub fn remove_campsite_member(
    conn: &mut DbConnection,
    event_subject: &State<ReactiveSubject>,
    campsite_id: &str,
    target_member: &CampsiteMember,
    target_profile: Option<&Profile>,
    target_actor: &Actor,
    actor: &str,
) -> Result<(), diesel::result::Error> {
    diesel::delete(campsite_member::table)
        .filter(
            campsite_member::campsiteid
                .eq(campsite_id)
                .and(campsite_member::userid.eq(actor)),
        )
        .execute(conn)?;

    diesel::update(appview::actor::table)
        .filter(appview::actor::did.eq(actor))
        .set(appview::actor::campsites.eq(array_remove(appview::actor::campsites, campsite_id)))
        .execute(conn)?;

    diesel::update(appview::campsite::table)
        .filter(appview::campsite::id.eq(campsite_id))
        .set(appview::campsite::memberdids.eq(array_remove(appview::campsite::memberdids, actor)))
        .execute(conn)?;

    event_next_campsite(
        event_subject,
        &target_member.campsite_id,
        0,
        "MemberRemoved",
        &member_view_basic(target_member, target_profile, target_actor),
    );
    event_next(
        event_subject,
        "CampsiteLeft",
        &CampsiteLeftOutput {
            id: campsite_id.to_string(),
        },
        |binary| ReactiveSubjectData::CampsiteRemoved {
            campsite_id: campsite_id.to_string(),
            to_actor: actor.to_string(),
            binary,
        },
    );

    Ok(())
}

pub fn ensure_user_isnt_higher(
    is_owner: bool,
    all_roles: &mut Vec<CampsiteRole>,
    target_roles: &Vec<Option<Uuid>>,
    executor_roles: Vec<Option<Uuid>>,
) -> Result<(), XRPCError> {
    if is_owner {
        return Ok(());
    }

    all_roles.sort_by(|a, b| a.priority.cmp(&b.priority));

    let executor_highest_role = all_roles
        .iter()
        .find(|x| executor_roles.contains(&Some(x.id)));
    let target_highest_role = all_roles
        .iter()
        .find(|x| target_roles.contains(&Some(x.id)));

    return if executor_highest_role.map_or(true, |executor_role| {
        target_highest_role.map_or(false, |target_role| {
            executor_role.priority <= target_role.priority
        })
    }) {
        Err(XRPCError::Forbidden("Cannot remove target member that has role higher or the same priority as the executing member".to_string()))
    } else {
        Ok(())
    };
}
