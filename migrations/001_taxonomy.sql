-- Taxonomy tables for PornsOK replica (categories + tags, separate tables).
-- See docs/schema/taxonomy.md for design rationale.

CREATE TABLE IF NOT EXISTS categories (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    slug VARCHAR(191) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT NULL,
    thumb_url VARCHAR(512) NULL,
    video_count INT UNSIGNED NOT NULL DEFAULT 0,
    intro_html TEXT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    is_active TINYINT(1) NOT NULL DEFAULT 1,
    created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uq_categories_slug (slug),
    KEY idx_categories_active_sort (is_active, sort_order, display_name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS tags (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    slug VARCHAR(191) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT NULL,
    thumb_url VARCHAR(512) NULL,
    video_count INT UNSIGNED NOT NULL DEFAULT 0,
    weekly_views BIGINT UNSIGNED NOT NULL DEFAULT 0,
    is_active TINYINT(1) NOT NULL DEFAULT 1,
    created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uq_tags_slug (slug),
    KEY idx_tags_active_weekly (is_active, weekly_views DESC),
    KEY idx_tags_active_name (is_active, display_name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS taxonomy_search_aliases (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    kind ENUM('category', 'tag') NOT NULL,
    entity_id BIGINT UNSIGNED NOT NULL,
    alias VARCHAR(191) NOT NULL,
    UNIQUE KEY uq_taxonomy_alias (kind, alias),
    KEY idx_taxonomy_alias_entity (kind, entity_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS video_categories (
    video_id BIGINT UNSIGNED NOT NULL,
    category_id BIGINT UNSIGNED NOT NULL,
    PRIMARY KEY (video_id, category_id),
    KEY idx_video_categories_category (category_id, video_id),
    CONSTRAINT fk_video_categories_category FOREIGN KEY (category_id) REFERENCES categories (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS video_tags (
    video_id BIGINT UNSIGNED NOT NULL,
    tag_id BIGINT UNSIGNED NOT NULL,
    PRIMARY KEY (video_id, tag_id),
    KEY idx_video_tags_tag (tag_id, video_id),
    CONSTRAINT fk_video_tags_tag FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
