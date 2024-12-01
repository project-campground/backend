/**
 * Implementation from https://github.com/blacksky-algorithms/rsky
 * Modified to work with our own DB
 * License: https://github.com/blacksky-algorithms/rsky/blob/main/LICENSE
 */
use crate::account_manager::helpers::account::{ActorAccount, AvailabilityFlags};
use crate::account_manager::AccountManager;
use anyhow::{bail, Result};

pub async fn assert_repo_availability(
    did: &String,
    is_admin_of_self: bool,
) -> Result<ActorAccount> {
    let account = AccountManager::get_account(
        did,
        Some(AvailabilityFlags {
            include_deactivated: Some(true),
            include_taken_down: Some(true),
        }),
    )
    .await?;
    match account {
        None => bail!("RepoNotFound: Could not find repo for DID: {did}"),
        Some(account) => {
            if is_admin_of_self {
                return Ok(account);
            }
            if account.takedown_ref.is_some() {
                bail!("RepoTakendown: Repo has been takendown: {did}");
            }
            if account.deactivated_at.is_some() {
                bail!("RepoDeactivated: Repo has been deactivated: {did}");
            }
            Ok(account)
        }
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        apply_writes::apply_writes,
        create_record::create_record,
        delete_record::delete_record,
        describe_repo::describe_repo,
        get_record::get_record,
        import_repo::import_repo,
        list_missing_blobs::list_missing_blobs,
        list_records::list_records,
        put_record::put_record,
        upload_blob::upload_blob,
    ]
}

pub mod apply_writes;
pub mod create_record;
pub mod delete_record;
pub mod describe_repo;
pub mod get_record;
pub mod import_repo;
pub mod list_missing_blobs;
pub mod list_records;
pub mod put_record;
pub mod upload_blob;