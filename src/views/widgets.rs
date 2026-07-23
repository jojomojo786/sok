//! Homepage AJAX widget HTML/JSON fragments (`/ajax/update_*`).

use crate::models::entities::{channel_profile_path, pornstar_profile_path, EntityIndexCard};
use crate::models::taxonomy::{CategoryRow, TagRow, CATEGORY_THUMB_CDN_PREFIX};
use crate::models::video::VideoThumb;
use serde::Serialize;

pub const HOME_WIDGET_PORNSTARS_LIMIT: u32 = 6;
pub const HOME_WIDGET_CHANNELS_LIMIT: u32 = 6;
pub const HOME_WIDGET_TAGS_LIMIT: u32 = 24;
pub const HOME_WIDGET_WATCHING_NOW_LIMIT: u32 = 84;
pub const HOME_WIDGET_NEWEST_DEFAULT_COUNT: u32 = 12;
pub const HOME_WIDGET_NEWEST_MAX_COUNT: u32 = 48;

const LAZY: &str =
    "data:image/gif;base64,R0lGODlhAQABAJAAAAAAAAAAACH5BAEUAAAALAAAAAABAAEAAAICRAEAOw==";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UpdateTagsResponse {
    pub html: String,
    pub preload_before: String,
    pub preload_after: String,
    pub preload_array: Vec<String>,
}

pub fn render_pornstars_widget(cards: &[EntityIndexCard], cdn_base: &str) -> String {
    let html = cards
        .iter()
        .map(|card| {
            render_entity_cat_thumb(
                &pornstar_profile_path(&card.slug),
                &card.thumb_url(cdn_base),
                &card.display_name,
                card.video_count,
                false,
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    format!(" {html} ")
}

pub fn render_channels_widget(cards: &[EntityIndexCard], cdn_base: &str) -> String {
    let html = cards
        .iter()
        .map(|card| {
            render_entity_cat_thumb(
                &channel_profile_path(&card.slug),
                &card.thumb_url(cdn_base),
                &card.display_name,
                card.video_count,
                false,
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    format!(" {html} ")
}

pub fn render_watching_now_widget(videos: &[VideoThumb]) -> String {
    videos
        .iter()
        .map(render_watching_now_video_thumb)
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn render_newest_videos_widget(videos: &[VideoThumb]) -> String {
    videos
        .iter()
        .map(render_newest_video_thumb)
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn build_update_tags_response(tags: &[TagRow]) -> UpdateTagsResponse {
    let preload_before = CATEGORY_THUMB_CDN_PREFIX.to_string();
    let preload_after = "-mini.jpg".to_string();
    let preload_array: Vec<String> = tags.iter().map(|t| t.slug.clone()).collect();
    let html = tags
        .iter()
        .map(render_home_tag_span)
        .collect::<Vec<_>>()
        .join("\n");
    UpdateTagsResponse {
        html,
        preload_before,
        preload_after,
        preload_array,
    }
}

pub fn tags_from_category_rows(rows: &[CategoryRow]) -> Vec<TagRow> {
    rows.iter().map(category_row_as_tag).collect()
}

fn category_row_as_tag(row: &CategoryRow) -> TagRow {
    TagRow {
        id: row.id,
        slug: row.slug.clone(),
        display_name: row.display_name.clone(),
        description: row.description.clone(),
        thumb_url: row.thumb_url.clone(),
        video_count: row.video_count,
        weekly_views: u64::from(row.video_count),
        is_active: row.is_active,
    }
}

fn render_entity_cat_thumb(
    href: &str,
    thumb_url: &str,
    display_name: &str,
    video_count: u32,
    lazy: bool,
) -> String {
    let name = html_escape(display_name);
    let img = if lazy {
        format!(
            r#"<img class="thumb-cover" src="{lazy}" data-original="{thumb}" alt="{name}" />"#,
            lazy = LAZY,
            thumb = html_escape(thumb_url),
            name = name
        )
    } else {
        format!(
            r#"<img class="thumb-cover" src="{thumb}" alt="{name}" />"#,
            thumb = html_escape(thumb_url),
            name = name
        )
    };
    format!(
        concat!(
            r#"<div class="thumb cat" itemscope itemtype="http://schema.org/ImageObject"> "#,
            r#"<a href="{href}" class="thumb-in" itemprop="url"> "#,
            r#"<div class="thumb-img"> {img} "#,
            "<span class=\"count-videos\"><svg fill=\"#fff\"><use xlink:href=\"#camera-svg\" /></svg>{count}</span> ",
            r#"</div> <div class="thumb-title" itemprop="name">{name}</div> </a> </div>"#
        ),
        href = html_escape(href),
        img = img,
        name = name,
        count = video_count,
    )
}

fn render_watching_now_video_thumb(v: &VideoThumb) -> String {
    let title = html_escape(&v.title);
    let wide_class = if v.wide_thumb { "" } else { " not-wide" };
    let date_meta = v
        .schema_date_published()
        .map(|d| format!(r#"<meta itemprop="datePublished" content="{d}">"#))
        .unwrap_or_default();
    format!(
        concat!(
            r#"<div class="thumb vid" itemscope="" itemtype="http://schema.org/ImageObject"> "#,
            r#"<a class="thumb-in" href="{url}" target="_blank" itemprop="url"> "#,
            r#"<div class="thumb-img{wide_class}"> <div class="video-preview"></div> "#,
            r#"<img class="thumb-cover" src="{thumb}" data-video="{preview}" alt="{title}" /> "#,
            r#"<div class="thumb-meta-top fx-row"> <span class="ttime" itemprop="duration" content="{schema_duration}">{duration}</span> </div> "#,
            r#"<div class="thumb-meta-bottom fx-row"> <span class="tview"><i class="fa fa-eye"></i>{views}</span> "#,
            "<span class=\"tlike\"><svg fill=\"#fff\"><use xlink:href=\"#thumb-up-svg\" /></svg><span>{likes}</span></span> </div> {date_meta} ",
            r#"<div itemprop="interactionStatistic" itemscope itemtype="http://schema.org/InteractionCounter"> "#,
            r#"<link itemprop="interactionType" href="http://schema.org/WatchAction"/> <meta itemprop="userInteractionCount" content="{views_raw}"> </div> "#,
            r#"</div> <div class="thumb-title" itemprop="name">{title}</div> </a> </div>"#
        ),
        url = html_escape(&v.page_path()),
        wide_class = wide_class,
        thumb = html_escape(&v.thumb_url),
        preview = html_escape(&v.preview_mp4),
        title = title,
        schema_duration = v.schema_duration(),
        duration = v.duration_label(),
        views = v.views_label(),
        likes = v.likes_label(),
        date_meta = date_meta,
        views_raw = v.views,
    )
}

fn render_newest_video_thumb(v: &VideoThumb) -> String {
    render_watching_now_video_thumb(v)
}

fn render_home_tag_span(tag: &TagRow) -> String {
    let label = html_escape(&tag.display_name);
    let href = format!("//{}", tag.slug);
    let mini = format!("{CATEGORY_THUMB_CDN_PREFIX}{}-mini.jpg", tag.slug);
    let count = tag.weekly_views.max(u64::from(tag.video_count));
    format!(
        concat!(
            r#"    <span><i class="fa fa-tag"></i> <a href="{href}" "#,
            r#"onMouseMove="ShowVisualBox(event, '{mini}', false, false, {count}, 150, '{label}')" "#,
            r#"onMouseOut="HideVisualBox()">{label}</a></span>"#
        ),
        href = html_escape(&href),
        mini = html_escape(&mini),
        count = count,
        label = label,
    )
}

fn html_escape(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::video::VideoThumb;

    fn sample_video() -> VideoThumb {
        VideoThumb {
            id: 9,
            slug: "sample-scene".into(),
            title: "Sample Scene".into(),
            duration_seconds: 247,
            thumb_url: "https://c.foxporn.tv/fox-images/videos/sample-scene.jpg".into(),
            preview_mp4: "https://c.foxporn.tv/fox-images/videos/m-sample-scene.mp4".into(),
            views: 29_223,
            likes_percent: 80,
            comments: 0,
            published_at: chrono::NaiveDate::from_ymd_opt(2025, 1, 15),
            is_hd: true,
            wide_thumb: true,
        }
    }

    #[test]
    fn watching_now_fragment_contains_vid_thumb_markers() {
        let html = render_watching_now_widget(&[sample_video()]);
        assert!(html.contains(r#"class="thumb vid""#));
        assert!(html.contains("data-video="));
        assert!(html.contains("fa-eye"));
        assert!(html.contains("thumb-up-svg"));
        assert!(html.contains("/video/sample-scene.html"));
    }

    #[test]
    fn update_tags_json_shape_matches_frontend_contract() {
        let tags = vec![TagRow {
            id: 1,
            slug: "busty".into(),
            display_name: "Busty".into(),
            description: None,
            thumb_url: None,
            video_count: 120,
            weekly_views: 5_000,
            is_active: true,
        }];
        let resp = build_update_tags_response(&tags);
        assert!(resp.html.contains("fa fa-tag"));
        assert!(resp.html.contains("//busty"));
        assert_eq!(resp.preload_before, CATEGORY_THUMB_CDN_PREFIX);
        assert_eq!(resp.preload_after, "-mini.jpg");
        assert_eq!(resp.preload_array, vec!["busty".to_string()]);
        let json = serde_json::to_string(&resp).expect("json");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v.get("html").and_then(|x| x.as_str()).is_some());
        assert!(v.get("preload_before").and_then(|x| x.as_str()).is_some());
        assert!(v.get("preload_after").and_then(|x| x.as_str()).is_some());
        assert!(v.get("preload_array").and_then(|x| x.as_array()).is_some());
    }
}
