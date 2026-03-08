use std::convert::Infallible;

use rxrust::{MutArc, SharedCtx, SharedScheduler, Subject, Subscribers, observer::DynObserver};
use uuid::Uuid;

#[derive(Clone)]
pub enum ReactiveSubjectData {
    RocketMessage(ws::Message),
    RocketError,
    // For users, wherever they are in app
    CampsiteGlobal {
        campsite_id: String,
        binary: Vec<u8>
    },
    #[allow(dead_code)]
    Personal {
        to_actor: String,
        binary: Vec<u8>
    },
    // For users, wherever they are + modifying WebSocket filtering
    CampsiteAdded {
        campsite_id: String,
        to_actor: String,
        binary: Vec<u8>,
    },
    CampsiteRemoved {
        campsite_id: String,
        to_actor: String,
        binary: Vec<u8>,
    },
    // Within campsite + has appropriate perms to view it
    Campsite {
        campsite_id: String,
        binary: Vec<u8>
    },
    #[allow(dead_code)]
    Bonfire {
        campsite_id: String,
        bonfire_id: String,
        deleted: bool,
        binary: Vec<u8>
    },
    Category {
        campsite_id: String,
        bonfire_id: String,
        category_id: Uuid,
        deleted: bool,
        binary: Vec<u8>
    },
    Tent {
        campsite_id: String,
        bonfire_id: String,
        category_id: Option<Uuid>,
        tent_id: Uuid,
        deleted: bool,
        binary: Vec<u8>
    },
    #[allow(dead_code)]
    // For updating WebSocket known permissions
    MemberRolesModified {
        campsite_id: String,
        actors: Vec<String>,
        role_id: Uuid,
        permissions_are_empty: bool,
        removed: bool,
        binary: Vec<u8>,
    },
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