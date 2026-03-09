use appview_schema::{models::appview::{Actor, CampsiteMember, CampsiteRole, Profile}, schema::appview::{self, campsite_member, campsite_role, profile}};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::membership::CampsiteLeftOutput;
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use diesel::pg::expression::dsl::array_remove;
use reqwest::Client;
use rocket::State;
use uuid::Uuid;

use crate::{database::{establish_connection, profiles::get_profile}, helpers::{api::handle_select_first_error, campsites::campsite_member_view_basic, permissions::{CampsitePermissionConsts, has_role_perms_or_owner}, ws::{event_next, event_next_campsite}}, realtime::data::{ReactiveSubject, ReactiveSubjectData}, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[post("/xrpc/gg.campground.membership.removeMember?<campsite_id>&<actor>")]
pub async fn remove_member(auth: CampsiteInfo<'_>, event_subject: &State<ReactiveSubject>, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, campsite_id: &str, actor: &str) -> Result<()> {    
    if actor == "" || actor == auth.actor.did {
        return remove_self(auth, event_subject, client, did_document_storage, campsite_id).await;
    } else if !has_role_perms_or_owner(&auth.campsite, &auth.member, CampsitePermissionConsts::KICK_MEMBERS, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

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
                .and(
                    campsite_member::userid.eq(&auth.actor.did)
                )
        )
        .inner_join(
            profile::table
                .on(
                    profile::creator.eq(
                        campsite_member::userid
                    )
                )
        )
        .inner_join(
            crate::schema::appview::actor::table
                .on(
                    crate::schema::appview::actor::did.eq(
                        campsite_member::userid
                    )
                )
        )
        .select(
            (campsite_member::all_columns, profile::all_columns, crate::schema::appview::actor::all_columns)
        )
        .first::<(CampsiteMember, Profile, Actor)>(&mut conn)
        .map_err(handle_select_first_error)?;

    ensure_user_isnt_higher(auth.campsite.owner == auth.actor.did, &mut all_roles.clone(), &target.0.roles, auth.member.roles.clone())?;

    remove_campsite_member(event_subject, campsite_id, &target.0, &target.1, &target.2, actor)
}

#[post("/xrpc/gg.campground.membership.removeMember?<campsite_id>")]
pub async fn remove_self(auth: CampsiteInfo<'_>, event_subject: &State<ReactiveSubject>, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, campsite_id: &str) -> Result<()> {    
    let (actor, profile) = get_profile(client, did_document_storage, &auth.actor.did)
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    remove_campsite_member(event_subject, campsite_id, &auth.member, &profile, &actor, &auth.actor.did)
}

pub fn remove_campsite_member(event_subject: &State<ReactiveSubject>, campsite_id: &str, target_member: &CampsiteMember, target_profile: &Profile, target_actor: &Actor, actor: &str) -> Result<()> {
    let mut conn = establish_connection().unwrap();
    diesel::delete(campsite_member::table)
        .filter(
            campsite_member::campsiteid.eq(campsite_id)
                .and(
                    campsite_member::userid
                        .eq(actor)
                )
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

    diesel::update(appview::actor::table)
        .filter(
            appview::actor::did.eq(actor)
        )
        .set(
            appview::actor::campsites
                .eq(
                    array_remove(
                        appview::actor::campsites,
                        campsite_id
                    )
                )
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

    diesel::update(appview::campsite::table)
        .filter(
            appview::campsite::id.eq(campsite_id)
        )
        .set(
            appview::campsite::memberdids
                .eq(
                    array_remove(
                        appview::campsite::memberdids,
                        actor,
                    )
                )
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

    event_next_campsite(event_subject, &target_member.campsite_id, 0, "MemberRemoved", campsite_member_view_basic(&target_member, &target_profile, &target_actor));
    event_next(event_subject, "CampsiteLeft", CampsiteLeftOutput { id: campsite_id.to_string() }, |binary|
        ReactiveSubjectData::CampsiteRemoved {
            campsite_id: campsite_id.to_string(),
            to_actor: actor.to_string(),
            binary,
        }
    );

    Ok(())
}

pub fn ensure_user_isnt_higher(is_owner: bool, all_roles: &mut Vec<CampsiteRole>, target_roles: &Vec<Option<Uuid>>, executor_roles: Vec<Option<Uuid>>) -> Result<(), XRPCError> {
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

    return if executor_highest_role.map_or(
        true,
        |executor_role|
            target_highest_role
                .map_or(false, |target_role| executor_role.priority <= target_role.priority)
    ) {
        Err(XRPCError::Forbidden("Cannot remove target member that has role higher or the same priority as the executing member".to_string()))
    } else {
        Ok(())
    }
}