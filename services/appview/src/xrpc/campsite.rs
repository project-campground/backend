use appview_schema::{models::appview::{Actor, Campsite, CampsiteMember, Tent}, schema::appview::{campsite, campsite_member}};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, result::Error::NotFound};

use rocket::{http::Status, request::{FromRequest, Outcome, Request}};
use thiserror::Error;
use uuid::Uuid;

use crate::{database::{actors::get_actor, campsites::get_campsite_and_member_from_db, establish_connection}, xrpc::{auth::Authorization, error::XRPCError}};

pub struct CampsiteInfo<'a> {
    pub actor: Actor,
    pub campsite: Campsite,
    pub member: CampsiteMember,
    pub auth: Authorization<'a>,
}
pub struct CampsiteInfoBasic<'a> {
    pub actor: Actor,
    pub auth: Authorization<'a>,
}
pub struct TentInfo<'a> {
    pub actor: Actor,
    pub tent: Tent,
    pub campsite: Campsite,
    pub member: CampsiteMember,
    pub auth: Authorization<'a>,
}

#[rocket::async_trait]
impl<'r, 'a> FromRequest<'r> for CampsiteInfo<'a> where 'r: 'a {
    type Error = CampsiteError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth = req.guard::<Authorization>().await.unwrap();

        let actor = get_actor(auth.client, auth.did_document_storage, auth.actor_did.clone().as_str())
            .await;

        if actor.is_err() {
            return Outcome::Error((Status::Unauthorized, CampsiteError::NotOnInstance));
        }

        let actor = actor.unwrap();
        let campsite_id = req.query_value::<String>("campsite_id").map(|x| x.ok()).flatten();

        // Not in the campsite to view that. It also confirms existence of campsite
        if campsite_id.clone().map_or(false, |x| !actor.campsites.contains(&Some(x))) {
            return Outcome::Error((Status::Forbidden, CampsiteError::CannotView));
        }

        let campsite_id = campsite_id.clone().unwrap();

        let mut conn = establish_connection().unwrap();
        let query = campsite_member::table
            .filter(
                campsite_member::campsiteid
                    .eq(
                        campsite_id
                    )
                    .and(
                        campsite_member::userid
                            .eq(
                                auth.actor_did.clone()
                            )
                    )
            )
            .inner_join(
                campsite::table
                    .on(
                        campsite::id.eq(campsite_member::campsiteid)
                    )
            )
            .first::<(CampsiteMember, Campsite)>(&mut conn)
            .map_err(|err| match err {
                NotFound => Outcome::Error((Status::NotFound, CampsiteError::NoSuchCampsite)),
                _ => Outcome::Error((Status::InternalServerError, CampsiteError::InternalServerError)),
            });

        if query.is_err() {
            return query.err().unwrap();
        }

        let (member, camp) = query.unwrap();

        rocket::outcome::Outcome::Success(CampsiteInfo { actor, campsite: camp, member, auth })
    }
}
#[rocket::async_trait]
impl<'r, 'a> FromRequest<'r> for CampsiteInfoBasic<'a> where 'r: 'a {
    type Error = CampsiteError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth = req.guard::<Authorization>().await.unwrap();

        let actor = get_actor(auth.client, auth.did_document_storage, auth.actor_did.clone().as_str())
            .await;

        if actor.is_err() {
            return Outcome::Error((Status::Unauthorized, CampsiteError::NotOnInstance));
        }

        let actor = actor.unwrap();
        let campsite_id = req.query_value::<String>("campsite_id").map(|x| x.ok()).flatten();

        // Not in the campsite to view that. It also confirms existence of campsite
        if campsite_id.clone().map_or(false, |x| !actor.campsites.contains(&Some(x))) {
            return Outcome::Error((Status::Forbidden, CampsiteError::CannotView));
        }

        rocket::outcome::Outcome::Success(CampsiteInfoBasic { actor, auth })
    }
}
#[rocket::async_trait]
impl<'r, 'a> FromRequest<'r> for TentInfo<'a> where 'r: 'a {
    type Error = CampsiteError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth = req.guard::<Authorization>().await.unwrap();
        
        let actor = get_actor(auth.client, auth.did_document_storage, auth.actor_did.clone().as_str())
            .await;
        
        if actor.is_err() {
            return Outcome::Error((Status::Unauthorized, CampsiteError::NotOnInstance));
        }
        
        let actor = actor.unwrap();
        let tent_id = req.query_value::<&str>("tent_id").map(|x| x.ok()).flatten();
        
        if tent_id.is_none() {
            return Outcome::Error((Status::BadRequest, CampsiteError::InvalidId));
        }
        
        let tent_id_uuid = Uuid::parse_str(tent_id.unwrap());
        
        if tent_id_uuid.is_err() {
            return Outcome::Error((Status::BadRequest, CampsiteError::InvalidId));
        }
        
        let mut conn = establish_connection().unwrap();
        let tent = crate::schema::appview::tent::table
            .filter(
                crate::schema::appview::tent::id
                    .eq(tent_id_uuid.unwrap())
            )
            .first::<Tent>(&mut conn)
            .map_err(|err| match err {
                NotFound => Outcome::Error((Status::NotFound, CampsiteError::NoSuchCampsite)),
                _ => Outcome::Error((Status::InternalServerError, CampsiteError::InternalServerError)),
            });
        
        if tent.is_err() {
            return tent.err().unwrap();
        }
        
        let tent = tent.unwrap();
        // Not in the campsite to view that. It also confirms existence of campsite
        if !actor.campsites.contains(&Some(tent.campsite_id.clone())) {
            return Outcome::Error((Status::Forbidden, CampsiteError::CannotView));
        }
        
        let campsite_and_member = get_campsite_and_member_from_db(tent.campsite_id.clone(), actor.did.clone())
            .map_err(|err|
                match err {
                    XRPCError::NotFound => Outcome::Error((Status::NotFound, CampsiteError::NoSuchCampsite)),
                    _ => Outcome::Error((Status::InternalServerError, CampsiteError::InternalServerError))
                }
            );
        if campsite_and_member.is_err() {
            return campsite_and_member.err().unwrap();
        }
        
        let (campsite, member) = campsite_and_member.unwrap();;

        rocket::outcome::Outcome::Success(TentInfo { campsite, member, actor, tent, auth })
    }
}

#[derive(Error, Debug)]
pub enum CampsiteError {
    #[error("Auth required")]
    AuthRequired,
    #[error("Actor not part of the instance")]
    NotOnInstance,
    #[error("User cannot view campsite that they are not member of")]
    CannotView,
    #[error("Invalid supplied content ID in the query")]
    InvalidId,
    #[error("Campsite does not exist")]
    NoSuchCampsite,
    #[error("Internal server error")]
    InternalServerError,
    // #[error("Account has been taken down")]
    // AccountTakedown,
    // #[error("Account was deactivated")]
    // AccountDeactivated,
}