/**
 * Implementation from https://github.com/blacksky-algorithms/rsky
 * Modified to work with our own DB
 * License: https://github.com/blacksky-algorithms/rsky/blob/main/LICENSE
 */
use crate::auth_verifier::AccessStandard;
use crate::repository::aws::s3::S3BlobStore;
use crate::repository::ActorStore;
use rsky_pds::models::{ErrorCode, ErrorMessageResponse};
use anyhow::Result;
use aws_config::SdkConfig;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use rsky_lexicon::app::bsky::actor;
use serde::{Deserialize, Serialize};

async fn inner_get_preferences(
    s3_config: &State<SdkConfig>,
    auth: AccessStandard,
) -> Result<GetPreferencesOutput> {
    let auth = auth.access.credentials.unwrap();
    let requester = auth.did.unwrap().clone();
    let actor_store = ActorStore::new(
        requester.clone(),
        S3BlobStore::new(requester.clone(), s3_config),
    );
    let preferences: Vec<RefPreferences> = convert_preferences(actor_store
        .pref
        .get_preferences(Some("app.bsky".to_string()), auth.scope.unwrap())
        .await?);

    Ok(GetPreferencesOutput { preferences })
}

/// Get private preferences attached to the current account. Expected use is synchronization
/// between multiple devices, and import/export during account migration. Requires auth.
#[rocket::get("/xrpc/app.bsky.actor.getPreferences")]
pub async fn get_preferences(
    s3_config: &State<SdkConfig>,
    auth: AccessStandard,
) -> Result<Json<GetPreferencesOutput>, status::Custom<Json<ErrorMessageResponse>>> {
    match inner_get_preferences(s3_config, auth).await {
        Ok(res) => Ok(Json(res)),
        Err(error) => {
            eprintln!("@LOG: ERROR: {error}");
            let internal_error = ErrorMessageResponse {
                code: Some(ErrorCode::InternalServerError),
                message: Some(error.to_string()),
            };
            return Err(status::Custom(
                Status::InternalServerError,
                Json(internal_error),
            ));
        }
    }
}

// Dumb workaround because rsky_lexicon does not serialize BskyAppPreferences properly

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GetPreferencesOutput {
    pub preferences: Vec<RefPreferences>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
pub enum RefPreferences {
    #[serde(rename = "app.bsky.actor.defs#adultContentPref")]
    AdultContentPref(actor::AdultContentPref),
    #[serde(rename = "app.bsky.actor.defs#contentLabelPref")]
    ContentLabelPref(actor::ContentLabelPref),
    #[serde(rename = "app.bsky.actor.defs#savedFeedsPref")]
    SavedFeedsPref(actor::SavedFeedsPref),
    #[serde(rename = "app.bsky.actor.defs#savedFeedsPrefV2")]
    SavedFeedsPrefV2(actor::SavedFeedsPrefV2),
    #[serde(rename = "app.bsky.actor.defs#personalDetailsPref")]
    PersonalDetailsPref(actor::PersonalDetailsPref),
    #[serde(rename = "app.bsky.actor.defs#feedViewPref")]
    FeedViewPref(actor::FeedViewPref),
    #[serde(rename = "app.bsky.actor.defs#threadViewPref")]
    ThreadViewPref(actor::ThreadViewPref),
    #[serde(rename = "app.bsky.actor.defs#interestsPref")]
    InterestsPref(actor::InterestsPref),
    #[serde(rename = "app.bsky.actor.defs#mutedWordsPref")]
    MutedWordsPref(actor::MutedWordsPref),
    #[serde(rename = "app.bsky.actor.defs#hiddenPostsPref")]
    HiddenPostsPref(actor::HiddenPostsPref),
    #[serde(rename = "app.bsky.actor.defs#bskyAppStatePref")]
    BskyAppStatePref(BskyAppStatePref),
    #[serde(rename = "app.bsky.actor.defs#labelersPref")]
    LabelersPref(actor::LabelersPref),
}

fn convert_preferences(prefs: Vec<actor::RefPreferences>) -> Vec<RefPreferences> {
    prefs.into_iter().map(|x| x.into()).collect()
}

impl Into<RefPreferences> for actor::RefPreferences {
    fn into(self) -> RefPreferences {
        match self {
            actor::RefPreferences::AdultContentPref(x) => RefPreferences::AdultContentPref(x),
            actor::RefPreferences::ContentLabelPref(x) => RefPreferences::ContentLabelPref(x),
            actor::RefPreferences::SavedFeedsPref(x) => RefPreferences::SavedFeedsPref(x),
            actor::RefPreferences::SavedFeedsPrefV2(x) => RefPreferences::SavedFeedsPrefV2(x),
            actor::RefPreferences::PersonalDetailsPref(x) => RefPreferences::PersonalDetailsPref(x),
            actor::RefPreferences::FeedViewPref(x) => RefPreferences::FeedViewPref(x),
            actor::RefPreferences::ThreadViewPref(x) => RefPreferences::ThreadViewPref(x),
            actor::RefPreferences::InterestsPref(x) => RefPreferences::InterestsPref(x),
            actor::RefPreferences::MutedWordsPref(x) => RefPreferences::MutedWordsPref(x),
            actor::RefPreferences::HiddenPostsPref(x) => RefPreferences::HiddenPostsPref(x),
            actor::RefPreferences::BskyAppStatePref(x) => RefPreferences::BskyAppStatePref(x.into()),
            actor::RefPreferences::LabelersPref(x) => RefPreferences::LabelersPref(x),
        }
    }
}

impl RefPreferences {
    pub fn get_type(&self) -> String {
        let r#type = match self {
            RefPreferences::AdultContentPref(_) => "app.bsky.actor.defs#adultContentPref",
            RefPreferences::ContentLabelPref(_) => "app.bsky.actor.defs#contentLabelPref",
            RefPreferences::SavedFeedsPref(_) => "app.bsky.actor.defs#savedFeedsPref",
            RefPreferences::SavedFeedsPrefV2(_) => "app.bsky.actor.defs#savedFeedsPrefV2",
            RefPreferences::PersonalDetailsPref(_) => "app.bsky.actor.defs#personalDetailsPref",
            RefPreferences::FeedViewPref(_) => "app.bsky.actor.defs#feedViewPref",
            RefPreferences::ThreadViewPref(_) => "app.bsky.actor.defs#threadViewPref",
            RefPreferences::InterestsPref(_) => "app.bsky.actor.defs#interestsPref",
            RefPreferences::MutedWordsPref(_) => "app.bsky.actor.defs#mutedWordsPref",
            RefPreferences::HiddenPostsPref(_) => "app.bsky.actor.defs#hiddenPostsPref",
            RefPreferences::BskyAppStatePref(_) => "app.bsky.actor.defs#bskyAppStatePref",
            RefPreferences::LabelersPref(_) => "app.bsky.actor.defs#labelersPref",
        };
        r#type.to_string()
    }
}

/// A grab bag of state that's specific to the bsky.app program.
/// Third-party apps shouldn't use this.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BskyAppStatePref {
    pub active_progress_guide: Option<actor::BskyAppProgressGuide>,
    // An array of tokens which identify nudges (modals, popups, tours, highlight dots)
    // that should be shown to the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queued_nudges: Option<Vec<String>>,
}

impl Into<BskyAppStatePref> for actor::BskyAppStatePref {
    fn into(self) -> BskyAppStatePref {
        BskyAppStatePref {
            active_progress_guide: self.active_progress_guide,
            queued_nudges: self.queued_nudges,
        }
    }
}