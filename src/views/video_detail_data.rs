//! Video detail page shell data (tags, related, comments) with DB + fixture fallback.

use crate::db::DbPool;
use crate::errors::AppError;
use crate::logging::log_ajax_db_fallback;
use crate::models::comment_store::{
    count_visible_comments_for_video, list_comments_for_video, render_comments_html_fragment,
    COMMENTS_INITIAL_LIMIT,
};
use crate::models::comments::Comment;
use crate::models::video::fixtures::{
    sample_dog_house_video_page, DogHouseFixtureExtras, DogHouseVideoPageFixture,
};
use crate::models::video::{
    fixtures::related_fixture_pool_for_slug, list_related_thumbs_for_video, video_detail_by_slug,
    VideoDetail, VideoThumb, RELATED_THUMB_BATCH_SIZE,
};
use crate::views::{RenderContext, SiteLayout, VideoPlayerPageContext};

#[derive(Debug, Clone)]
pub struct VideoTagLink {
    pub label: String,
    pub href: String,
}

#[derive(Debug, Clone)]
pub struct VideoCategoryLink {
    pub label: String,
    pub href: String,
}

#[derive(Debug, Clone)]
pub struct VideoDetailShell {
    pub player: VideoPlayerPageContext,
    pub rating_percent: u8,
    pub rating_count: u32,
    pub views_count: u64,
    pub views_label: String,
    pub comments_count: u32,
    pub schema_date_published: Option<String>,
    pub schema_upload_date: Option<String>,
    pub upload_date_label: String,
    pub upload_date_iso: Option<String>,
    pub channel_label: Option<String>,
    pub channel_href: Option<String>,
    pub categories: Vec<VideoCategoryLink>,
    pub tags: Vec<VideoTagLink>,
    pub related: Vec<VideoThumb>,
    pub related_ajax_batches: Vec<Vec<crate::models::video::RelatedAjaxThumb>>,
    pub comments: Vec<Comment>,
    pub comments_initial_html: String,
    pub show_more_comments_button: bool,
    pub download_form_action: String,
}

impl VideoDetailShell {
    pub fn download_href(&self) -> String {
        self.player.media.download_href()
    }

    pub fn download_enabled(&self) -> bool {
        self.player.media.download_enabled()
    }

    pub fn show_channel(&self) -> bool {
        self.channel_label.is_some()
    }

    pub fn channel_label_text(&self) -> &str {
        self.channel_label.as_deref().unwrap_or("")
    }

    pub fn channel_href_text(&self) -> &str {
        self.channel_href.as_deref().unwrap_or("")
    }

    pub fn channel_has_link(&self) -> bool {
        self.channel_href.as_deref().is_some_and(|h| !h.is_empty())
    }

    pub fn views_count_text(&self) -> String {
        self.views_count.to_string()
    }

    pub fn comments_count_text(&self) -> String {
        self.comments_count.to_string()
    }

    pub fn has_schema_date_published(&self) -> bool {
        self.schema_date_published.is_some()
    }

    pub fn schema_date_published_text(&self) -> &str {
        self.schema_date_published.as_deref().unwrap_or("")
    }

    pub fn has_schema_upload_date(&self) -> bool {
        self.schema_upload_date.is_some()
    }

    pub fn schema_upload_date_text(&self) -> &str {
        self.schema_upload_date.as_deref().unwrap_or("")
    }

    pub fn has_comment_interaction_stat(&self) -> bool {
        self.comments_count > 0
    }

    pub fn upload_date_iso_text(&self) -> &str {
        self.upload_date_iso.as_deref().unwrap_or("")
    }

    pub fn has_upload_date_iso(&self) -> bool {
        self.upload_date_iso.is_some()
    }

    pub fn related_array_json(&self) -> String {
        serde_json::to_string(&self.related_ajax_batches).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn has_comments_section(&self) -> bool {
        !self.comments.is_empty() || !self.comments_initial_html.is_empty()
    }

    pub fn show_more_comments(&self) -> bool {
        self.show_more_comments_button
    }
}

#[derive(Debug, Clone)]
struct CommentPack {
    comments: Vec<Comment>,
    comments_initial_html: String,
    comments_count: u32,
    show_more_comments_button: bool,
}

async fn load_comments_for_video(pool: &DbPool, video_id: u64, detail_count: u32) -> CommentPack {
    let listed = list_comments_for_video(pool, video_id, COMMENTS_INITIAL_LIMIT, 0)
        .await
        .unwrap_or_default();
    let total = count_visible_comments_for_video(pool, video_id)
        .await
        .unwrap_or(detail_count)
        .max(detail_count)
        .max(listed.len() as u32);
    let initial_html = render_comments_html_fragment(&listed);
    CommentPack {
        comments: listed,
        comments_initial_html: initial_html,
        comments_count: total,
        show_more_comments_button: total > COMMENTS_INITIAL_LIMIT,
    }
}

pub async fn load_video_detail_shell(
    pool: &DbPool,
    slug: &str,
    layout: SiteLayout,
) -> Result<VideoDetailShell, AppError> {
    match video_detail_by_slug(pool, slug).await {
        Ok(detail) => {
            Ok(build_shell_from_detail(pool, layout, detail, fixture_extras_for_slug(slug)).await)
        }
        Err(AppError::NotFound(_)) | Err(AppError::Db(_)) => {
            load_video_detail_fixture_fallback(slug, layout)
        }
        Err(e) => Err(e),
    }
}

fn load_video_detail_fixture_fallback(
    slug: &str,
    layout: SiteLayout,
) -> Result<VideoDetailShell, AppError> {
    let fixture = sample_dog_house_video_page();
    if fixture.detail.slug() == slug {
        Ok(build_shell_from_fixture(layout, fixture))
    } else {
        Err(AppError::NotFound(format!("video not found: {slug}")))
    }
}

async fn build_shell_from_detail(
    pool: &DbPool,
    layout: SiteLayout,
    detail: VideoDetail,
    extras: DogHouseFixtureExtras,
) -> VideoDetailShell {
    let slug = detail.slug().to_string();
    let (related, related_ajax_batches) = load_related_for_detail(pool, &detail, &slug).await;
    let comment_pack =
        load_comments_for_video(pool, detail.thumb.id, detail.comments_count()).await;
    let mut extras = extras;
    extras.related = related;
    extras.related_ajax_batches = related_ajax_batches;
    let player = RenderContext::video_detail_page(layout, &detail, extras.channel_label.as_deref());
    let download_form_action = player.media.download_form_action();
    shell_from_parts(player, extras, download_form_action, &detail, comment_pack)
}

fn build_shell_from_fixture(
    layout: SiteLayout,
    fixture: DogHouseVideoPageFixture,
) -> VideoDetailShell {
    let mut extras = fixture.extras;
    if extras.related_ajax_batches.is_empty() {
        let batches = crate::models::video::chunk_related_ajax_batches(&extras.related);
        extras.related_ajax_batches = batches;
    }
    let player =
        RenderContext::video_detail_page(layout, &fixture.detail, extras.channel_label.as_deref());
    let download_form_action = player.media.download_form_action();
    let comment_pack = CommentPack {
        comments: extras.comments.clone(),
        comments_initial_html: render_comments_html_fragment(&extras.comments),
        comments_count: fixture
            .detail
            .comments_count()
            .max(extras.comments.len() as u32),
        show_more_comments_button: false,
    };
    shell_from_parts(
        player,
        extras,
        download_form_action,
        &fixture.detail,
        comment_pack,
    )
}

fn shell_from_parts(
    player: VideoPlayerPageContext,
    extras: DogHouseFixtureExtras,
    download_form_action: String,
    detail: &VideoDetail,
    comment_pack: CommentPack,
) -> VideoDetailShell {
    VideoDetailShell {
        rating_percent: extras.rating_percent,
        rating_count: extras.rating_count,
        views_count: detail.views_count(),
        views_label: detail.views_label(),
        comments_count: comment_pack.comments_count,
        schema_date_published: detail.schema_date_published(),
        schema_upload_date: detail
            .schema_upload_date()
            .or_else(|| extras.upload_date_iso.clone()),
        upload_date_label: extras.upload_date_label,
        upload_date_iso: extras.upload_date_iso,
        channel_label: extras.channel_label,
        channel_href: extras.channel_href,
        categories: extras
            .categories
            .into_iter()
            .map(|c| VideoCategoryLink {
                label: c.label,
                href: c.href,
            })
            .collect(),
        tags: extras
            .tags
            .into_iter()
            .map(|t| VideoTagLink {
                label: t.label,
                href: t.href,
            })
            .collect(),
        related: extras.related.clone(),
        related_ajax_batches: extras.related_ajax_batches.clone(),
        comments: comment_pack.comments,
        comments_initial_html: comment_pack.comments_initial_html,
        show_more_comments_button: comment_pack.show_more_comments_button,
        download_form_action,
        player,
    }
}

async fn load_related_for_detail(
    pool: &DbPool,
    _detail: &VideoDetail,
    slug: &str,
) -> (
    Vec<VideoThumb>,
    Vec<Vec<crate::models::video::RelatedAjaxThumb>>,
) {
    let initial_limit = RELATED_THUMB_BATCH_SIZE as u32;
    match list_related_thumbs_for_video(pool, slug, initial_limit).await {
        Ok(related) if !related.is_empty() => {
            let batches = crate::models::video::chunk_related_ajax_batches(&related);
            return (related, batches);
        }
        Ok(_) => {}
        Err(AppError::Db(e)) => {
            log_ajax_db_fallback("related_videos", &e);
        }
        Err(_) => {}
    }

    let related = related_fixture_pool_for_slug(slug);
    let batches = crate::models::video::chunk_related_ajax_batches(&related);
    (related, batches)
}

fn fixture_extras_for_slug(slug: &str) -> DogHouseFixtureExtras {
    let fixture = sample_dog_house_video_page();
    if fixture.detail.slug() == slug {
        fixture.extras
    } else {
        DogHouseFixtureExtras::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::video::fixtures::{sample_dog_house_video_detail, DOG_HOUSE_SLUG};
    use crate::views::SiteLayout;
    use askama::Template;

    #[test]
    fn dog_house_fixture_differs_from_athena_sample() {
        let dog = sample_dog_house_video_detail();
        let athena = crate::models::video::fixtures::sample_video_detail();
        assert_ne!(dog.slug(), athena.slug());
        assert_eq!(dog.slug(), DOG_HOUSE_SLUG);
    }

    #[test]
    fn shell_from_fixture_exposes_schema_metadata() {
        let fixture = sample_dog_house_video_page();
        let shell = build_shell_from_fixture(SiteLayout::production(), fixture);
        assert!(shell
            .player
            .ctx
            .page
            .title
            .contains(" / porn video by Dog House Digital"));
        assert_eq!(shell.player.media.schema_duration, "PT9M59S");
        assert_eq!(shell.schema_date_published.as_deref(), Some("2026-03-02"));
        assert_eq!(shell.views_count, 15_077);
        assert_eq!(shell.comments_count, 2);
        assert!(shell
            .player
            .media
            .schema_content_url
            .contains("/videofile/"));
        assert!(shell.tags.iter().any(|t| t.href.starts_with("/pornstar/")));
        assert!(shell.tags.iter().any(|t| t.href.starts_with("/channel/")));
        assert!(shell.download_enabled());
        assert!(shell.download_href().contains("/videofile/"));
        assert!(shell.download_form_action.contains("/videofile/"));
    }

    #[test]
    fn shell_from_fixture_has_player_mount_id() {
        let fixture = sample_dog_house_video_page();
        let shell = build_shell_from_fixture(SiteLayout::production(), fixture);
        assert_eq!(shell.player.media.player_container_id, "player_container2");
        assert!(!shell.tags.is_empty());
        assert!(!shell.related.is_empty());
    }

    #[test]
    fn renders_download_markup_for_dog_house_fixture() {
        let fixture = sample_dog_house_video_page();
        let shell = build_shell_from_fixture(SiteLayout::production(), fixture);
        let html = crate::views::VideoTemplate { shell }
            .render()
            .expect("render video template");
        assert!(html.contains("class=\"video_download meta-item\""));
        assert!(html.contains(">Download</a>"));
        assert!(html.contains("/videofile/WyJwb3JuaHViIiwicGg2MzVhNDIwNzAyNmE1IiwwXQ%3D%3D"));
        assert!(html.contains("id=\"download_form\""));
    }

    #[test]
    fn renders_disabled_download_markup_when_media_unavailable() {
        let fixture = sample_dog_house_video_page();
        let mut shell = build_shell_from_fixture(SiteLayout::production(), fixture);
        shell.player.media.stream_token = None;
        shell.player.media.preview_mp4_url.clear();
        shell.player.media.download_url = None;
        shell.player.media.stream_mode = crate::views::player_media::PlayerStreamMode::Unavailable;
        shell.player.media.stream_src.clear();
        shell.player.media.videofile_path = None;
        shell.download_form_action.clear();

        assert!(!shell.download_enabled());
        assert_eq!(shell.download_href(), "#");

        let html = crate::views::VideoTemplate { shell }
            .render()
            .expect("render video template");
        assert!(html.contains("class=\"video_download meta-item is-disabled\""));
        assert!(html.contains("title=\"Download unavailable\""));
        assert!(!html.contains("id=\"download_form\""));
        assert!(!html.contains(">Download</a>"));
    }
}
