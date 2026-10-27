-- Unfortunately, this clears permissions
-- Roles
ALTER TABLE appview.campsite_role
ADD COLUMN IF NOT EXISTS permissions integer NOT NULL
DEFAULT 0;

ALTER TABLE appview.campsite_role
DROP COLUMN generalPermissions;

ALTER TABLE appview.campsite_role
DROP COLUMN contentPermissions;

-- Permission
ALTER TABLE appview.campsite_permission
ADD COLUMN IF NOT EXISTS permissions bigint NOT NULL
DEFAULT 0;

ALTER TABLE appview.campsite_permission
DROP COLUMN allowedGeneralPermissions;

ALTER TABLE appview.campsite_permission
DROP COLUMN deniedGeneralPermissions;

ALTER TABLE appview.campsite_permission
DROP COLUMN allowedContentPermissions;

ALTER TABLE appview.campsite_permission
DROP COLUMN deniedContentPermissions;