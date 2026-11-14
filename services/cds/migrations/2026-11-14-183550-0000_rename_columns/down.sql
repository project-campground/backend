-- Actor
ALTER TABLE cds.actor
RENAME COLUMN indexed_at
TO indexedat;

-- Bonfire
ALTER TABLE cds.bonfire
RENAME COLUMN campsite
TO campsiteid;

ALTER TABLE cds.bonfire
RENAME COLUMN avatar
TO avataruri;

ALTER TABLE cds.bonfire
RENAME COLUMN banner
TO banneruri;

ALTER TABLE cds.bonfire
RENAME COLUMN created_by
TO createdby;

ALTER TABLE cds.bonfire
RENAME COLUMN created_at
TO createdat;

ALTER TABLE cds.bonfire
RENAME COLUMN updated_by
TO updatedby;

ALTER TABLE cds.bonfire
RENAME COLUMN updated_at
TO updatedat;

-- Campsite
ALTER TABLE cds.campsite
RENAME COLUMN members
TO memberdids;

ALTER TABLE cds.campsite
RENAME COLUMN vanity_url
TO vanityurl;

ALTER TABLE cds.campsite
RENAME COLUMN avatar
TO avataruri;

ALTER TABLE cds.campsite
RENAME COLUMN banner
TO banneruri;

ALTER TABLE cds.campsite
RENAME COLUMN created_by
TO createdby;

ALTER TABLE cds.campsite
RENAME COLUMN created_at
TO createdat;

ALTER TABLE cds.campsite
RENAME COLUMN updated_by
TO updatedby;

ALTER TABLE cds.campsite
RENAME COLUMN updated_at
TO updatedat;

-- Campsite ban
ALTER TABLE cds.campsite_ban
RENAME COLUMN campsite
TO campsiteid;

ALTER TABLE cds.campsite_ban
RENAME COLUMN target
TO userid;

ALTER TABLE cds.campsite_ban
RENAME COLUMN created_by
TO createdby;

ALTER TABLE cds.campsite_ban
RENAME COLUMN created_at
TO createdat;

ALTER TABLE cds.campsite_ban
RENAME COLUMN updated_by
TO updatedby;

ALTER TABLE cds.campsite_ban
RENAME COLUMN updated_at
TO updatedat;

-- Campsite invite
ALTER TABLE cds.campsite_invite
RENAME COLUMN campsite
TO campsiteid;

ALTER TABLE cds.campsite_invite
RENAME COLUMN allowed_uses
TO allowedamount;

ALTER TABLE cds.campsite_invite
RENAME COLUMN expires_at
TO expiresat;

ALTER TABLE cds.campsite_invite
RENAME COLUMN created_by
TO createdby;

ALTER TABLE cds.campsite_invite
RENAME COLUMN created_at
TO createdat;

-- Campsite member
ALTER TABLE cds.campsite_member
RENAME COLUMN actor
TO userid;

ALTER TABLE cds.campsite_member
RENAME COLUMN campsite
TO campsiteid;

ALTER TABLE cds.campsite_member
RENAME COLUMN joined_at
TO joinedat;

ALTER TABLE cds.campsite_member
RENAME COLUMN used_invite
TO usedinviteid;

-- Campsite permission
ALTER TABLE cds.permission
RENAME COLUMN campsite
TO campsiteid;

ALTER TABLE cds.permission
RENAME COLUMN bonfire
TO bonfireid;

ALTER TABLE cds.permission
RENAME COLUMN category
TO categoryid;

ALTER TABLE cds.permission
RENAME COLUMN tent
TO tentid;

ALTER TABLE cds.permission
RENAME COLUMN role
TO roleid;

ALTER TABLE cds.permission
RENAME COLUMN actor
TO userid;

ALTER TABLE cds.permission
RENAME COLUMN created_by
TO createdby;

ALTER TABLE cds.permission
RENAME COLUMN created_at
TO createdat;

ALTER TABLE cds.permission
RENAME COLUMN updated_by
TO updatedby;

ALTER TABLE cds.permission
RENAME COLUMN updated_at
TO updatedat;

-- Campsite role
ALTER TABLE cds.role
RENAME COLUMN campsite
TO campsiteid;

ALTER TABLE cds.role
RENAME COLUMN hoist
TO displayseparately;

ALTER TABLE cds.role
RENAME COLUMN pingable
TO mentionable;

ALTER TABLE cds.role
RENAME COLUMN created_by
TO createdby;

ALTER TABLE cds.role
RENAME COLUMN created_at
TO createdat;

ALTER TABLE cds.role
RENAME COLUMN updated_by
TO updatedby;

ALTER TABLE cds.role
RENAME COLUMN updated_at
TO updatedat;

-- Profile
ALTER TABLE cds.profile
RENAME COLUMN actor
TO creator;

ALTER TABLE cds.profile
RENAME COLUMN name
TO displayname;

ALTER TABLE cds.profile
RENAME COLUMN avatar
TO avatarcid;

ALTER TABLE cds.profile
RENAME COLUMN banner
TO bannercid;

ALTER TABLE cds.profile
RENAME COLUMN indexed_at
TO indexedat;

ALTER TABLE cds.profile
RENAME COLUMN created_at
TO createdat;

ALTER TABLE cds.profile
ADD COLUMN IF NOT EXISTS firstseen character varying NOT NULL
DEFAULT '';

-- Profile post
ALTER TABLE cds.profile_post
RENAME COLUMN parent
TO parenturi;

ALTER TABLE cds.profile_post
RENAME COLUMN indexed_at
TO indexedat;

ALTER TABLE cds.profile_post
RENAME COLUMN created_at
TO createdat;

ALTER TABLE cds.profile_post
RENAME COLUMN updated_at
TO updatedat;

-- Tent
ALTER TABLE cds.tent
RENAME COLUMN campsite
TO campsiteid;

ALTER TABLE cds.tent
RENAME COLUMN bonfire
TO bonfireid;

ALTER TABLE cds.tent
RENAME COLUMN category
TO categoryid;

ALTER TABLE cds.tent
RENAME COLUMN view_type
TO viewtype;

ALTER TABLE cds.tent
RENAME COLUMN created_by
TO createdby;

ALTER TABLE cds.tent
RENAME COLUMN created_at
TO createdat;

ALTER TABLE cds.tent
RENAME COLUMN updated_by
TO updatedby;

ALTER TABLE cds.tent
RENAME COLUMN updated_at
TO updatedat;

-- Tent category
ALTER TABLE cds.tent_category
RENAME COLUMN campsite
TO campsiteid;

ALTER TABLE cds.tent_category
RENAME COLUMN bonfire
TO bonfireid;

ALTER TABLE cds.tent_category
RENAME COLUMN created_by
TO createdby;

ALTER TABLE cds.tent_category
RENAME COLUMN created_at
TO createdat;

ALTER TABLE cds.tent_category
RENAME COLUMN updated_by
TO updatedby;

ALTER TABLE cds.tent_category
RENAME COLUMN updated_at
TO updatedat;

-- Messages
ALTER TABLE cds.tent_message
RENAME COLUMN campsite
TO campsiteid;

ALTER TABLE cds.tent_message
RENAME COLUMN tent
TO tentid;

ALTER TABLE cds.tent_message
RENAME COLUMN replying_to
TO replyingto;

ALTER TABLE cds.tent_message
RENAME COLUMN created_by
TO createdby;

ALTER TABLE cds.tent_message
RENAME COLUMN created_at
TO createdat;

ALTER TABLE cds.tent_message
RENAME COLUMN updated_at
TO updatedat;

-- Tables
ALTER TABLE cds.role
RENAME TO campsite_role;

ALTER TABLE cds.permission
RENAME TO campsite_permission;

ALTER TABLE cds.member_ban
RENAME TO campsite_ban;

CREATE TABLE IF NOT EXISTS cds.setting (
    name character varying PRIMARY KEY,
    value character varying
);