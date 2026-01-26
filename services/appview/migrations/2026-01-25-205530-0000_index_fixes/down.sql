DROP INDEX IF EXISTS campsite_invite_campsite_id_idx;
DROP INDEX IF EXISTS tent_message_tent_id_idx;

ALTER TABLE appview.campsite_member
ALTER COLUMN usedInviteId TYPE character varying USING NULL;

-- Campsite foreign keys
ALTER TABLE appview.campsite_member
DROP CONSTRAINT IF EXISTS fk_campsite;

ALTER TABLE appview.campsite_invite
DROP CONSTRAINT IF EXISTS fk_campsite;

ALTER TABLE appview.campsite_ban
DROP CONSTRAINT IF EXISTS fk_campsite;

ALTER TABLE appview.campsite_permission
DROP CONSTRAINT IF EXISTS fk_campsite;

ALTER TABLE appview.campsite_role
DROP CONSTRAINT IF EXISTS fk_campsite;

ALTER TABLE appview.bonfire
DROP CONSTRAINT IF EXISTS fk_campsite;

ALTER TABLE appview.tent_category
DROP CONSTRAINT IF EXISTS fk_campsite;

ALTER TABLE appview.tent
DROP CONSTRAINT IF EXISTS fk_campsite;

ALTER TABLE appview.tent_message
DROP CONSTRAINT IF EXISTS fk_tent;

ALTER TABLE appview.campsite_permission
DROP CONSTRAINT IF EXISTS fk_tent;

ALTER TABLE appview.campsite_permission
DROP CONSTRAINT IF EXISTS fk_category;