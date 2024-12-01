pub mod get_blob;
pub mod get_blocks;
pub mod get_latest_commit;
pub mod get_record;
pub mod get_repo;
pub mod get_repo_status;
pub mod list_blobs;
pub mod list_repos;
pub mod subscribe_repos;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_blob::get_blob,
        get_blocks::get_blocks,
        get_latest_commit::get_latest_commit,
        get_record::get_record,
        get_repo::get_repo,
        get_repo_status::get_repo_status,
        list_blobs::list_blobs,
        list_repos::list_repos,
        subscribe_repos::subscribe_repos,
    ]
}