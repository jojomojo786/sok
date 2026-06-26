-- PornsOK replica — catalog search/thumbnail alignment (sok-replica.3.8)
--
-- Problem: the live Aiven catalog was provisioned from the original
-- `0001_catalog_schema.sql`, which named entity columns `display_name` /
-- `thumb_url` / `view_count` (videos) and omitted `week_views`. The Rust
-- `sqlx` models, however, query `pornstars.thumb_path`, `channels.title`,
-- `*.week_views`, and the `videos` columns `views` / `status` / `wide_thumb`.
-- Header autocomplete (`/ajax/search_help`) and the in-page entity search
-- therefore failed with `Unknown column 'p.thumb_path' in 'field list'` and
-- silently fell back to bundled fixtures.
--
-- This migration aligns a legacy (`0001`-shaped) catalog to the model schema
-- WITHOUT dropping data. Every statement is additive and idempotent: column
-- adds are guarded against `information_schema`, and backfills only touch rows
-- that are still empty. It is safe to run repeatedly and safe against an
-- already-aligned database. Legacy columns are retained; new columns are
-- backfilled from them where present.
--
-- Compatibility: written as flat statements (no stored procedures, no custom
-- DELIMITER) so it applies cleanly under a real MySQL client AND under the
-- repo's simple `;`-splitting runner (`execute_sql_script`). Each guarded ADD
-- builds its DDL from `information_schema` and runs via PREPARE/EXECUTE.
--
-- Apply per docs/catalog-schema.md. Do not edit `0001_catalog_schema.sql` in
-- place; this is the `0002_*` alignment migration the docs call for.

SET NAMES utf8mb4;
SET time_zone = '+00:00';

-- pornstars.thumb_path -------------------------------------------------------
SET @ddl = IF(
    NOT EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'pornstars' AND COLUMN_NAME = 'thumb_path'),
    "ALTER TABLE pornstars ADD COLUMN thumb_path VARCHAR(512) NOT NULL DEFAULT ''",
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- pornstars.week_views -------------------------------------------------------
SET @ddl = IF(
    NOT EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'pornstars' AND COLUMN_NAME = 'week_views'),
    'ALTER TABLE pornstars ADD COLUMN week_views BIGINT UNSIGNED NOT NULL DEFAULT 0',
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- pornstars.thumb_path <- legacy thumb_url (only when both columns exist) -----
SET @ddl = IF(
    EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'pornstars' AND COLUMN_NAME = 'thumb_path')
    AND EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'pornstars' AND COLUMN_NAME = 'thumb_url'),
    "UPDATE pornstars SET thumb_path = COALESCE(thumb_url, '') WHERE (thumb_path = '' OR thumb_path IS NULL) AND thumb_url IS NOT NULL",
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- channels.title -------------------------------------------------------------
SET @ddl = IF(
    NOT EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'channels' AND COLUMN_NAME = 'title'),
    "ALTER TABLE channels ADD COLUMN title VARCHAR(255) NOT NULL DEFAULT ''",
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- channels.thumb_path --------------------------------------------------------
SET @ddl = IF(
    NOT EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'channels' AND COLUMN_NAME = 'thumb_path'),
    "ALTER TABLE channels ADD COLUMN thumb_path VARCHAR(512) NOT NULL DEFAULT ''",
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- channels.week_views --------------------------------------------------------
SET @ddl = IF(
    NOT EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'channels' AND COLUMN_NAME = 'week_views'),
    'ALTER TABLE channels ADD COLUMN week_views BIGINT UNSIGNED NOT NULL DEFAULT 0',
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- channels.title <- legacy display_name --------------------------------------
SET @ddl = IF(
    EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'channels' AND COLUMN_NAME = 'title')
    AND EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'channels' AND COLUMN_NAME = 'display_name'),
    "UPDATE channels SET title = display_name WHERE (title = '' OR title IS NULL) AND display_name IS NOT NULL",
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- channels.thumb_path <- legacy thumb_url ------------------------------------
SET @ddl = IF(
    EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'channels' AND COLUMN_NAME = 'thumb_path')
    AND EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'channels' AND COLUMN_NAME = 'thumb_url'),
    "UPDATE channels SET thumb_path = COALESCE(thumb_url, '') WHERE (thumb_path = '' OR thumb_path IS NULL) AND thumb_url IS NOT NULL",
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- videos.views ---------------------------------------------------------------
SET @ddl = IF(
    NOT EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'videos' AND COLUMN_NAME = 'views'),
    'ALTER TABLE videos ADD COLUMN views BIGINT UNSIGNED NOT NULL DEFAULT 0',
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- videos.wide_thumb ----------------------------------------------------------
SET @ddl = IF(
    NOT EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'videos' AND COLUMN_NAME = 'wide_thumb'),
    'ALTER TABLE videos ADD COLUMN wide_thumb TINYINT(1) NOT NULL DEFAULT 1',
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- videos.status --------------------------------------------------------------
SET @ddl = IF(
    NOT EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'videos' AND COLUMN_NAME = 'status'),
    "ALTER TABLE videos ADD COLUMN status ENUM('published','hidden') NOT NULL DEFAULT 'published'",
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- videos.views <- legacy view_count ------------------------------------------
SET @ddl = IF(
    EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'videos' AND COLUMN_NAME = 'views')
    AND EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'videos' AND COLUMN_NAME = 'view_count'),
    'UPDATE videos SET views = view_count WHERE views = 0 AND view_count <> 0',
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- videos.wide_thumb <- legacy is_wide_thumb ----------------------------------
SET @ddl = IF(
    EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'videos' AND COLUMN_NAME = 'wide_thumb')
    AND EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'videos' AND COLUMN_NAME = 'is_wide_thumb'),
    'UPDATE videos SET wide_thumb = is_wide_thumb',
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- videos.status <- legacy is_active ------------------------------------------
SET @ddl = IF(
    EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'videos' AND COLUMN_NAME = 'status')
    AND EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'videos' AND COLUMN_NAME = 'is_active'),
    "UPDATE videos SET status = CASE WHEN is_active = 0 THEN 'hidden' ELSE 'published' END",
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- Alias tables used by the search LEFT JOINs (created if absent) -------------
CREATE TABLE IF NOT EXISTS pornstar_aliases (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    pornstar_id BIGINT UNSIGNED NOT NULL,
    alias VARCHAR(255) NOT NULL,
    UNIQUE KEY uq_pornstar_alias (pornstar_id, alias),
    KEY idx_pornstar_alias_lookup (alias),
    CONSTRAINT fk_pornstar_aliases_pornstar
        FOREIGN KEY (pornstar_id) REFERENCES pornstars (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS channel_aliases (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    channel_id BIGINT UNSIGNED NOT NULL,
    alias VARCHAR(255) NOT NULL,
    UNIQUE KEY uq_channel_alias (channel_id, alias),
    KEY idx_channel_alias_lookup (alias),
    CONSTRAINT fk_channel_aliases_channel
        FOREIGN KEY (channel_id) REFERENCES channels (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
