/**
 * Implementation from https://github.com/blacksky-algorithms/rsky
 * Modified to work with our own DB
 * License: https://github.com/blacksky-algorithms/rsky/blob/main/LICENSE
 */
use crate::account_manager::helpers::account::AvailabilityFlags;
use crate::account_manager::AccountManager;
use crate::auth_verifier::AccessStandardIncludeChecks;
use crate::SharedSequencer;
use crate::repository::ActorStore;
use rsky_pds::models::{ErrorCode, ErrorMessageResponse};
use crate::repository::aws::s3::S3BlobStore;
use rsky_pds::repo::types::{CommitData, PreparedWrite};
use rsky_pds::repo::{
    make_aturi, prepare_create, prepare_update, PrepareCreateOpts, PrepareUpdateOpts,
};
use anyhow::{bail, Result};
use aws_config::SdkConfig;
use libipld::Cid;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use rsky_lexicon::com::atproto::repo::{PutRecordInput, PutRecordOutput};
use std::str::FromStr;

async fn inner_put_record(
    body: Json<PutRecordInput>,
    auth: AccessStandardIncludeChecks,
    sequencer: &State<SharedSequencer>,
    s3_config: &State<SdkConfig>,
) -> Result<PutRecordOutput> {
    let PutRecordInput {
        repo,
        collection,
        rkey,
        validate,
        record,
        swap_record,
        swap_commit,
    } = body.into_inner();
    let account = AccountManager::get_account(
        &repo,
        Some(AvailabilityFlags {
            include_deactivated: Some(true),
            include_taken_down: None,
        }),
    )
    .await?;
    if let Some(account) = account {
        if account.deactivated_at.is_some() {
            bail!("Account is deactivated")
        }
        let did = account.did;
        if did != auth.access.credentials.unwrap().did.unwrap() {
            bail!("AuthRequiredError")
        }
        // @TODO: Use ATUri
        let uri = make_aturi(did.clone(), Some(collection.clone()), Some(rkey.clone()));
        let swap_commit_cid = match swap_commit {
            Some(swap_commit) => Some(Cid::from_str(&swap_commit)?),
            None => None,
        };
        let swap_record_cid = match swap_record {
            Some(swap_record) => Some(Cid::from_str(&swap_record)?),
            None => None,
        };
        let (commit, write): (Option<CommitData>, PreparedWrite) = {
            let mut actor_store =
                ActorStore::new(did.clone(), S3BlobStore::new(did.clone(), s3_config));

            let current = actor_store
                .record
                .get_record(&uri, None, Some(true))
                .await?;
            println!("@LOG: debug inner_put_record, current: {current:?}");
            let write: PreparedWrite = if current.is_some() {
                PreparedWrite::Update(
                    prepare_update(PrepareUpdateOpts {
                        did: did.clone(),
                        collection,
                        rkey,
                        swap_cid: swap_record_cid,
                        record: serde_json::from_value(record)?,
                        validate,
                    })
                    .await?,
                )
            } else {
                PreparedWrite::Create(
                    prepare_create(PrepareCreateOpts {
                        did: did.clone(),
                        collection,
                        rkey: Some(rkey),
                        swap_cid: swap_record_cid,
                        record: serde_json::from_value(record)?,
                        validate,
                    })
                    .await?,
                )
            };

            match current {
                Some(current) if current.cid == write.cid().unwrap().to_string() => (None, write),
                _ => {
                    let commit = actor_store
                        .process_writes(vec![write.clone()], swap_commit_cid)
                        .await?;
                    (Some(commit), write)
                }
            }
        };

        if let Some(commit) = commit {
            let mut lock = sequencer.sequencer.write().await;
            lock.sequence_commit(did.clone(), commit.clone(), vec![write.clone()])
                .await?;
            AccountManager::update_repo_root(did, commit.cid, commit.rev)?;
        }
        Ok(PutRecordOutput {
            uri: write.uri().to_string(),
            cid: write.cid().unwrap().to_string(),
        })
    } else {
        bail!("Could not find repo: `{repo}`")
    }
}

#[rocket::post("/xrpc/com.atproto.repo.putRecord", format = "json", data = "<body>")]
pub async fn put_record(
    body: Json<PutRecordInput>,
    auth: AccessStandardIncludeChecks,
    sequencer: &State<SharedSequencer>,
    s3_config: &State<SdkConfig>,
) -> Result<Json<PutRecordOutput>, status::Custom<Json<ErrorMessageResponse>>> {
    println!("@LOG: debug put_record {body:#?}");
    match inner_put_record(body, auth, sequencer, s3_config).await {
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