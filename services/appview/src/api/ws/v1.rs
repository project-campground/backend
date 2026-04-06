use appview_schema::models::appview::{Actor, CampsiteMember};
use tokio_stream::StreamExt;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use reqwest::Client;
use rocket::State;
use rxrust::{Observable, SharedScheduler};
use crate::{api::ws::reactive::{MEMBER_PERMS_DEFAULT, WebSocketOutput, on_reactive_data}, database::actors::get_actor, helpers::ws::{CampsiteMemberPermissions, get_did_from_auth}, realtime::frames::SocketErrorFrame};

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
        let mut current_membership: Option<CampsiteMember> = None;
        let actor_did = &get_did_from_auth(did_document_storage, client, init_message).await;

        if let Err(actor_did_error) = actor_did.clone() {
            if let Some(error_frame) = actor_did_error.0 {
                yield ws::Message::Binary(error_frame);
            }

            yield actor_did_error.1;
        }

        let actor_did = actor_did.clone().unwrap();

        let actor: &Option<Actor> = if actor_did.is_none() { &None } else {
            &get_actor(client, did_document_storage, actor_did.as_ref().unwrap())
                .await
                .ok()
        };
        let mut actor_campsites = actor.clone().map_or(vec![], |x| x.campsites);
        let mut permissions: &mut CampsiteMemberPermissions = &mut MEMBER_PERMS_DEFAULT.clone();

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

            match on_reactive_data(omsg, &actor_did, &mut actor_campsites, &mut current_campsite, &mut current_membership, &mut permissions).await {
                WebSocketOutput::Ignore => {
                },
                WebSocketOutput::BinaryData(data) => {
                    yield ws::Message::Binary(data);
                },
                WebSocketOutput::RocketMessage(message) => {
                    yield message;
                },
                WebSocketOutput::EmptyClose => {
                    break;
                },
                WebSocketOutput::MessagedClose(header, message) => {
                    println!("Messaged close: {:?}, {:?}", header, message);
                    let (message, close_message) = SocketErrorFrame::from_error_message(&header, &message);
        
                    if let Ok(message) = message {
                        yield ws::Message::Binary(message);
                    }
        
                    yield close_message;
                    break;
                },
                WebSocketOutput::RocketError(err) => {
                    println!("Rocket error: {:?}", err);
                    yield ws::Message::Close(Some(ws::frame::CloseFrame {
                        code: ws::frame::CloseCode::Error,
                        reason: std::borrow::Cow::Owned(err)
                    }));
                    break;
                }
            };
        }
    })
}