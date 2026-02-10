// @generated automatically by Diesel CLI.

pub mod appview {
    diesel::table! {
        appview.actor (did) {
            did -> Varchar,
            handle -> Nullable<Varchar>,
            indexedat -> Varchar,
            homeserver -> Varchar,
            campsites -> Array<Nullable<Text>>,
        }
    }

    diesel::table! {
        appview.bonfire (id) {
            id -> Varchar,
            campsiteid -> Varchar,
            name -> Varchar,
            description -> Varchar,
            avataruri -> Nullable<Varchar>,
            banneruri -> Nullable<Varchar>,
            priority -> Int4,
            createdby -> Varchar,
            createdat -> Timestamp,
            updatedby -> Varchar,
            updatedat -> Timestamp,
        }
    }

    diesel::table! {
        appview.campsite (id) {
            id -> Varchar,
            name -> Varchar,
            vanityurl -> Nullable<Varchar>,
            description -> Varchar,
            avataruri -> Nullable<Varchar>,
            banneruri -> Nullable<Varchar>,
            tags -> Array<Nullable<Text>>,
            memberdids -> Array<Nullable<Text>>,
            owner -> Varchar,
            createdby -> Varchar,
            createdat -> Timestamp,
            updatedby -> Varchar,
            updatedat -> Timestamp,
        }
    }

    diesel::table! {
        appview.campsite_ban (campsiteid, userid) {
            campsiteid -> Varchar,
            userid -> Varchar,
            reason -> Nullable<Varchar>,
            createdby -> Varchar,
            createdat -> Timestamp,
            updatedby -> Varchar,
            updatedat -> Timestamp,
        }
    }

    diesel::table! {
        appview.campsite_invite (id) {
            id -> Uuid,
            campsiteid -> Varchar,
            allowedamount -> Nullable<Int4>,
            expiresat -> Nullable<Timestamp>,
            createdby -> Varchar,
            createdat -> Timestamp,
            used -> Int4,
        }
    }

    diesel::table! {
        appview.campsite_member (userid, campsiteid) {
            userid -> Varchar,
            campsiteid -> Varchar,
            joinedat -> Timestamp,
            usedinviteid -> Nullable<Uuid>,
            nickname -> Nullable<Varchar>,
            roles -> Array<Nullable<Uuid>>,
        }
    }

    diesel::table! {
        appview.campsite_permission (id) {
            id -> Uuid,
            campsiteid -> Varchar,
            roleid -> Nullable<Uuid>,
            userid -> Nullable<Varchar>,
            bonfireid -> Nullable<Varchar>,
            categoryid -> Nullable<Uuid>,
            tentid -> Nullable<Uuid>,
            allowedcampsitepermissions -> Int8,
            deniedcampsitepermissions -> Int8,
            allowedtentpermissions -> Int8,
            deniedtentpermissions -> Int8,
            createdby -> Varchar,
            createdat -> Timestamp,
            updatedby -> Varchar,
            updatedat -> Timestamp,
        }
    }

    diesel::table! {
        appview.campsite_role (id) {
            id -> Uuid,
            campsiteid -> Varchar,
            name -> Varchar,
            displayseparately -> Bool,
            mentionable -> Bool,
            campsitepermissions -> Int8,
            tentpermissions -> Int8,
            color -> Int4,
            colorsecondary -> Int4,
            priority -> Int4,
            createdby -> Varchar,
            createdat -> Timestamp,
            updatedby -> Varchar,
            updatedat -> Timestamp,
            members -> Array<Nullable<Text>>,
            flags -> Int4,
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

    diesel::table! {
        appview.tent (id) {
            id -> Uuid,
            campsiteid -> Varchar,
            bonfireid -> Varchar,
            categoryid -> Nullable<Uuid>,
            name -> Varchar,
            #[sql_name = "type"]
            type_ -> Int4,
            viewtype -> Int4,
            description -> Varchar,
            priority -> Int4,
            createdby -> Varchar,
            createdat -> Timestamp,
            updatedby -> Varchar,
            updatedat -> Timestamp,
        }
    }

    diesel::table! {
        appview.tent_category (id) {
            id -> Uuid,
            campsiteid -> Varchar,
            bonfireid -> Varchar,
            name -> Varchar,
            description -> Varchar,
            priority -> Int4,
            createdby -> Varchar,
            createdat -> Timestamp,
            updatedby -> Varchar,
            updatedat -> Timestamp,
        }
    }

    diesel::table! {
        appview.tent_message (id) {
            id -> Uuid,
            campsiteid -> Varchar,
            tentid -> Uuid,
            content -> Varchar,
            replyingto -> Array<Nullable<Uuid>>,
            createdby -> Varchar,
            createdat -> Timestamp,
            updatedat -> Nullable<Timestamp>,
        }
    }

    diesel::joinable!(bonfire -> campsite (campsiteid));
    diesel::joinable!(campsite_ban -> campsite (campsiteid));
    diesel::joinable!(campsite_invite -> campsite (campsiteid));
    diesel::joinable!(campsite_member -> campsite (campsiteid));
    diesel::joinable!(campsite_permission -> bonfire (bonfireid));
    diesel::joinable!(campsite_permission -> campsite (campsiteid));
    diesel::joinable!(campsite_permission -> tent (tentid));
    diesel::joinable!(campsite_permission -> tent_category (categoryid));
    diesel::joinable!(campsite_role -> campsite (campsiteid));
    diesel::joinable!(tent -> campsite (campsiteid));
    diesel::joinable!(tent_category -> campsite (campsiteid));
    diesel::joinable!(tent_message -> tent (tentid));

    diesel::allow_tables_to_appear_in_same_query!(
        actor,
        bonfire,
        campsite,
        campsite_ban,
        campsite_invite,
        campsite_member,
        campsite_permission,
        campsite_role,
        profile,
        profile_post,
        setting,
        tent,
        tent_category,
        tent_message,
    );
}
