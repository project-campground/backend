use std::borrow::Cow;

use appview_schema::models::appview::Actor;
use tokio_stream::StreamExt;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use reqwest::Client;
use rocket::State;
use rsky_common::cbor_to_struct;
use rxrust::{Observable, SharedScheduler};
use ws::frame::CloseFrame;
use crate::{database::actors::get_actor, realtime::{frames::{SocketErrorFrame, SocketFrameType}, messages::{SocketAuthFrame, SocketInAnyFrame, SocketInFramePayload}}, xrpc::auth::validate_jwt};

use crate::realtime::data::{ReactiveSubject, ReactiveSubjectData};

#[allow(unused_assignments)]
#[get("/ws/v1")]
pub async fn subscribe<'a>(ws: ws::WebSocket, client: &'a State<Client>, did_document_storage: &'a State<LruDidDocumentStorage>, event_subject: &'a State<ReactiveSubject>) -> ws::Stream!['a] {
    ws.stream(move |mut ws| rocket::async_stream::try_stream! {
        let subject: &ReactiveSubject = event_subject;

        let init_message = &ws.next().await;

        if init_message.is_none() || init_message.as_ref().unwrap().is_err() {
            let (message, close_message) = SocketErrorFrame::from_error_message("Unauthenticated", "Expected authentication message");

            if let Ok(message) = message {
                yield ws::Message::Binary(message);
            }

            yield close_message;
            return;
        }
        
        let init_message = init_message.as_ref().unwrap().as_ref().unwrap();
        let mut current_campsite: Option<String> = None;
        let actor_did: Option<String> = match init_message {
            ws::Message::Binary(bytes) => {
                let auth_data = cbor_to_struct::<SocketAuthFrame>(bytes.clone());

                // Bad CBOR
                if auth_data.is_err() {
                    let (message, close_message) = SocketErrorFrame::from_error_message("AuthenticationParsingError", "Could not parse binary auth CBOR");

                    if let Ok(message) = message {
                        yield ws::Message::Binary(message);
                    }

                    yield close_message;
                }
                
                let auth_data = auth_data.unwrap();

                // Make sure it is authentication frame, could be other frame
                if auth_data.op != SocketFrameType::Auth {
                    let (message, close_message) = SocketErrorFrame::from_error_message("AuthenticationBadMessage", "Expected authentication op to be 0");
    
                    if let Ok(message) = message {
                        yield ws::Message::Binary(message);
                    }
    
                    yield close_message;
                }

                if let Some(payload) = auth_data.payload {
                    // Get actor
                    let validated_jwt = validate_jwt(&payload.service_auth, did_document_storage, client).await;
                    if let Err(_) = validated_jwt {
                        None
                    } else {
                        let (_, claims) = validated_jwt.unwrap();
                        claims.jose.issuer
                    }
                } else {
                    None
                }
            },
            _ => {
                let (message, close_message) = SocketErrorFrame::from_error_message("InvalidAuthenticationFormat", "Expected a binary authentication message");

                if let Ok(message) = message {
                    yield ws::Message::Binary(message);
                }

                yield close_message;
                return;
            },
        };

        let actor: &Option<Actor> = if actor_did.is_none() { &None } else {
            &get_actor(client, did_document_storage, actor_did.as_ref().unwrap())
                .await
                .ok()
        };
        let mut actor_campsites = actor.clone().map_or(vec![], |x| x.campsites);

        // Might be useless?
        let observer = subject
            .clone()
            .subscribe_on(SharedScheduler);
        let observer_stream = observer.into_stream();

        let weird_ws = ws.map(|x|
            Ok(match x {
                Ok(value) => ReactiveSubjectData::RocketMessage(value),
                Err(_) => ReactiveSubjectData::RocketError,
            })
        );

        // Outgoing messages
        for await omsg in observer_stream.merge(weird_ws) {
            // Perhaps there's a better way to do that? It seems that observables cannot be unsubscribed if they are streams
            if omsg.is_err() {
                break;
            }

            let omsg = omsg.unwrap();

            match omsg {
                ReactiveSubjectData::RocketError => {
                    yield ws::Message::Close(Some(ws::frame::CloseFrame {
                        code: ws::frame::CloseCode::Error,
                        reason: std::borrow::Cow::Owned("WS Error received".to_string())
                    }));
                    break;
                },
                // Incoming messages, sent by the user
                ReactiveSubjectData::RocketMessage(msg) => {
                    match msg {
                        ws::Message::Ping(payload) =>
                            yield ws::Message::Pong(payload),
                        ws::Message::Close(_) => {
                            break;
                        },
                        ws::Message::Binary(bytes) => {
                            let data = cbor_to_struct::<SocketInAnyFrame>(bytes.clone());

                            if data.is_err() {
                                println!("Data is err: {:?}", data.err().unwrap());
                                let (message, close_message) = SocketErrorFrame::from_error_message("ParsingError", "Could not parse binary CBOR");

                                if let Ok(message) = message {
                                    yield ws::Message::Binary(message);
                                }

                                yield close_message;
                                continue;
                            }
                            
                            let data = data.unwrap();

                            // Make sure it is authentication frame, could be other frame
                            match data.op {
                                SocketFrameType::Data => {
                                    match data.payload {
                                        SocketInFramePayload::View(view) => {
                                            println!("Change campsite: {:?}", view.campsite.clone());
                                            current_campsite = if view.campsite == "" { None } else { Some(view.campsite) };
                                        }
                                    }
                                },
                                _ => {
                                    yield ws::Message::Close(Some(CloseFrame { code: ws::frame::CloseCode::Error, reason: Cow::Owned("Unexpected opcode".to_string()) }));
                                    break;
                                }
                            }
                        },
                        _ => ()
                    }
                },
                // Role created, Message created, etc.
                ReactiveSubjectData::Campsite(campsite, binary) => {
                    // Could be used in observer .filter, but borrowing could be less intuitive
                    if Some(campsite.clone()) != current_campsite {
                        continue;
                    }

                    yield ws::Message::Binary(binary);

                },
                // Campsite has been deleted, campsite modified, etc.
                ReactiveSubjectData::CampsiteGlobal(campsite, binary) => {
                    if !actor_campsites.contains(&Some(campsite.clone())) {
                        continue;
                    }

                    yield ws::Message::Binary(binary);
                },
                // Campsite joined
                ReactiveSubjectData::CampsiteAdded(campsite_id, to_actor, _event) => {
                    if Some(to_actor.clone()) != actor_did {
                        continue;
                    }

                    actor_campsites.push(Some(campsite_id));
                },
                // Campsite left
                ReactiveSubjectData::CampsiteRemoved(campsite_id, to_actor, _event) => {
                    if Some(to_actor.clone()) != actor_did {
                        continue;
                    } else if Some(campsite_id.clone()) == current_campsite {
                        current_campsite = None;
                    }

                    // To no longer send global campsite events from
                    let campsite_index = actor_campsites.iter().position(|x| x.as_ref().map_or(false, |y| *y == campsite_id));
                    if campsite_index.is_none() {
                        continue;
                    }

                    actor_campsites.remove(campsite_index.unwrap());
                },
                // DM received and whatever
                ReactiveSubjectData::Personal(to_actor, _event) => {
                    if Some(to_actor.clone()) != actor_did {
                        continue;
                    }
                },
            }
        }
    })
}