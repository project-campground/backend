use std::convert::Infallible;

use rxrust::{MutArc, SharedCtx, SharedScheduler, Subject, Subscribers, observer::DynObserver};

#[derive(Clone)]
pub enum ReactiveSubjectData {
    RocketMessage(ws::Message),
    RocketError,
    Campsite(String, Vec<u8>),
    CampsiteGlobal(String, Vec<u8>),
    CampsiteAdded(String, String, Vec<u8>),
    CampsiteRemoved(String, String, Vec<u8>),
    #[allow(dead_code)]
    Personal(String, Vec<u8>),
}

// #[derive(Clone)]
// pub enum ReactiveSubjectDataType {
//     TentMessageCreated,
// }
pub type ReactiveSubject = SharedCtx<Subject<MutArc<Subscribers<Box<dyn DynObserver<ReactiveSubjectData, Infallible> + Send>>>>, SharedScheduler>;