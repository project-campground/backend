-- Actor
ALTER TABLE cds.actor
RENAME COLUMN indexedat
TO indexed_at;

-- Bonfire
ALTER TABLE cds.bonfire
RENAME COLUMN campsiteid
TO campsite;

ALTER TABLE cds.bonfire
RENAME COLUMN avataruri
TO avatar;

ALTER TABLE cds.bonfire
RENAME COLUMN banneruri
TO banner;

ALTER TABLE cds.bonfire
RENAME COLUMN createdby
TO created_by;

ALTER TABLE cds.bonfire
RENAME COLUMN createdat
TO created_at;

ALTER TABLE cds.bonfire
RENAME COLUMN updatedby
TO updated_by;

ALTER TABLE cds.bonfire
RENAME COLUMN updatedat
TO updated_at;

-- Campsite
ALTER TABLE cds.campsite
RENAME COLUMN memberdids
TO members;

ALTER TABLE cds.campsite
RENAME COLUMN vanityurl
TO vanity_url;

ALTER TABLE cds.campsite
RENAME COLUMN avataruri
TO avatar;

ALTER TABLE cds.campsite
RENAME COLUMN banneruri
TO banner;

ALTER TABLE cds.campsite
RENAME COLUMN createdby
TO created_by;

ALTER TABLE cds.campsite
RENAME COLUMN createdat
TO created_at;

ALTER TABLE cds.campsite
RENAME COLUMN updatedby
TO updated_by;

ALTER TABLE cds.campsite
RENAME COLUMN updatedat
TO updated_at;

-- Campsite ban
ALTER TABLE cds.campsite_ban
RENAME COLUMN campsiteid
TO campsite;

ALTER TABLE cds.campsite_ban
RENAME COLUMN userid
TO target;

ALTER TABLE cds.campsite_ban
RENAME COLUMN createdby
TO created_by;

ALTER TABLE cds.campsite_ban
RENAME COLUMN createdat
TO created_at;

ALTER TABLE cds.campsite_ban
RENAME COLUMN updatedby
TO updated_by;

ALTER TABLE cds.campsite_ban
RENAME COLUMN updatedat
TO updated_at;

-- Campsite invite
ALTER TABLE cds.campsite_invite
RENAME COLUMN campsiteid
TO campsite;

ALTER TABLE cds.campsite_invite
RENAME COLUMN allowedamount
TO allowed_uses;

ALTER TABLE cds.campsite_invite
RENAME COLUMN expiresat
TO expires_at;

ALTER TABLE cds.campsite_invite
RENAME COLUMN createdby
TO created_by;

ALTER TABLE cds.campsite_invite
RENAME COLUMN createdat
TO created_at;

-- Campsite member
ALTER TABLE cds.campsite_member
RENAME COLUMN userid
TO actor;

ALTER TABLE cds.campsite_member
RENAME COLUMN campsiteid
TO campsite;

ALTER TABLE cds.campsite_member
RENAME COLUMN joinedat
TO joined_at;

ALTER TABLE cds.campsite_member
RENAME COLUMN usedinviteid
TO used_invite;

-- Campsite permission
ALTER TABLE cds.campsite_permission
RENAME COLUMN campsiteid
TO campsite;

ALTER TABLE cds.campsite_permission
RENAME COLUMN bonfireid
TO bonfire;

ALTER TABLE cds.campsite_permission
RENAME COLUMN categoryid
TO category;

ALTER TABLE cds.campsite_permission
RENAME COLUMN tentid
TO tent;

ALTER TABLE cds.campsite_permission
RENAME COLUMN roleid
TO role;

ALTER TABLE cds.campsite_permission
RENAME COLUMN userid
TO actor;

ALTER TABLE cds.campsite_permission
RENAME COLUMN createdby
TO created_by;

ALTER TABLE cds.campsite_permission
RENAME COLUMN createdat
TO created_at;

ALTER TABLE cds.campsite_permission
RENAME COLUMN updatedby
TO updated_by;

ALTER TABLE cds.campsite_permission
RENAME COLUMN updatedat
TO updated_at;

-- Campsite role
ALTER TABLE cds.campsite_role
RENAME COLUMN campsiteid
TO campsite;

ALTER TABLE cds.campsite_role
RENAME COLUMN displayseparately
TO hoist;

ALTER TABLE cds.campsite_role
RENAME COLUMN mentionable
TO pingable;

ALTER TABLE cds.campsite_role
RENAME COLUMN createdby
TO created_by;

ALTER TABLE cds.campsite_role
RENAME COLUMN createdat
TO created_at;

ALTER TABLE cds.campsite_role
RENAME COLUMN updatedby
TO updated_by;

ALTER TABLE cds.campsite_role
RENAME COLUMN updatedat
TO updated_at;

-- Profile
ALTER TABLE cds.profile
RENAME COLUMN creator
TO actor;

ALTER TABLE cds.profile
RENAME COLUMN displayname
TO name;

ALTER TABLE cds.profile
RENAME COLUMN avatarcid
TO avatar;

ALTER TABLE cds.profile
RENAME COLUMN bannercid
TO banner;

ALTER TABLE cds.profile
RENAME COLUMN indexedat
TO indexed_at;

ALTER TABLE cds.profile
RENAME COLUMN createdat
TO created_at;

ALTER TABLE cds.profile
DROP COLUMN IF EXISTS firstseen;

-- Profile post
ALTER TABLE cds.profile_post
RENAME COLUMN parenturi
TO parent;

ALTER TABLE cds.profile_post
RENAME COLUMN indexedat
TO indexed_at;

ALTER TABLE cds.profile_post
RENAME COLUMN createdat
TO created_at;

ALTER TABLE cds.profile_post
RENAME COLUMN updatedat
TO updated_at;

-- Tent
ALTER TABLE cds.tent
RENAME COLUMN campsiteid
TO campsite;

ALTER TABLE cds.tent
RENAME COLUMN bonfireid
TO bonfire;

ALTER TABLE cds.tent
RENAME COLUMN categoryid
TO category;

ALTER TABLE cds.tent
RENAME COLUMN viewtype
TO view_type;

ALTER TABLE cds.tent
RENAME COLUMN createdby
TO created_by;

ALTER TABLE cds.tent
RENAME COLUMN createdat
TO created_at;

ALTER TABLE cds.tent
RENAME COLUMN updatedby
TO updated_by;

ALTER TABLE cds.tent
RENAME COLUMN updatedat
TO updated_at;

-- Tent category
ALTER TABLE cds.tent_category
RENAME COLUMN campsiteid
TO campsite;

ALTER TABLE cds.tent_category
RENAME COLUMN bonfireid
TO bonfire;

ALTER TABLE cds.tent_category
RENAME COLUMN createdby
TO created_by;

ALTER TABLE cds.tent_category
RENAME COLUMN createdat
TO created_at;

ALTER TABLE cds.tent_category
RENAME COLUMN updatedby
TO updated_by;

ALTER TABLE cds.tent_category
RENAME COLUMN updatedat
TO updated_at;

-- Messages
ALTER TABLE cds.tent_message
RENAME COLUMN campsiteid
TO campsite;

ALTER TABLE cds.tent_message
RENAME COLUMN tentid
TO tent;

ALTER TABLE cds.tent_message
RENAME COLUMN replyingto
TO replying_to;

ALTER TABLE cds.tent_message
RENAME COLUMN createdby
TO created_by;

ALTER TABLE cds.tent_message
RENAME COLUMN createdat
TO created_at;

ALTER TABLE cds.tent_message
RENAME COLUMN updatedat
TO updated_at;

-- Tables
ALTER TABLE cds.campsite_role
RENAME TO role;

ALTER TABLE cds.campsite_permission
RENAME TO permission;

ALTER TABLE cds.campsite_ban
RENAME TO member_ban;

DROP TABLE IF EXISTS cds.setting;