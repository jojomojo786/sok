use actix_web::{http::header, web, HttpResponse, Responder};
use askama::Template;

use crate::config::Config;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::video::normalize_video_slug;

use super::common::HANDLER_MARKER;

pub async fn video_html(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let slug = normalize_video_slug(&path.into_inner());
    let layout = crate::views::SiteLayout::from_config(cfg.get_ref());
    let shell = crate::views::load_video_detail_shell(pool.get_ref(), &slug, layout).await?;
    let html = crate::views::VideoTemplate { shell }
        .render()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "video_html"))
        .content_type("text/html; charset=utf-8")
        .body(html))
}

/// `GET /videofile/{token}` — opaque stream token from live `<video src>`.
pub async fn videofile(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let token = path.into_inner();
    if token.trim().is_empty() {
        return Err(AppError::NotFound("videofile: empty token".into()));
    }

    if let Some(preview) = resolve_videofile_preview(pool.get_ref(), &token).await? {
        return Ok(HttpResponse::Found()
            .insert_header((HANDLER_MARKER, "videofile"))
            .insert_header((header::LOCATION, preview))
            .finish());
    }

    Err(AppError::NotFound("videofile: unknown token".into()))
}

/// `GET /embeded/{slug}.html` — production spelling preserved for Schema.org `embedUrl`.
pub async fn embeded_html(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let slug = normalize_video_slug(&path.into_inner());
    let meta = resolve_embed_meta(pool.get_ref(), &slug)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("embed: video not found: {slug}")))?;

    let stream_src = meta
        .stream_token
        .as_deref()
        .map(|t| format!("/videofile/{t}"))
        .or(meta.preview_mp4)
        .unwrap_or_else(|| "#".into());

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width,initial-scale=1" />
<title>{title} | PornsOK embed</title>
<style>html,body{{margin:0;height:100%;background:#000}}video{{width:100%;height:100%;object-fit:contain}}</style>
</head>
<body>
<video controls preload="metadata" poster="{thumb}" src="{stream_src}"></video>
</body>
</html>"#,
        title = html_escape(&meta.title),
        thumb = html_escape(&meta.thumb_url),
        stream_src = html_escape(&stream_src),
    );

    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "embeded_html"))
        .content_type("text/html; charset=utf-8")
        .body(html))
}

struct EmbedMeta {
    title: String,
    thumb_url: String,
    preview_mp4: Option<String>,
    stream_token: Option<String>,
}

async fn resolve_videofile_preview(pool: &DbPool, token: &str) -> Result<Option<String>, AppError> {
    if let Some(url) = query_migration_videofile(pool, token).await? {
        return Ok(Some(url));
    }
    query_dev_videofile(pool, token).await
}

async fn query_migration_videofile(pool: &DbPool, token: &str) -> Result<Option<String>, AppError> {
    let sql = r#"
        SELECT preview_mp4_url
        FROM videos
        WHERE is_active = 1 AND stream_url = ?
        LIMIT 1
    "#;
    let row = fetch_optional_resilient(
        sqlx::query_as::<_, (Option<String>,)>(sql).bind(token),
        pool,
    )
    .await?;
    Ok(row.and_then(|(url,)| url).filter(|u| !u.is_empty()))
}

async fn query_dev_videofile(pool: &DbPool, token: &str) -> Result<Option<String>, AppError> {
    let sql = r#"
        SELECT preview_mp4
        FROM videos
        WHERE status = 'published' AND stream_token = ?
        LIMIT 1
    "#;
    let row =
        fetch_optional_resilient(sqlx::query_as::<_, (String,)>(sql).bind(token), pool).await?;
    Ok(row.map(|(url,)| url).filter(|u| !u.is_empty()))
}

async fn resolve_embed_meta(pool: &DbPool, slug: &str) -> Result<Option<EmbedMeta>, AppError> {
    if let Some(meta) = query_migration_embed(pool, slug).await? {
        return Ok(Some(meta));
    }
    query_dev_embed(pool, slug).await
}

async fn query_migration_embed(pool: &DbPool, slug: &str) -> Result<Option<EmbedMeta>, AppError> {
    let sql = r#"
        SELECT title, COALESCE(thumb_url, ''), preview_mp4_url, stream_url
        FROM videos
        WHERE is_active = 1 AND slug = ?
        LIMIT 1
    "#;
    let row = fetch_optional_resilient(
        sqlx::query_as::<_, (String, String, Option<String>, Option<String>)>(sql).bind(slug),
        pool,
    )
    .await?;
    Ok(row.map(
        |(title, thumb_url, preview_mp4_url, stream_url)| EmbedMeta {
            title,
            thumb_url,
            preview_mp4: preview_mp4_url,
            stream_token: stream_url,
        },
    ))
}

async fn query_dev_embed(pool: &DbPool, slug: &str) -> Result<Option<EmbedMeta>, AppError> {
    let sql = r#"
        SELECT title, thumb_url, preview_mp4, stream_token
        FROM videos
        WHERE status = 'published' AND slug = ?
        LIMIT 1
    "#;
    let row = fetch_optional_resilient(
        sqlx::query_as::<_, (String, String, String, Option<String>)>(sql).bind(slug),
        pool,
    )
    .await?;
    Ok(
        row.map(|(title, thumb_url, preview_mp4, stream_token)| EmbedMeta {
            title,
            thumb_url,
            preview_mp4: Some(preview_mp4),
            stream_token,
        }),
    )
}

async fn fetch_optional_resilient<'q, T>(
    query: sqlx::query::QueryAs<'q, sqlx::MySql, T, sqlx::mysql::MySqlArguments>,
    pool: &DbPool,
) -> Result<Option<T>, AppError>
where
    T: for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> + Send + Unpin,
{
    match query.fetch_optional(pool).await {
        Ok(row) => Ok(row),
        Err(e) if is_optional_schema_mismatch(&e) => Ok(None),
        Err(e) => Err(AppError::Db(e)),
    }
}

fn is_optional_schema_mismatch(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db) => {
            matches!(db.code().as_deref(), Some("42S22") | Some("1054"))
        }
        _ => false,
    }
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
