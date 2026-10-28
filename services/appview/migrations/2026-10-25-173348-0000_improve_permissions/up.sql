-- Unfortunately, this clears permissions
-- Roles
ALTER TABLE appview.campsite_role
ADD COLUMN IF NOT EXISTS permissions integer NOT NULL
DEFAULT 0;

ALTER TABLE appview.campsite_role
DROP COLUMN IF EXISTS contentPermissions;

ALTER TABLE appview.campsite_role
DROP COLUMN IF EXISTS generalPermissions;

-- Permission
ALTER TABLE appview.campsite_permission
ADD COLUMN IF NOT EXISTS permissions bigint NOT NULL
DEFAULT 0;

ALTER TABLE appview.campsite_permission
DROP COLUMN IF EXISTS allowedGeneralPermissions;

ALTER TABLE appview.campsite_permission
DROP COLUMN IF EXISTS deniedGeneralPermissions;

ALTER TABLE appview.campsite_permission
DROP COLUMN IF EXISTS allowedContentPermissions;

ALTER TABLE appview.campsite_permission
DROP COLUMN IF EXISTS deniedContentPermissions;

ALTER TABLE appview.campsite_permission
ADD CONSTRAINT campsite_permission_unique_keys
UNIQUE (userid, roleid, tentid, categoryid, bonfireid, campsiteid);