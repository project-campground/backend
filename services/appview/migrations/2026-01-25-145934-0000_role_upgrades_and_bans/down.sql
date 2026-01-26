DROP TABLE appview.campsite_ban;
ALTER TABLE appview.campsite_role
DROP COLUMN flags;
ALTER TABLE appview.campsite_role
DROP COLUMN members;
ALTER TABLE appview.campsite_invite
DROP COLUMN used;