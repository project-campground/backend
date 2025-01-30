use rsky_identity::{types::{DidDocument, Service}, IdResolver};
use rsky_syntax::{did::ensure_valid_did, handle::ensure_valid_handle};
use tokio::sync::RwLockWriteGuard;

use crate::xrpc_server::error::{Result, XRPCError};

pub const BLESSED_DID_METHODS: [&str; 2] = ["plc", "web"];

pub async fn resolve_at_identifier(resolver: &mut RwLockWriteGuard<'_, IdResolver>, actor: &String) -> Option<String> {
    if actor.starts_with("did:") {
        if !ensure_valid_did(actor.to_owned()).is_ok() {
            return None
        }
        
        let method = actor.split(":").nth(1).unwrap();
        if !BLESSED_DID_METHODS.contains(&method) {
            return None
        }

        return Some(actor.to_string())
    } else {
        if !ensure_valid_handle(actor.to_owned()).is_ok() {
            return None
        }
        
        match resolver.handle.resolve(&actor.to_owned()).await {
            Ok(did) => did,
            Err(_) => None
        }
    }
}

pub async fn try_resolve_at_identifier(resolver: &mut RwLockWriteGuard<'_, IdResolver>, actor: &String) -> Result<String> {
    match resolve_at_identifier(resolver, actor).await {
        Some(did) => Ok(did),
        None => Err(XRPCError::NotFound),
    }
}

pub async fn get_did_doc(resolver: &mut RwLockWriteGuard<'_, IdResolver>, did: &String) -> Option<DidDocument> {
    resolver.did.ensure_resolve(did, Some(false)).await.ok()
}

pub async fn try_get_did_doc(resolver: &mut RwLockWriteGuard<'_, IdResolver>, did: &String) -> Result<DidDocument> {
    match get_did_doc(resolver, did).await {
        Some(did_doc) => Ok(did_doc),
        None => Err(XRPCError::NotFound),
    }
}

pub fn get_handle(doc: &DidDocument) -> Option<String> {
    if let Some(known_as) = &doc.also_known_as {
        return known_as.iter().find(|&s| s.starts_with("at://")).map(|s| s.replace("at://", ""))
    }
    None
}

pub fn get_service(doc: &DidDocument, id: String, r#type: String) -> Option<Service> {
    if let Some(service) = &doc.service {
        return service.iter().find(|s| s.id == id && s.r#type == r#type).cloned()
    }
    None
}

pub fn get_pds(doc: &DidDocument) -> Option<Service> {
    get_service(doc, "#atproto_pds".to_string(), "AtprotoPersonalDataServer".to_string())
}

pub fn try_get_pds(doc: &DidDocument) -> Result<Service> {
    match get_pds(doc) {
        Some(service) => Ok(service),
        None => Err(XRPCError::NotFound),
    }
}
