mod categories_data;
mod context;
mod home;
mod legal_page;
mod player_media;
mod video_detail_data;

pub use categories_data::{
    categories_page_from_fixture_seed, load_categories_page_data, CategoriesPageData,
    CategoriesTopPornstar, CategoriesTopTag,
};

pub use context::{
    AssetPaths, PageMeta, RenderContext, SiteLayout, ThemeDefaults, DEFAULT_MEDIA_CDN,
};
pub use home::{HomeFilterView, HomePageView, HomePaginationItem};
pub use legal_page::{legal_page_view, legal_static_context, LegalPageView};
pub use video_detail_data::{load_video_detail_shell, VideoDetailShell};

pub use player_media::{
    build_player_media, PlayerBootGlobals, PlayerMediaView, PlayerStreamMode,
    VideoPlayerPageContext,
};

mod pornstars;
pub use pornstars::PornstarsIndexView;

mod widgets;
pub use widgets::{
    build_update_tags_response, render_channels_widget, render_newest_videos_widget,
    render_pornstars_widget, render_watching_now_widget, tags_from_category_rows,
    UpdateTagsResponse, HOME_WIDGET_CHANNELS_LIMIT, HOME_WIDGET_NEWEST_DEFAULT_COUNT,
    HOME_WIDGET_NEWEST_MAX_COUNT, HOME_WIDGET_PORNSTARS_LIMIT, HOME_WIDGET_TAGS_LIMIT,
    HOME_WIDGET_WATCHING_NOW_LIMIT,
};

mod channels;
pub use channels::ChannelsIndexView;

mod entity_profile;
mod search_listing;
mod slug_listing;
pub use entity_profile::EntityProfileView;
pub use search_listing::SearchListingView;
pub use slug_listing::{SlugListingView, TagsHubCard, TagsHubView};

use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub ctx: RenderContext,
    pub page: HomePageView,
}

#[derive(Template)]
#[template(path = "categories.html")]
pub struct CategoriesTemplate {
    pub ctx: RenderContext,
    pub categories: Vec<crate::models::taxonomy::CategoryCard>,
    pub top_tags: Vec<CategoriesTopTag>,
    pub tag_preload_slugs: Vec<String>,
    pub top_pornstars: Vec<CategoriesTopPornstar>,
}

#[derive(Template)]
#[template(path = "pornstars.html")]
pub struct PornstarsTemplate {
    pub ctx: RenderContext,
    pub pornstars: PornstarsIndexView,
}

#[derive(Template)]
#[template(path = "channels.html")]
pub struct ChannelsTemplate {
    pub ctx: RenderContext,
    pub channels: ChannelsIndexView,
}

#[derive(Template)]
#[template(path = "entity_profile.html")]
pub struct EntityProfileTemplate {
    pub ctx: RenderContext,
    pub page: EntityProfileView,
}

#[derive(Template)]
#[template(path = "search_listing.html")]
pub struct SearchListingTemplate {
    pub ctx: RenderContext,
    pub page: SearchListingView,
}

#[derive(Template)]
#[template(path = "slug_listing.html")]
pub struct SlugListingTemplate {
    pub ctx: RenderContext,
    pub page: SlugListingView,
}

#[derive(Template)]
#[template(path = "tags.html")]
pub struct TagsTemplate {
    pub ctx: RenderContext,
    pub tags: TagsHubView,
}

#[derive(Template)]
#[template(path = "video.html")]
pub struct VideoTemplate {
    pub shell: VideoDetailShell,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub ctx: RenderContext,
}

#[derive(Template)]
#[template(path = "legal_page.html")]
pub struct LegalPageTemplate {
    pub ctx: RenderContext,
    pub page: LegalPageView,
}
