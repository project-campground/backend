use std::convert::Infallible;

use rxrust::{MutArc, SharedCtx, SharedScheduler, Subject, Subscribers, observer::DynObserver};
use uuid::Uuid;

#[derive(Clone)]
pub enum ReactiveSubjectData {
    RocketMessage(ws::Message),
    RocketError,
    /// # Summary
    /// For all actors who are members of a campsite, wherever they are in app, regardless if they are actively viewing the campsite or not.
    /// # Remarks
    /// This is useful WebSocket message for events like campsite avatar changes, where all members should see the new avatar even if they are not actively viewing the campsite.
    /// This reduces the need for refreshing the client to reflect the globally visible changes in the campsite.
    CampsiteGlobal {
        campsite_id: String,
        binary: Vec<u8>
    },
    #[allow(dead_code)]
    Personal {
        to_actor: String,
        binary: Vec<u8>
    },
    /// This event is visible to actor who joined or created a campsite.
    /// Internally, this also modifies campsite filtering for the any active WebSockets from the actor.
    CampsiteAdded {
        campsite_id: String,
        to_actor: String,
        binary: Vec<u8>,
    },
    /// This event is visible to actor who left a campsite or to everyone who were in a campsite that was deleted.
    /// Internally, this also modifies campsite filtering for the any active WebSockets from the members of the campsite.
    CampsiteRemoved {
        campsite_id: String,
        to_actor: String,
        binary: Vec<u8>,
    },
    /// # Summary
    /// Redirects a campsite WebSocket message to any member who has the specified permissions in the campsite and is currently viewing the campsite.
    /// # Remarks
    /// This is useful for events that may be campsite-wide, but not bound to any tent, category or bonfire. Examples of this are role and role list modifications, as well as invite creation.
    /// 
    /// > **DANGER:** For events like campsite invite creation, it should require `MANAGE_INVITES` permission and not be visible to every member, as it could easily be abused by allowing anyone to invite to any campsite, whether it's private or not.
    Campsite {
        campsite_id: String,
        permissions_required: u64,
        binary: Vec<u8>
    },
    /// Redirects a campsite WebSocket message to any member who can view the bonfire and is currently viewing the campsite.
    /// 
    /// > **NOTE:** This only requires permissions to have bonfire visible in the API.
    #[allow(dead_code)]
    Bonfire {
        campsite_id: String,
        bonfire_id: String,
        deleted: bool,
        binary: Vec<u8>
    },
    /// Redirects a campsite WebSocket message to any member who can view the bonfire and is currently viewing the campsite.
    /// 
    /// > **NOTE:** This only requires permissions to have tent category visible in the API.
    Category {
        campsite_id: String,
        bonfire_id: String,
        category_id: Uuid,
        deleted: bool,
        binary: Vec<u8>
    },
    /// Redirects a campsite WebSocket message to any member who can view the bonfire and is currently viewing the campsite.
    /// 
    /// > **NOTE:** This only requires permissions to have tent visible in the API.
    Tent {
        campsite_id: String,
        bonfire_id: String,
        category_id: Option<Uuid>,
        tent_id: Uuid,
        deleted: bool,
        binary: Vec<u8>
    },
    /// # Summary
    /// This is similar to other Campsite WS messages, but it also updates actor's permissions in the campsite known to WebSocket.
    /// # Remarks
    /// This is only used for roles added to or removed from members in the campsite.
    MemberRolesModified {
        campsite_id: String,
        actors: Vec<String>,
        role_id: Uuid,
        permissions_are_empty: bool,
        removed: bool,
        binary: Vec<u8>,
    },
    /// # Summary
    /// This is similar to other Campsite WS messages, but it also updates actor's permissions in the campsite known to WebSocket.
    /// # Remarks
    /// This is only used when role or user permissions are modified in tents, tent categories or bonfires.
    CampsitePermissionUpdated {
        campsite_id: String,

        bonfire_id: String,
        category_id: Option<Uuid>,
        tent_id: Option<Uuid>,

        user_id: Option<String>,
        role_id: Option<Uuid>,

        binary: Vec<u8>,
    },
}

pub type ReactiveSubject = SharedCtx<Subject<MutArc<Subscribers<Box<dyn DynObserver<ReactiveSubjectData, Infallible> + Send>>>>, SharedScheduler>;