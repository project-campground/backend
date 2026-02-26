use rocket::State;
use rxrust::Observer;
use serde::Serialize;

use crate::realtime::{data::{ReactiveSubject, ReactiveSubjectData}, frames::{SocketDataFrame, SocketFrameSerializer}};

fn event_next_inner<TData, TFn>(event_subject: &State<ReactiveSubject>, data_type: &str, payload: TData, on_next: TFn)
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

pub fn event_next<T>(event_subject: &State<ReactiveSubject>, campsite_id: &String, data_type: &str, payload: T) where T: Serialize {
    event_next_inner(event_subject, data_type, payload, |binary|
        ReactiveSubjectData::Campsite(
            campsite_id.clone(),
            binary,
        )
    );
}
pub fn event_next_campsite_global<T>(event_subject: &State<ReactiveSubject>, campsite_id: &String, data_type: &str, payload: T) where T: Serialize {
    event_next_inner(event_subject, data_type, payload, |binary|
        ReactiveSubjectData::CampsiteGlobal(
            campsite_id.clone(),
            binary,
        )
    );
}
#[allow(dead_code)]
pub fn event_next_personal<T>(event_subject: &State<ReactiveSubject>, actor: &String, data_type: &str, payload: T) where T: Serialize {
    event_next_inner(event_subject, data_type, payload, |binary|
        ReactiveSubjectData::Personal(
            actor.clone(),
            binary,
        )
    );
}
pub fn event_next_campsite_added<T>(event_subject: &State<ReactiveSubject>, campsite_id: &String, actor: &String, data_type: &str, payload: T) where T: Serialize {
    event_next_inner(event_subject, data_type, payload, |binary|
        ReactiveSubjectData::CampsiteAdded(
            campsite_id.clone(),
            actor.clone(),
            binary,
        )
    );
}
pub fn event_next_campsite_removed<T>(event_subject: &State<ReactiveSubject>, campsite_id: &String, actor: &String, data_type: &str, payload: T) where T: Serialize {
    event_next_inner(event_subject, data_type, payload, |binary|
        ReactiveSubjectData::CampsiteRemoved(
            campsite_id.clone(),
            actor.clone(),
            binary,
        )
    );
}