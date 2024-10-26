// @generated automatically by Diesel CLI.

pub mod registry {
    diesel::table! {
        registry.account (did) {
            did -> Varchar,
            email -> Varchar,
            recoveryKey -> Nullable<Varchar>,
            password -> Varchar,
            createdAt -> Varchar,
            emailConfirmedAt -> Nullable<Varchar>,
        }
    }

    diesel::table! {
        registry.account_pref (id) {
            id -> Int4,
            did -> Varchar,
            name -> Varchar,
            valueJson -> Nullable<Text>,
        }
    }

    diesel::table! {
        registry.actor (did) {
            did -> Varchar,
            handle -> Nullable<Varchar>,
            createdAt -> Varchar,
            deactivatedAt -> Nullable<Varchar>,
            deleteAfter -> Nullable<Varchar>,
            takedownRef -> Nullable<Varchar>,
        }
    }

    diesel::table! {
        registry.app_password (did, name) {
            did -> Varchar,
            name -> Varchar,
            password -> Varchar,
            createdAt -> Varchar,
        }
    }

    diesel::table! {
        registry.backlink (uri, path) {
            uri -> Varchar,
            path -> Varchar,
            linkTo -> Varchar,
        }
    }

    diesel::table! {
        registry.blob (cid, did) {
            cid -> Varchar,
            did -> Varchar,
            mimeType -> Varchar,
            size -> Int4,
            tempKey -> Nullable<Varchar>,
            width -> Nullable<Int4>,
            height -> Nullable<Int4>,
            createdAt -> Varchar,
            takedownRef -> Nullable<Varchar>,
        }
    }

    diesel::table! {
        registry.did_doc (did) {
            did -> Varchar,
            doc -> Text,
            updatedAt -> Int8,
        }
    }

    diesel::table! {
        registry.record (uri) {
            uri -> Varchar,
            cid -> Varchar,
            did -> Varchar,
            collection -> Varchar,
            rkey -> Varchar,
            repoRev -> Nullable<Varchar>,
            indexedAt -> Varchar,
            takedownRef -> Nullable<Varchar>,
        }
    }

    diesel::table! {
        registry.record_blob (blobCid, recordUri) {
            blobCid -> Varchar,
            recordUri -> Varchar,
            did -> Varchar,
        }
    }

    diesel::table! {
        registry.refresh_token (id) {
            id -> Varchar,
            did -> Varchar,
            expiresAt -> Varchar,
            nextId -> Nullable<Varchar>,
            appPasswordName -> Nullable<Varchar>,
        }
    }

    diesel::table! {
        registry.repo_block (cid, did) {
            cid -> Varchar,
            did -> Varchar,
            repoRev -> Varchar,
            size -> Int4,
            content -> Bytea,
        }
    }

    diesel::table! {
        registry.repo_root (did) {
            did -> Varchar,
            cid -> Varchar,
            rev -> Varchar,
            indexedAt -> Varchar,
        }
    }

    diesel::table! {
        registry.repo_seq (seq) {
            seq -> Int8,
            did -> Varchar,
            eventType -> Varchar,
            event -> Bytea,
            invalidated -> Int2,
            sequencedAt -> Varchar,
        }
    }

    diesel::allow_tables_to_appear_in_same_query!(
        account,
        account_pref,
        actor,
        app_password,
        backlink,
        blob,
        did_doc,
        record,
        record_blob,
        refresh_token,
        repo_block,
        repo_root,
        repo_seq,
    );
}
