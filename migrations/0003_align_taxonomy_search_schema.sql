SET NAMES utf8mb4 /*
PornsOK replica -- taxonomy search schema alignment (sok-replica.3.9)

Problem: the live Aiven taxonomy was provisioned before 001_taxonomy.sql
grew its current shape, and 001_taxonomy.sql only uses CREATE TABLE IF NOT
EXISTS, so it never altered the pre-existing legacy categories / tags
tables. The Rust sqlx model src/models/taxonomy.rs, however, filters on
categories.is_active / tags.is_active, selects categories.intro_html and
tags.weekly_views, and EXISTS-joins taxonomy_search_aliases. The in-page
category/tag search (POST /ajax/search_cats_tags_queries) therefore failed
with "Unknown column 'is_active' in 'where clause'" and silently fell back
to the bundled fixtures.

This migration aligns a legacy taxonomy to the model schema WITHOUT
dropping data. Every statement is additive and idempotent: column adds are
guarded against information_schema, and backfills only touch rows that are
still at the column default. It is safe to run repeatedly and safe against
an already-aligned database. No legacy columns are removed.

Backfill rationale (sok-replica.3.9):
- categories.is_active / tags.is_active default to 1. Legacy is_featured /
  is_top are NOT active-state equivalents (featured/top is a ranking flag,
  not a visibility flag), so they are deliberately NOT used as the active
  source. Every legacy row is treated as active, matching the model's
  WHERE is_active = 1 intent for an already-published catalog.
- categories.intro_html defaults to NULL (optional long-form copy with no
  legacy source column to backfill from).
- tags.weekly_views defaults to 0. There is no legacy weekly-views source.
  video_count is a lifetime total, not a weekly metric, so using it would
  misrepresent the ranking. 0 is the honest default until a rollup job
  populates it.

Compatibility: written as flat statements (no stored procedures, no custom
DELIMITER). Per-statement documentation uses inline block comments placed
AFTER the first executable token so the migration applies cleanly under a
real MySQL client AND under the repo's simple split-on-semicolon runner
(execute_sql_script), which skips any chunk whose trimmed text starts with
a line comment. Each guarded ADD builds its DDL from information_schema and
runs via PREPARE/EXECUTE. Dynamic SQL uses single-quoted literals with
doubled single quotes for escaping, never double quotes, so it stays valid
under Aiven/MySQL sessions where ANSI_QUOTES may treat "..." as an
identifier.

Apply path: run via a MySQL text-protocol client (the docs Option A
mysql CLI, or an equivalent text-protocol client), exactly like 0002. The PREPARE/EXECUTE guard
relies on the text protocol. MySQL rejects PREPARE/EXECUTE/DEALLOCATE over
the binary prepared-statement protocol (error 1295), so this file is not
meant to be streamed through execute_sql_script (which is only wired to the
dev/test 001_taxonomy.sql, already column-complete). The splitter
compatibility above only guarantees the statements survive that splitter
intact if it is ever used. It does not change the required apply path.

Apply per docs/local-development.md and docs/catalog-schema.md. Do not edit
001_taxonomy.sql in place. This is the 0003_* alignment migration the docs
call for.
*/;

SET time_zone = '+00:00';

SET @ddl = /* categories.intro_html: add when missing */ IF(
    NOT EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'categories' AND COLUMN_NAME = 'intro_html'),
    'ALTER TABLE categories ADD COLUMN intro_html TEXT NULL',
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @ddl = /* categories.is_active: add when missing */ IF(
    NOT EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'categories' AND COLUMN_NAME = 'is_active'),
    'ALTER TABLE categories ADD COLUMN is_active TINYINT(1) NOT NULL DEFAULT 1',
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @ddl = /* tags.weekly_views: add when missing */ IF(
    NOT EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'tags' AND COLUMN_NAME = 'weekly_views'),
    'ALTER TABLE tags ADD COLUMN weekly_views BIGINT UNSIGNED NOT NULL DEFAULT 0',
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @ddl = /* tags.is_active: add when missing */ IF(
    NOT EXISTS (SELECT 1 FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'tags' AND COLUMN_NAME = 'is_active'),
    'ALTER TABLE tags ADD COLUMN is_active TINYINT(1) NOT NULL DEFAULT 1',
    'SELECT 1');
PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

CREATE TABLE IF NOT EXISTS taxonomy_search_aliases (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    kind ENUM('category', 'tag') NOT NULL,
    entity_id BIGINT UNSIGNED NOT NULL,
    alias VARCHAR(191) NOT NULL,
    UNIQUE KEY uq_taxonomy_alias (kind, alias),
    KEY idx_taxonomy_alias_entity (kind, entity_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
