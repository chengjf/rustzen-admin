-- ============================================================================
-- Module: Remove dictionary feature
-- Description: Drop dictionary table and remove built-in menu permissions/data.
-- ============================================================================

DELETE FROM role_menus
WHERE menu_id IN (
    SELECT id FROM menus WHERE code LIKE 'system:dict:%'
);

DELETE FROM menus
WHERE code LIKE 'system:dict:%';

DROP TRIGGER IF EXISTS update_dicts_updated_at ON dicts;
DROP TABLE IF EXISTS dicts;
