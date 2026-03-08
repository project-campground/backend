use std::collections::HashMap;

use appview_schema::models::appview::Bonfire;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::permission::{PermissionsDictionary, PermissionsStateDictionary};
use reqwest::Client;
use rocket::State;
use rsky_common::cbor_to_struct;
use rxrust::Observer;
use serde::Serialize;
use uuid::Uuid;
use ws::Message;

use crate::{realtime::{data::{ReactiveSubject, ReactiveSubjectData}, frames::{SocketDataFrame, SocketErrorFrame, SocketFrameSerializer, SocketFrameType}, messages::SocketAuthFrame}, xrpc::auth::validate_jwt};

#[derive(Clone, Debug)]
pub struct CampsiteMemberPermissions {
    pub roles: PermissionsDictionary,
    pub bonfires: HashMap<String, PermissionsStateDictionary>,
    pub categories: HashMap<Uuid, PermissionsStateDictionary>,
    pub tents: HashMap<Uuid, PermissionsStateDictionary>,
}

pub async fn get_did_from_auth(did_document_storage: &State<LruDidDocumentStorage>, client: &State<Client>, init_message: &Message) -> Result<Option<String>, (Option<Vec<u8>>, ws::Message)> {
    match init_message {
        ws::Message::Binary(bytes) => {
            let auth_data = cbor_to_struct::<SocketAuthFrame>(bytes.clone());

            // Bad CBOR
            if auth_data.is_err() {
                println!("Err: {:?}", auth_data);
                let (message, close_message) = SocketErrorFrame::from_error_message("AuthenticationParsingError", "Could not parse binary auth CBOR");

                return Err((message.ok(), close_message));
            }
            
            let auth_data = auth_data.unwrap();

            // Make sure it is authentication frame, could be other frame
            if auth_data.op != SocketFrameType::Auth {
                let (message, close_message) = SocketErrorFrame::from_error_message("AuthenticationBadMessage", "Expected authentication op to be 0");

                return Err((message.ok(), close_message));
            }

            if let Some(payload) = auth_data.payload {
                // Get actor
                let validated_jwt = validate_jwt(&payload.service_auth, did_document_storage, client).await;
                if let Err(_) = validated_jwt {
                    Ok(None)
                } else {
                    let (_, claims) = validated_jwt.unwrap();
                    Ok(claims.jose.issuer)
                }
            } else {
                Ok(None)
            }
        },
        _ => {
            let (message, close_message) = SocketErrorFrame::from_error_message("InvalidAuthenticationFormat", "Expected a binary authentication message");

            return Err((message.ok(), close_message));
        },
    }
}

pub fn event_next<TData, TFn>(event_subject: &State<ReactiveSubject>, data_type: &str, payload: TData, on_next: TFn)
    where TData: Serialize,
          TFn: Fn(Vec<u8>) -> ReactiveSubjectData,
{
    // Use references somehow. I hate lack of GC
    let response = SocketDataFrame::<TData>::new(
        data_type.to_string(),
        payload,
    );
    let binary = response.binary();

    if let Ok(binary) = binary {
        event_subject.inner.clone().next(
            on_next(binary)
        );
    }
}

pub fn event_next_campsite<T>(event_subject: &State<ReactiveSubject>, campsite_id: &String, data_type: &str, payload: T) where T: Serialize {
    event_next(event_subject, data_type, payload, |binary|
        ReactiveSubjectData::Campsite {
            campsite_id: campsite_id.clone(),
            binary,
        }
    );
}
pub fn event_next_bonfire<T>(event_subject: &State<ReactiveSubject>, bonfire: &Bonfire, deleted: bool, data_type: &str, payload: T) where T: Serialize {
    event_next(event_subject, data_type, payload, |binary|
        ReactiveSubjectData::Bonfire {
            campsite_id: bonfire.campsite_id.clone(),
            bonfire_id: bonfire.id.clone(),
            deleted,
            binary,
        }
    );
}
pub fn event_next_category<T>(event_subject: &State<ReactiveSubject>, campsite_id: &String, bonfire_id: &String, category_id: Uuid, deleted: bool, data_type: &str, payload: T) where T: Serialize {
    event_next(event_subject, data_type, payload, |binary|
        ReactiveSubjectData::Category {
            campsite_id: campsite_id.clone(),
            bonfire_id: bonfire_id.clone(),
            category_id: category_id,
            deleted,
            binary,
        }
    );
}
pub fn event_next_tent<T>(event_subject: &State<ReactiveSubject>, campsite_id: &String, bonfire_id: &String, category_id: Option<Uuid>, tent_id: Uuid, deleted: bool, data_type: &str, payload: T) where T: Serialize {
    event_next(event_subject, data_type, payload, |binary|
        ReactiveSubjectData::Tent {
            campsite_id: campsite_id.clone(),
            bonfire_id: bonfire_id.clone(),
            category_id: category_id,
            tent_id: tent_id,
            deleted,
            binary,
        }
    );
}
pub fn event_next_campsite_global<T>(event_subject: &State<ReactiveSubject>, campsite_id: &String, data_type: &str, payload: T) where T: Serialize {
    event_next(event_subject, data_type, payload, |binary|
        ReactiveSubjectData::CampsiteGlobal {
            campsite_id: campsite_id.clone(),
            binary,
        }
    );
}
#[allow(dead_code)]
pub fn event_next_personal<T>(event_subject: &State<ReactiveSubject>, actor: &String, data_type: &str, payload: T) where T: Serialize {
    event_next(event_subject, data_type, payload, |binary|
        ReactiveSubjectData::Personal {
            to_actor: actor.clone(),
            binary,
        }
    );
}
pub fn event_next_campsite_removed<T>(event_subject: &State<ReactiveSubject>, campsite_id: &String, actor: &String, payload: T) where T: Serialize {
    event_next(event_subject, "CampsiteLeft", payload, |binary|
        ReactiveSubjectData::CampsiteRemoved {
            campsite_id: campsite_id.clone(),
            to_actor: actor.clone(),
            binary,
        }
    );
}