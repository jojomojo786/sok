-- Catalog videos table (fallback when live MySQL is unavailable).
-- Aligns with PornsOK thumb cards (ImageObject) and video detail (VideoObject).

CREATE TABLE IF NOT EXISTS videos (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    slug VARCHAR(512) NOT NULL,
    title VARCHAR(512) NOT NULL,
    description TEXT NULL,
    duration_seconds INT UNSIGNED NOT NULL DEFAULT 0,
    thumb_url VARCHAR(1024) NOT NULL,
    preview_mp4 VARCHAR(1024) NOT NULL,
    stream_token VARCHAR(512) NULL,
    views BIGINT UNSIGNED NOT NULL DEFAULT 0,
    likes_up INT UNSIGNED NOT NULL DEFAULT 0,
    likes_down INT UNSIGNED NOT NULL DEFAULT 0,
    comment_count INT UNSIGNED NOT NULL DEFAULT 0,
    is_hd TINYINT(1) NOT NULL DEFAULT 0,
    wide_thumb TINYINT(1) NOT NULL DEFAULT 1,
    published_at DATE NULL,
    uploaded_at DATETIME NULL,
    status ENUM('published', 'hidden') NOT NULL DEFAULT 'published',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY uq_videos_slug (slug),
    KEY idx_videos_status_published (status, published_at),
    KEY idx_videos_views (views),
    KEY idx_videos_comments (comment_count),
    KEY idx_videos_hd (is_hd)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
