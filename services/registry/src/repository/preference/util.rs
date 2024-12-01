/**
 * Implementation from https://github.com/blacksky-algorithms/rsky
 * Modified to work with our own DB
 * License: https://github.com/blacksky-algorithms/rsky/blob/main/LICENSE
 */
use rsky_pds::auth_verifier::AuthScope;

const FULL_ACCESS_ONLY_PREFS: [&'static str; 1] = ["app.bsky.actor.defs#personalDetailsPref"];

pub fn pref_in_scope(scope: AuthScope, pref_type: String) -> bool {
    if scope == AuthScope::Access {
        return true;
    }
    return !FULL_ACCESS_ONLY_PREFS.contains(&&*pref_type);
}