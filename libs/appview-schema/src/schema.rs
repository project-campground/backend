// @generated automatically by Diesel CLI.

pub mod appview {
    diesel::table! {
        appview.account (did) {
            did -> Varchar,
            settings -> Jsonb,
            socialconnections -> Jsonb,
            activities -> Jsonb,
            status -> Varchar,
            statustext -> Nullable<Varchar>,
            statusemoji -> Nullable<Varchar>,
            lastseen -> Timestamp,
            email -> Nullable<Varchar>,
        }
    }

    diesel::table! {
        appview.actor (did) {
            did -> Varchar,
            handle -> Nullable<Varchar>,
            indexedat -> Varchar,
            homeserver -> Varchar,
        }
    }

    diesel::table! {
        appview.block (id) {
            id -> Int8,
            did -> Varchar,
            targetdid -> Varchar,
        }
    }

    diesel::table! {
        appview.event_seq (seq) {
            seq -> Int8,
            homeserver -> Varchar,
            eventtype -> Varchar,
            event -> Jsonb,
            invalidated -> Int2,
            sequencedat -> Timestamp,
        }
    }

    diesel::table! {
        appview.friend (id) {
            id -> Int8,
            did -> Varchar,
            targetdid -> Varchar,
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
            tagline -> Nullable<Varchar>,
            createdat -> Nullable<Varchar>,
            firstseen -> Varchar,
            location -> Nullable<Varchar>,
        }
    }

    diesel::table! {
        appview.setting (name) {
            name -> Varchar,
            value -> Nullable<Varchar>,
        }
    }

    diesel::allow_tables_to_appear_in_same_query!(
        account,
        actor,
        block,
        event_seq,
        friend,
        profile,
        setting,
    );
}
