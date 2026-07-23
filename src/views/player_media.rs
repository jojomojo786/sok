//! Video detail player media URLs and inline JS boot globals (`isPLAYER`, `thumbs_path`, …).

use serde::Serialize;

use crate::config::media_url;
use crate::models::video::{preview_mp4_from_slug, thumb_url_from_slug, VideoDetail};
use crate::views::{AssetPaths, RenderContext, SiteLayout};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerStreamMode {
    VideofileToken,
    PreviewMp4Fallback,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlayerMediaView {
    pub video_id: u64,
    pub slug: String,
    pub title: String,
    pub duration_seconds: u32,
    pub duration_label: String,
    pub schema_duration: String,
    pub poster_url: String,
    pub preview_mp4_url: String,
    pub stream_src: String,
    pub stream_mode: PlayerStreamMode,
    pub videofile_path: Option<String>,
    pub stream_token: Option<String>,
    pub download_url: Option<String>,
    pub embed_path: String,
    pub embed_url: String,
    pub canonical_path: String,
    pub canonical_url: String,
    pub schema_content_url: String,
    pub videourl: String,
    pub player_container_id: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerBootGlobals {
    pub is_thumbs_or_player: bool,
    pub is_player: bool,
    pub lazy_threshold: u32,
    pub directory: String,
    pub thumbs_path: String,
    pub thumbs_dir: String,
    pub video_path: String,
    pub seb: bool,
    pub first_load: bool,
    pub pjs_v: u32,
    pub screen_mode: char,
    pub is_mobile: bool,
    pub videourl: String,
    pub id_video: u64,
    pub include_video_fields: bool,
    pub include_listing_preloads: bool,
}

impl PlayerBootGlobals {
    pub fn video_page(assets: &AssetPaths, thumbs_cdn_base: &str) -> Self {
        Self {
            is_thumbs_or_player: true,
            is_player: true,
            lazy_threshold: 2000,
            directory: assets.static_fox_tpl_directory(),
            thumbs_path: media_url(thumbs_cdn_base, &assets.thumbs_videos_dir),
            thumbs_dir: assets.thumbs_videos_dir.clone(),
            video_path: assets.video_path_segment.clone(),
            seb: false,
            first_load: false,
            pjs_v: 17,
            screen_mode: 'n',
            is_mobile: false,
            videourl: String::new(),
            id_video: 0,
            include_video_fields: true,
            include_listing_preloads: false,
        }
    }

    fn base_listing(
        is_thumbs_or_player: bool,
        directory: String,
        assets: &AssetPaths,
        thumbs_cdn_base: &str,
    ) -> Self {
        Self {
            is_thumbs_or_player,
            is_player: false,
            lazy_threshold: 2000,
            directory,
            thumbs_path: media_url(thumbs_cdn_base, &assets.thumbs_videos_dir),
            thumbs_dir: assets.thumbs_videos_dir.clone(),
            video_path: assets.video_path_segment.clone(),
            seb: false,
            first_load: false,
            pjs_v: 17,
            screen_mode: 'n',
            is_mobile: false,
            videourl: String::new(),
            id_video: 0,
            include_video_fields: false,
            include_listing_preloads: false,
        }
    }

    pub fn listing_page(assets: &AssetPaths, thumbs_cdn_base: &str) -> Self {
        let mut boot = Self::base_listing(
            false,
            media_url(thumbs_cdn_base, "fox-tpl"),
            assets,
            thumbs_cdn_base,
        );
        boot.first_load = true;
        boot.screen_mode = 'd';
        boot.include_listing_preloads = true;
        boot
    }

    pub fn home_listing_page(assets: &AssetPaths, thumbs_cdn_base: &str) -> Self {
        let mut boot = Self::base_listing(
            true,
            media_url(thumbs_cdn_base, "fox-tpl"),
            assets,
            thumbs_cdn_base,
        );
        boot.first_load = true;
        boot.screen_mode = 'd';
        boot.include_listing_preloads = true;
        boot
    }

    pub fn video_page_for_media(
        assets: &AssetPaths,
        thumbs_cdn_base: &str,
        media: &PlayerMediaView,
    ) -> Self {
        let mut boot = Self::video_page(assets, thumbs_cdn_base);
        boot.videourl = media.videourl.clone();
        boot.id_video = media.video_id;
        boot
    }

    pub fn to_inline_script(&self) -> String {
        let mut script = format!(
            " var isTHUMBS_OR_PLAYER = {}, isPLAYER = {}, lazyThreshold = {}, directory = \"{}\", thumbs_path = \"{}\", thumbs_dir = \"{}\", video_path = \"{}\", seb = {}, first_load = {}, pjs_v = {}, screen_mode = '{}', is_mobile = {}",
            js_bool(self.is_thumbs_or_player),
            js_bool(self.is_player),
            self.lazy_threshold,
            escape_js_string(&self.directory),
            escape_js_string(&self.thumbs_path),
            escape_js_string(&self.thumbs_dir),
            escape_js_string(&self.video_path),
            js_bool(self.seb),
            js_bool(self.first_load),
            self.pjs_v,
            self.screen_mode,
            js_bool(self.is_mobile),
        );
        if self.include_video_fields {
            script.push_str(&format!(
                ", videourl = \"{}\", id_video = {}",
                escape_js_string(&self.videourl),
                self.id_video,
            ));
        }
        script.push(';');
        if self.include_listing_preloads {
            script.push_str(" preloads.push({'array': ['https://c.foxporn.tv/fox-tpl/js/playerjs.js?v=17'], 'cookie': 'prel_pjs', 'cookie_days': 365, 'on_scroll': true, 'data_type': 'js'}); preloads.push({'array': ['https://c.foxporn.tv/fox-tpl/images/loadMoreVideos.gif'], 'cookie': 'prel_loadmore', 'cookie_days': 365, 'on_scroll': true, 'delay': 2});");
        }
        script.push(' ');
        script
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoPlayerPageContext {
    pub ctx: RenderContext,
    pub media: PlayerMediaView,
    pub boot: PlayerBootGlobals,
}

impl SiteLayout {
    pub fn thumbs_cdn_base(&self) -> &str {
        &self.media_cdn
    }

    pub fn thumbs_videos_url(&self) -> String {
        self.assets.thumbs_videos_url.clone()
    }
}

impl RenderContext {
    pub fn video_detail_page(
        layout: SiteLayout,
        detail: &VideoDetail,
        channel_label: Option<&str>,
    ) -> VideoPlayerPageContext {
        let site = layout.site_base_url.clone();
        let title = crate::models::video::video_page_title(detail.title(), channel_label);
        let description = detail
            .schema_description()
            .map(str::to_string)
            .unwrap_or_else(|| format!("Watch {} on PornsOK, the best porn site.", detail.title()));
        let canonical_path = detail.canonical_path();
        let page = crate::views::PageMeta {
            title: title.clone(),
            description: description.clone(),
            canonical_path: canonical_path.clone(),
            h1: detail.title().to_string(),
            rel_prev_href: None,
            rel_next_href: None,
            og_title: title,
            og_description: description,
            og_url: crate::models::pagination::absolute_url(&site, &canonical_path),
        };
        let ctx = Self::new(layout.clone(), page);
        let media = build_player_media(detail, &layout);
        let boot = PlayerBootGlobals::video_page_for_media(
            &layout.assets,
            layout.thumbs_cdn_base(),
            &media,
        );
        VideoPlayerPageContext { ctx, media, boot }
    }
}

impl PlayerMediaView {
    pub fn download_href(&self) -> String {
        if let Some(token) = self
            .stream_token
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
        {
            return videofile_route(token);
        }
        if let Some(url) = self.download_url.clone() {
            return url;
        }
        if self.stream_mode == PlayerStreamMode::PreviewMp4Fallback
            && !self.preview_mp4_url.trim().is_empty()
        {
            return self.preview_mp4_url.clone();
        }
        "#".to_string()
    }

    pub fn download_enabled(&self) -> bool {
        self.download_href() != "#"
    }

    pub fn download_form_action(&self) -> String {
        self.videofile_path
            .clone()
            .or_else(|| {
                self.stream_token
                    .as_deref()
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .map(videofile_route)
            })
            .filter(|path| !path.is_empty())
            .unwrap_or_else(|| self.stream_src.clone())
    }
}

pub fn build_player_media(detail: &VideoDetail, layout: &SiteLayout) -> PlayerMediaView {
    let thumbs_base = layout.thumbs_videos_url();
    let cdn_host = layout.thumbs_cdn_base();
    let slug = detail.slug().to_string();

    let poster_url = if detail.thumb.thumb_url.is_empty() {
        thumb_url_from_slug(&slug, &thumbs_base)
    } else {
        detail.thumb.thumb_url.clone()
    };

    let preview_mp4_url = if detail.thumb.preview_mp4.trim().is_empty() {
        preview_mp4_from_slug(&slug, &thumbs_base)
    } else {
        detail.thumb.preview_mp4.trim().to_string()
    };

    let (stream_src, stream_mode, videofile_path) =
        resolve_stream_src(detail.stream_token.as_deref(), &preview_mp4_url);

    let download_url = build_download_url(
        cdn_host,
        &layout.assets.video_path_segment,
        &slug,
        stream_mode,
    );

    let embed_path = detail.embed_path();
    let canonical_path = detail.canonical_path();
    let canonical_url =
        crate::models::pagination::absolute_url(&layout.site_base_url, &canonical_path);
    let embed_url = crate::models::pagination::absolute_url(&layout.site_base_url, &embed_path);
    let schema_content_url = detail.schema_content_url(&layout.site_base_url);

    PlayerMediaView {
        video_id: detail.thumb.id,
        slug: slug.clone(),
        title: detail.title().to_string(),
        duration_seconds: detail.thumb.duration_seconds,
        duration_label: detail.thumb.duration_label(),
        schema_duration: detail.schema_duration(),
        poster_url,
        preview_mp4_url,
        stream_src,
        stream_mode,
        videofile_path,
        stream_token: detail.stream_token.clone(),
        download_url,
        embed_path,
        embed_url,
        canonical_path,
        canonical_url,
        schema_content_url,
        videourl: slug,
        player_container_id: "player_container2",
    }
}

fn resolve_stream_src(
    stream_token: Option<&str>,
    preview_mp4_url: &str,
) -> (String, PlayerStreamMode, Option<String>) {
    if let Some(token) = stream_token.map(str::trim).filter(|t| !t.is_empty()) {
        let path = videofile_route(token);
        return (path.clone(), PlayerStreamMode::VideofileToken, Some(path));
    }
    if !preview_mp4_url.trim().is_empty() {
        return (
            preview_mp4_url.to_string(),
            PlayerStreamMode::PreviewMp4Fallback,
            None,
        );
    }
    (String::new(), PlayerStreamMode::Unavailable, None)
}

fn build_download_url(
    cdn_host: &str,
    video_path_segment: &str,
    slug: &str,
    stream_mode: PlayerStreamMode,
) -> Option<String> {
    let base = cdn_host.trim().trim_end_matches('/');
    let segment = video_path_segment.trim().trim_matches('/');
    if segment.is_empty() || stream_mode == PlayerStreamMode::VideofileToken {
        return None;
    }
    Some(format!("{base}/{segment}/{slug}"))
}

fn videofile_route(token: &str) -> String {
    format!("/videofile/{}", urlencoding::encode(token))
}

fn js_bool(v: bool) -> &'static str {
    if v {
        "true"
    } else {
        "false"
    }
}

fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::video::fixtures::sample_video_detail;

    fn layout() -> SiteLayout {
        SiteLayout::production()
    }

    #[test]
    fn video_boot_globals_set_is_player_true() {
        let boot = PlayerBootGlobals::video_page(&layout().assets, layout().thumbs_cdn_base());
        assert!(boot.is_thumbs_or_player);
        assert!(boot.is_player);
        assert_eq!(boot.directory, "/static/fox-tpl");
        let script = boot.to_inline_script();
        assert!(script.contains("isPLAYER = true"));
        assert!(script.contains("isTHUMBS_OR_PLAYER = true"));
    }

    #[test]
    fn configurable_cdn_updates_thumbs_path_in_boot() {
        let layout = SiteLayout::production().with_media_cdn("https://cdn.example.com/");
        let detail = sample_video_detail();
        let media = build_player_media(&detail, &layout);
        let boot = PlayerBootGlobals::video_page_for_media(
            &layout.assets,
            layout.thumbs_cdn_base(),
            &media,
        );
        assert_eq!(
            boot.thumbs_path,
            "https://cdn.example.com/fox-images/videos"
        );
    }

    #[test]
    fn fixture_media_uses_videofile_token_src() {
        let detail = sample_video_detail();
        let media = build_player_media(&detail, &layout());
        assert_eq!(media.player_container_id, "player_container2");
        assert_eq!(
            media.stream_src,
            "/videofile/WyJwb3JuaHViIiwicGg2Mjg0ZmViNzZlOWU2IiwwXQ%3D%3D"
        );
        assert_eq!(media.stream_mode, PlayerStreamMode::VideofileToken);
        assert!(media.download_url.is_none());
        assert_eq!(
            media.download_href(),
            "/videofile/WyJwb3JuaHViIiwicGg2Mjg0ZmViNzZlOWU2IiwwXQ%3D%3D"
        );
        assert!(media.download_enabled());
    }

    #[test]
    fn missing_token_falls_back_to_preview_mp4() {
        let mut detail = sample_video_detail();
        detail.stream_token = None;
        let media = build_player_media(&detail, &layout());
        assert_eq!(media.stream_mode, PlayerStreamMode::PreviewMp4Fallback);
        assert!(media.stream_src.contains(".mp4"));
        assert!(media.download_url.is_some());
        assert!(media.download_enabled());
        assert!(media.download_href().contains("/video/"));
    }

    #[test]
    fn unavailable_media_documents_disabled_download_state() {
        let (stream_src, stream_mode, _) = resolve_stream_src(None, "");
        assert_eq!(stream_mode, PlayerStreamMode::Unavailable);
        assert!(stream_src.is_empty());

        let detail = sample_video_detail();
        let mut media = build_player_media(&detail, &layout());
        media.stream_token = None;
        media.preview_mp4_url.clear();
        media.download_url = None;
        media.stream_mode = PlayerStreamMode::Unavailable;
        media.stream_src.clear();
        media.videofile_path = None;

        assert!(!media.download_enabled());
        assert_eq!(media.download_href(), "#");
        assert!(media.download_form_action().is_empty());
    }

    #[test]
    fn video_detail_page_context_wires_meta_and_player() {
        let detail = sample_video_detail();
        let page = RenderContext::video_detail_page(layout(), &detail, None);
        assert!(page.boot.is_player);
        assert_eq!(page.media.stream_mode, PlayerStreamMode::VideofileToken);
    }
}
