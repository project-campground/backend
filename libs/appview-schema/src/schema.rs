// @generated automatically by Diesel CLI.

pub mod appview {
    diesel::table! {
        appview.actor (did) {
            did -> Varchar,
            handle -> Nullable<Varchar>,
            homeserver -> Varchar,
            indexedat -> Varchar,
        }
    }

    diesel::table! {
        appview.profile (uri) {
            uri -> Varchar,
            cid -> Varchar,
            creator -> Varchar,
            displayname -> Nullable<Varchar>,
            description -> Nullable<Varchar>,
            avatarcid -> Nullable<Varchar>,
            bannercid -> Nullable<Varchar>,
            indexedat -> Varchar,
        }
    }

    diesel::allow_tables_to_appear_in_same_query!(
        actor,
        profile,
    );
}
