-- This unfortunately clears permissions
-- Roles
ALTER TABLE appview.campsite_role
ADD COLUMN IF NOT EXISTS generalPermissions bigint NOT NULL
DEFAULT 0;

ALTER TABLE appview.campsite_role
ADD COLUMN IF NOT EXISTS contentPermissions bigint NOT NULL
DEFAULT 0;

ALTER TABLE appview.campsite_role
DROP COLUMN IF EXISTS permissions;

-- Permissions
ALTER TABLE appview.campsite_permission
ADD COLUMN IF NOT EXISTS allowedGeneralPermissions bigint NOT NULL
DEFAULT 0;

ALTER TABLE appview.campsite_permission
ADD COLUMN IF NOT EXISTS allowedContentPermissions bigint NOT NULL
DEFAULT 0;

ALTER TABLE appview.campsite_permission
ADD COLUMN IF NOT EXISTS deniedGeneralPermissions bigint NOT NULL
DEFAULT 0;

ALTER TABLE appview.campsite_permission
ADD COLUMN IF NOT EXISTS deniedContentPermissions bigint NOT NULL
DEFAULT 0;

ALTER TABLE appview.campsite_permission
DROP COLUMN permissions;

ALTER TABLE appview.campsite_permission
DROP CONSTRAINT IF EXISTS campsite_permission_unique_keys;