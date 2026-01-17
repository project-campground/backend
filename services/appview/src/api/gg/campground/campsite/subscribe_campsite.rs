// use appview_schema::models::appview::{Bonfire, Campsite};
// use atproto_identity::storage_lru::LruDidDocumentStorage;
// use campground_lexicon::gg::campground::campsite::{BonfireViewBasic, CampsiteViewDetailed, GetMembersOutput};
// use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
// use rocket::{serde::json::Json,State};
// use reqwest::Client;
// use serde_json::json;

// use crate::{database::{actors::get_actor, establish_connection}, helpers::campsites::{bonfire_view_basic, campsite_view_detailed}, xrpc::{
//     auth::OptionalAuthorization,
//     error::{Result, XRPCError}
// }, xws::frames::{SocketDataFrame, SocketFrameSerializer}};

// #[get("/xrpc/gg.campground.campsite.subscribeCampsite?<id>")]
// pub async fn subscribe_campsite<'a>(auth: OptionalAuthorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, ws: ws::WebSocket, id: &str) -> ws::Stream!['a] {
//     let actor_did = auth.1.jose.issuer;
//     ws::Stream! { ws =>
//         for await message in ws {
//             println!("MSG {:?}", message);
//             let msg = message?;
//             match msg {
//                 ws::Message::Ping(payload) =>
//                     yield ws::Message::Pong(payload),
//                 ws::Message::Close(_) => {
//                     break;
//                 },
//                 ws::Message::Text(payload) => {
//                     let response = SocketDataFrame::<GetMembersOutput>::new("test".to_string(), GetMembersOutput { members: vec![] });
//                     let binary = response.binary();
//                     println!("Payload: {:?}", payload);
//                     match binary {
//                         Ok(value) => yield ws::Message::Binary(value),
//                         Err(err) => yield ws::Message::Text(err.to_string()),
//                     }
//                 },
//                 _ => ()
//             }
//             // yield ws::Message::Binary();
//         }
//     }
// }