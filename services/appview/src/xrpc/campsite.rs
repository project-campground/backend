use appview_schema::{models::appview::{Actor, Bonfire, Campsite, CampsiteMember, Tent, TentCategory}, schema::appview::{campsite, campsite_member}};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, result::Error::NotFound};

use rocket::{http::Status, request::{FromRequest, Outcome, Request}};
use thiserror::Error;
use uuid::Uuid;

use crate::util::outcome::{OptionConversionOutcome, ResultConversionOutcome};
use rocket::outcome::try_outcome;

use crate::{database::{actors::get_actor, campsites::get_campsite_and_member_from_db, establish_connection}, xrpc::{auth::Authorization, error::XRPCError}};

pub struct CampsiteInfo<'a> {
    pub actor: Actor,
    pub campsite: Campsite,
    pub member: CampsiteMember,
    #[allow(dead_code)]
    pub auth: Authorization<'a>,
}
/// Similar to `CampsiteInfo<'a>`, but this does not fetch actual Campsite and only verifies if actor still exists in the campsite.
pub struct CampsiteInfoBasic<'a> {
    pub actor: Actor,
    #[allow(dead_code)]
    pub auth: Authorization<'a>,
}
pub struct TentInfo<'a> {
    pub actor: Actor,
    pub tent: Tent,
    pub campsite: Campsite,
    pub member: CampsiteMember,
    #[allow(dead_code)]
    pub auth: Authorization<'a>,
}
pub struct CategoryInfo<'a> {
    pub actor: Actor,
    pub category: TentCategory,
    pub campsite: Campsite,
    pub member: CampsiteMember,
    #[allow(dead_code)]
    pub auth: Authorization<'a>,
}
pub struct BonfireInfo<'a> {
    pub actor: Actor,
    pub bonfire: Bonfire,
    pub campsite: Campsite,
    pub member: CampsiteMember,
    #[allow(dead_code)]
    pub auth: Authorization<'a>,
}
/// Used in permissions where it could be any campsite entity.
pub enum OneOfInfo<'a> {
    Tent(TentInfo<'a>),
    Category(CategoryInfo<'a>),
    Bonfire(BonfireInfo<'a>),
}

#[rocket::async_trait]
impl<'r, 'a> FromRequest<'r> for CampsiteInfo<'a> where 'r: 'a {
    type Error = CampsiteError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth = try_outcome!(
            req
                .guard::<Authorization>()
                .await
                .map_error(|e| (e.0, CampsiteError::AuthRequired))
        );

        let actor = get_actor(auth.client, auth.did_document_storage, &auth.actor_did)
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
        let auth = try_outcome!(
            req
                .guard::<Authorization>()
                .await
                .map_error(|e| (e.0, CampsiteError::AuthRequired))
        );

        let actor = get_actor(auth.client, auth.did_document_storage, &auth.actor_did)
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
fn fetch_uuid_from_req<'r>(req: &'r Request<'_>, name: &str) -> Outcome<Uuid, CampsiteError> {
    let id = try_outcome!(
        req
            .query_value::<&str>(name).map(|x| x.ok())
            .flatten()
            .unwrap_outcome(Outcome::Error((Status::BadRequest, CampsiteError::InvalidId)))
    );

    Uuid::parse_str(id)
        .outcome(|_| Outcome::Error((Status::BadRequest, CampsiteError::InvalidId)))
}
#[rocket::async_trait]
impl<'r, 'a> FromRequest<'r> for TentInfo<'a> where 'r: 'a {
    type Error = CampsiteError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth = try_outcome!(
            req
                .guard::<Authorization>()
                .await
                .map_error(|e| (e.0, CampsiteError::AuthRequired))
        );

        let actor = try_outcome!(
            get_actor(auth.client, auth.did_document_storage, &auth.actor_did)
                .await
                .outcome(|_| Outcome::Error((Status::Unauthorized, CampsiteError::NotOnInstance)))
        );

        let tent_id = try_outcome!(fetch_uuid_from_req(req, "tent_id"));

        let mut conn = establish_connection().unwrap();

        let tent = try_outcome!(
            crate::schema::appview::tent::table
                .filter(
                    crate::schema::appview::tent::id
                        .eq(tent_id)
                )
                .first::<Tent>(&mut conn)
                .outcome(|err| match err {
                    NotFound => Outcome::Error((Status::NotFound, CampsiteError::NoSuchCampsite)),
                    _ => Outcome::Error((Status::InternalServerError, CampsiteError::InternalServerError)),
                })
        );

        // Not in the campsite to view that. It also confirms existence of campsite
        if !actor.campsites.contains(&Some(tent.campsite_id.clone())) {
            return Outcome::Error((Status::Forbidden, CampsiteError::CannotView));
        }
        
        let (campsite, member) = try_outcome!(
            get_campsite_and_member_from_db(&tent.campsite_id, &actor.did)
                .outcome(|err|
                    match err {
                        XRPCError::NotFound => Outcome::Error((Status::NotFound, CampsiteError::NoSuchCampsite)),
                        _ => Outcome::Error((Status::InternalServerError, CampsiteError::InternalServerError))
                    }
                )
        );

        rocket::outcome::Outcome::Success(TentInfo { campsite, member, actor, tent, auth })
    }
}
#[rocket::async_trait]
impl<'r, 'a> FromRequest<'r> for CategoryInfo<'a> where 'r: 'a {
    type Error = CampsiteError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth = req.guard::<Authorization>().await.unwrap();

        let actor = try_outcome!(
            get_actor(auth.client, auth.did_document_storage, &auth.actor_did)
                .await
                .outcome(|_| Outcome::Error((Status::Unauthorized, CampsiteError::NotOnInstance)))
        );

        let category_id = try_outcome!(fetch_uuid_from_req(req, "category_id"));

        let mut conn = establish_connection().unwrap();

        let category = try_outcome!(
            crate::schema::appview::tent_category::table
                .filter(
                    crate::schema::appview::tent_category::id
                        .eq(category_id)
                )
                .first::<TentCategory>(&mut conn)
                .outcome(|err| match err {
                    NotFound => Outcome::Error((Status::NotFound, CampsiteError::NoSuchCampsite)),
                    _ => Outcome::Error((Status::InternalServerError, CampsiteError::InternalServerError)),
                })
        );

        // Not in the campsite to view that. It also confirms existence of campsite
        if !actor.campsites.contains(&Some(category.campsite_id.clone())) {
            return Outcome::Error((Status::Forbidden, CampsiteError::CannotView));
        }
        
        let (campsite, member) = try_outcome!(
            get_campsite_and_member_from_db(&category.campsite_id, &actor.did)
                .outcome(|err|
                    match err {
                        XRPCError::NotFound => Outcome::Error((Status::NotFound, CampsiteError::NoSuchCampsite)),
                        _ => Outcome::Error((Status::InternalServerError, CampsiteError::InternalServerError))
                    }
                )
        );

        rocket::outcome::Outcome::Success(CategoryInfo { campsite, member, actor, category, auth })
    }
}
#[rocket::async_trait]
impl<'r, 'a> FromRequest<'r> for BonfireInfo<'a> where 'r: 'a {
    type Error = CampsiteError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth = req.guard::<Authorization>().await.unwrap();
        
        let actor = try_outcome!(
            get_actor(auth.client, auth.did_document_storage, &auth.actor_did)
                .await
                .outcome(|_| Outcome::Error((Status::Unauthorized, CampsiteError::NotOnInstance)))
        );

        let bonfire_id = try_outcome!(
            req
                .query_value::<&str>("bonfire_id").map(|x| x.ok())
                .flatten()
                .unwrap_outcome(Outcome::Error((Status::BadRequest, CampsiteError::InvalidId)))
        );
        let mut conn = establish_connection().unwrap();

        let bonfire = try_outcome!(
            crate::schema::appview::bonfire::table
                .filter(
                    crate::schema::appview::bonfire::id
                        .eq(bonfire_id)
                )
                .first::<Bonfire>(&mut conn)
                .outcome(|err| match err {
                    NotFound => Outcome::Error((Status::NotFound, CampsiteError::NoSuchCampsite)),
                    _ => Outcome::Error((Status::InternalServerError, CampsiteError::InternalServerError)),
                })
        );

        // Not in the campsite to view that. It also confirms existence of campsite
        if !actor.campsites.contains(&Some(bonfire.campsite_id.clone())) {
            return Outcome::Error((Status::Forbidden, CampsiteError::CannotView));
        }
        
        let (campsite, member) = try_outcome!(
            get_campsite_and_member_from_db(&bonfire.campsite_id, &actor.did)
                .outcome(|err|
                    match err {
                        XRPCError::NotFound => Outcome::Error((Status::NotFound, CampsiteError::NoSuchCampsite)),
                        _ => Outcome::Error((Status::InternalServerError, CampsiteError::InternalServerError))
                    }
                )
        );

        rocket::outcome::Outcome::Success(BonfireInfo { campsite, member, actor, bonfire, auth })
    }
}
#[rocket::async_trait]
impl<'r, 'a> FromRequest<'r> for OneOfInfo<'a> where 'r: 'a {
    type Error = CampsiteError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let queries = req.query_fields().map(|x| x.name.as_name().as_str()).collect::<Vec<&str>>();

        if queries.contains(&"bonfire_id") {
            BonfireInfo::from_request(req)
                .await
                .map(|x| OneOfInfo::Bonfire(x))
        } else if queries.contains(&"category_id") {
            CategoryInfo::from_request(req)
                .await
                .map(|x| OneOfInfo::Category(x))
        } else if queries.contains(&"tent_id") {
            TentInfo::from_request(req)
                .await
                .map(|x| OneOfInfo::Tent(x))
        } else {
            rocket::outcome::Outcome::Error((Status::NotFound, CampsiteError::InvalidId))
        }
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