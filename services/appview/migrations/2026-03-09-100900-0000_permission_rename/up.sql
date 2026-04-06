-- Permissions
ALTER TABLE appview.campsite_permission RENAME COLUMN allowedCampsitePermissions TO allowedGeneralPermissions;
ALTER TABLE appview.campsite_permission RENAME COLUMN deniedCampsitePermissions TO deniedGeneralPermissions;

ALTER TABLE appview.campsite_permission RENAME COLUMN allowedTentPermissions TO allowedContentPermissions;
ALTER TABLE appview.campsite_permission RENAME COLUMN deniedTentPermissions TO deniedContentPermissions;
-- Roles
ALTER TABLE appview.campsite_role RENAME COLUMN campsitePermissions TO generalPermissions;
ALTER TABLE appview.campsite_role RENAME COLUMN tentPermissions TO contentPermissions;