use rsky_pds::common::{validate_url, get_did};
use rsky_identity::types::DidDocument;

pub use rsky_pds::common::GetServiceEndpointOpts;

pub fn get_service_endpoint(doc: DidDocument, opts: GetServiceEndpointOpts) -> Option<String> {
    println!(
        "@LOG: common::get_service_endpoint() doc: {:?}; opts: {:?}",
        doc, opts
    );
    let did = get_did(&doc);
    match doc.service {
        None => None,
        Some(services) => {
            let found = services.iter().find(|service| {
                service.id == opts.id || service.id == format!("{}{}", did, opts.id)
            });
            match found {
                None => None,
                Some(found) => match opts.r#type {
                    None => validate_url(&found.service_endpoint),
                    Some(opts_type) if found.r#type == opts_type => {
                        validate_url(&found.service_endpoint)
                    }
                    _ => None,
                },
            }
        }
    }
}