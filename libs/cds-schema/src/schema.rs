// @generated automatically by Diesel CLI.

pub mod cds {
    diesel::table! {
        cds.actor (did) {
            did -> Varchar,
            handle -> Nullable<Varchar>,
            indexed_at -> Varchar,
            campsites -> Array<Nullable<Text>>,
        }
    }

    diesel::table! {
        cds.bonfire (id) {
            id -> Varchar,
            campsite -> Varchar,
            name -> Varchar,
            description -> Varchar,
            avatar -> Nullable<Varchar>,
            banner -> Nullable<Varchar>,
            priority -> Int4,
            created_by -> Varchar,
            created_at -> Timestamp,
            updated_by -> Varchar,
            updated_at -> Timestamp,
            home -> Bool,
        }
    }

    diesel::table! {
        cds.campsite (id) {
            id -> Varchar,
            name -> Varchar,
            vanity_url -> Nullable<Varchar>,
            description -> Varchar,
            avatar -> Nullable<Varchar>,
            banner -> Nullable<Varchar>,
            tags -> Array<Nullable<Text>>,
            members -> Array<Nullable<Text>>,
            owner -> Varchar,
            created_by -> Varchar,
            created_at -> Timestamp,
            updated_by -> Varchar,
            updated_at -> Timestamp,
        }
    }

    diesel::table! {
        cds.campsite_invite (id) {
            id -> Uuid,
            campsite -> Varchar,
            allowed_uses -> Nullable<Int4>,
            expires_at -> Nullable<Timestamp>,
            created_by -> Varchar,
            created_at -> Timestamp,
            used -> Int4,
        }
    }

    diesel::table! {
        cds.campsite_member (actor, campsite) {
            actor -> Varchar,
            campsite -> Varchar,
            joined_at -> Timestamp,
            used_invite -> Nullable<Uuid>,
            nickname -> Nullable<Varchar>,
            roles -> Array<Nullable<Uuid>>,
        }
    }

    diesel::table! {
        cds.member_ban (campsite, target) {
            campsite -> Varchar,
            target -> Varchar,
            reason -> Nullable<Varchar>,
            created_by -> Varchar,
            created_at -> Timestamp,
            updated_by -> Varchar,
            updated_at -> Timestamp,
        }
    }

    diesel::table! {
        cds.permission (id) {
            id -> Uuid,
            campsite -> Varchar,
            role -> Nullable<Uuid>,
            actor -> Nullable<Varchar>,
            bonfire -> Varchar,
            category -> Nullable<Uuid>,
            tent -> Nullable<Uuid>,
            created_by -> Varchar,
            created_at -> Timestamp,
            updated_by -> Varchar,
            updated_at -> Timestamp,
            permissions -> Int8,
        }
    }

    diesel::table! {
        cds.profile (uri) {
            uri -> Varchar,
            cid -> Varchar,
            actor -> Varchar,
            name -> Nullable<Varchar>,
            description -> Nullable<Varchar>,
            avatar -> Nullable<Varchar>,
            banner -> Nullable<Varchar>,
            indexed_at -> Varchar,
            location -> Nullable<Varchar>,
            tagline -> Nullable<Varchar>,
            created_at -> Nullable<Varchar>,
        }
    }

    diesel::table! {
        cds.profile_post (uri) {
            uri -> Varchar,
            cid -> Varchar,
            author -> Varchar,
            parent -> Nullable<Varchar>,
            content -> Varchar,
            replies -> Array<Nullable<Text>>,
            indexed_at -> Varchar,
            created_at -> Varchar,
            updated_at -> Nullable<Varchar>,
        }
    }

    diesel::table! {
        cds.role (id) {
            id -> Uuid,
            campsite -> Varchar,
            name -> Varchar,
            hoist -> Bool,
            pingable -> Bool,
            priority -> Int4,
            created_by -> Varchar,
            created_at -> Timestamp,
            updated_by -> Varchar,
            updated_at -> Timestamp,
            members -> Array<Nullable<Text>>,
            flags -> Int4,
            colors -> Array<Nullable<Int4>>,
            motion -> Int2,
            permissions -> Int4,
        }
    }

    diesel::table! {
        cds.tent (id) {
            id -> Uuid,
            campsite -> Varchar,
            bonfire -> Varchar,
            category -> Nullable<Uuid>,
            name -> Varchar,
            #[sql_name = "type"]
            type_ -> Int2,
            view_type -> Int2,
            description -> Varchar,
            priority -> Int4,
            created_by -> Varchar,
            created_at -> Timestamp,
            updated_by -> Varchar,
            updated_at -> Timestamp,
        }
    }

    diesel::table! {
        cds.tent_category (id) {
            id -> Uuid,
            campsite -> Varchar,
            bonfire -> Varchar,
            name -> Varchar,
            description -> Varchar,
            priority -> Int4,
            created_by -> Varchar,
            created_at -> Timestamp,
            updated_by -> Varchar,
            updated_at -> Timestamp,
        }
    }

    diesel::table! {
        cds.tent_message (id) {
            id -> Uuid,
            campsite -> Varchar,
            tent -> Uuid,
            content -> Varchar,
            replying_to -> Array<Nullable<Uuid>>,
            created_by -> Varchar,
            created_at -> Timestamp,
            updated_at -> Nullable<Timestamp>,
            components -> Array<Nullable<Jsonb>>,
            #[sql_name = "type"]
            type_ -> Int2,
        }
    }

    diesel::joinable!(bonfire -> campsite (campsite));
    diesel::joinable!(campsite_invite -> campsite (campsite));
    diesel::joinable!(campsite_member -> campsite (campsite));
    diesel::joinable!(member_ban -> campsite (campsite));
    diesel::joinable!(permission -> bonfire (bonfire));
    diesel::joinable!(permission -> campsite (campsite));
    diesel::joinable!(permission -> tent (tent));
    diesel::joinable!(permission -> tent_category (category));
    diesel::joinable!(role -> campsite (campsite));
    diesel::joinable!(tent -> bonfire (bonfire));
    diesel::joinable!(tent -> campsite (campsite));
    diesel::joinable!(tent_category -> campsite (campsite));
    diesel::joinable!(tent_message -> tent (tent));

    diesel::allow_tables_to_appear_in_same_query!(
        actor,
        bonfire,
        campsite,
        campsite_invite,
        campsite_member,
        member_ban,
        permission,
        profile,
        profile_post,
        role,
        tent,
        tent_category,
        tent_message,
    );
}
