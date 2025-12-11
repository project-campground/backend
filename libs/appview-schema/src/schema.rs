// @generated automatically by Diesel CLI.

pub mod appview {
    diesel::table! {
        appview.actor (did) {
            did -> Varchar,
            handle -> Nullable<Varchar>,
            indexedat -> Varchar,
            homeserver -> Varchar,
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
            location -> Nullable<Varchar>,
            tagline -> Nullable<Varchar>,
            createdat -> Nullable<Varchar>,
            firstseen -> Varchar,
        }
    }

    diesel::table! {
        appview.profile_post (uri) {
            uri -> Varchar,
            cid -> Varchar,
            author -> Varchar,
            parenturi -> Nullable<Varchar>,
            content -> Varchar,
            replies -> Array<Nullable<Text>>,
            indexedat -> Varchar,
            createdat -> Varchar,
            updatedat -> Nullable<Varchar>,
        }
    }

    diesel::table! {
        appview.setting (name) {
            name -> Varchar,
            value -> Nullable<Varchar>,
        }
    }

    diesel::allow_tables_to_appear_in_same_query!(actor, profile, profile_post, setting,);
}
