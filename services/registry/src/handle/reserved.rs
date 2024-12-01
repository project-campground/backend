use tldextract::{TldExtractor, TldOption};
use crate::config::IDENTITY_CONFIG;

fn parse_list(s: &str) -> Vec<String> {
    let mut list = Vec::new();
    for line in s.lines() {
        if line.starts_with("#") {
            // skip comments
            continue;
        }
        let line = line.trim();
        if line.is_empty() {
            // skip empty lines
            continue;
        }
        list.push(line.to_string());
    }
    list
}

const DEFAULT_LIST: &str = include_str!("reserved_handles.txt");

fn load_reserved_handles() -> Vec<String> {
    let mut list = vec![];
    if IDENTITY_CONFIG.use_default_reserved_handles.unwrap_or(true) {
        list.extend(parse_list(DEFAULT_LIST));
    }
    if let Some(path) = &IDENTITY_CONFIG.reserved_handles_path {
        let content = std::fs::read_to_string(path).unwrap();
        list.extend(parse_list(&content));
    }
    list
}

fn trim_service_domain(handle: &str) -> Option<String> {
    for domain in &IDENTITY_CONFIG.service_handle_domains {
        if handle.ends_with(domain) {
            return Some(handle.trim_end_matches(&(".".to_owned() + domain)).to_string());
        }
    }
    None
}

pub fn is_handle_reserved(handle: &str) -> bool {
    let handle = match trim_service_domain(handle) {
        Some(handle) => handle,
        None => {
            let extractor = TldExtractor::new(TldOption::default());
            match extractor.extract(handle) {
                Ok(domain) => {
                    match domain.suffix {
                        Some(suffix) => {
                            handle.trim_end_matches(&(".".to_owned() + &suffix)).to_string()
                        },
                        None => handle.to_string(),
                    }
                },
                Err(_) => handle.to_string(),
            }
        },
    };
    println!("is_handle_reserved: {}", handle);
    load_reserved_handles().contains(&handle.to_lowercase())
}