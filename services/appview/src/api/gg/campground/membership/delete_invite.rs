use appview_schema::{models::appview::{Actor, CampsiteInvite, Profile}, schema::appview::{campsite_invite, profile}};
use campground_lexicon::gg::campground::membership::CampsiteInviteViewCampsite;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::{State, serde::json::Json};
use uuid::Uuid;

use crate::{database::establish_connection, helpers::{api::{handle_all_db_errors, handle_select_first_error}, campsites::campsite_invite_view_campsite, permissions::{CampsitePermissionConsts, has_role_perms_or_owner}, ws::event_next_campsite}, realtime::data::ReactiveSubject, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[post("/xrpc/gg.campground.membership.deleteInvite?<campsite_id>&<invite_id>")]
pub async fn delete_invite(auth: CampsiteInfo<'_>, event_subject: &State<ReactiveSubject>,  campsite_id: &str, invite_id: &str) -> Result<Json<CampsiteInviteViewCampsite>> {    
    if !has_role_perms_or_owner(&auth.campsite, &auth.member, CampsitePermissionConsts::MANAGE_INVITES, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    let uuid = Uuid::try_parse(invite_id)
        .map_err(|_| XRPCError::BadRequest("Invalid 'invite_id' format. Expected UUID".to_string()))?;

    let invite = campsite_invite::table
        .filter(campsite_invite::id.eq(uuid))
        .inner_join(
            profile::table
                .on(
                    profile::creator.eq(
                        campsite_invite::createdby
                    )
                )
        )
        .inner_join(
            crate::schema::appview::actor::table
                .on(
                    crate::schema::appview::actor::did.eq(
                        campsite_invite::createdby
                    )
                )
        )
        .first::<(CampsiteInvite, Profile, Actor)>(&mut conn)
        .map_err(handle_select_first_error)?;

    if invite.0.campsite_id != campsite_id {
        return Err(XRPCError::NotFound);
    }

    diesel::delete(
        campsite_invite::table
    )
        .filter(
            campsite_invite::id
                .eq(invite.0.id.clone())
        )
        .execute(&mut conn)
        .map_err(handle_all_db_errors)?;

    event_next_campsite(
        event_subject,
        &auth.campsite.id,
        CampsitePermissionConsts::MANAGE_INVITES,
        "InviteDeleted",
        campsite_invite_view_campsite(&invite.0, &invite.1, &invite.2)
    );

    return Ok(Json(campsite_invite_view_campsite(&invite.0, &invite.1, &invite.2)));
}