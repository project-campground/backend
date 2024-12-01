use crate::config::CORE_CONFIG;
use crate::api::com::atproto::server::validate_handle;
use tldextract::{TldExtractor, TldOption};
use anyhow::{Result, bail};

fn disallowed_tlds() -> Vec<String> {
    match CORE_CONFIG.dev_mode() {
        // If we're in developer mode then we can assume localhost is being used as the registry's domain
        // and remove it from the blacklist.
        true => vec![
            "alt".to_owned(), "arpa".to_owned(), "example".to_owned(), "internal".to_owned(), "invalid".to_owned(),
            "local".to_owned(), "onion".to_owned()
        ],
        false => vec![
            "alt".to_owned(), "arpa".to_owned(), "example".to_owned(), "internal".to_owned(), "invalid".to_owned(),
            "local".to_owned(), "localhost".to_owned(), "onion".to_owned()
        ]
    }
}

pub fn normalize_and_ensure_valid_handle(handle: &str) -> Result<String> {
    let normalized_handle = normalize_handle(handle);
    if !ensure_valid_handle(&normalized_handle) {
        bail!("Invalid handle");
    }
    Ok(normalized_handle)
}

pub fn normalize_and_validate_handle(handle: &str) -> Result<String> {
    let normalized_handle = normalize_handle(handle);
    if !validate_handle(&normalized_handle) {
        bail!("Invalid handle");
    }
    Ok(normalized_handle)
}

pub fn normalize_handle(handle: &str) -> String {
    handle.to_lowercase()
}

pub fn ensure_valid_handle(handle: &str) -> bool {
    if handle.len() > 253 {
        return false;
    }
    if handle.starts_with('.') || handle.ends_with('.') {
        return false;
    }
    let extractor = TldExtractor::new(TldOption::default());
    let tld_result = match extractor.extract(&handle.to_lowercase()) {
        Ok(result) => result,
        Err(_) => return false,
    };
    if tld_result.suffix.is_none() {
        return false
    }
    if disallowed_tlds().contains(&tld_result.suffix.clone().unwrap()) {
        return false
    }
    let segments = handle[0..handle.len()-tld_result.suffix.clone().unwrap().len()-1].split('.');
    for segment in segments {
        if segment.len() < 1 || segment.len() > 63 {
            return false;
        }
        for c in segment.chars() {
            if !c.is_ascii_alphanumeric() && c != '-' {
                return false;
            }
        }
        if segment.starts_with('-') || segment.ends_with('-') {
            return false;
        }
    }
    true
}

pub mod explicit_slurs;
pub mod reserved;