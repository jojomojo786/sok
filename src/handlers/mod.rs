mod ajax;
mod common;
mod entities;
mod health;
mod home;
mod legal;
mod listings;
mod replay;
mod search;
mod video;

pub use common::HANDLER_MARKER;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Fixed paths (must register before `/{slug}` fallback)
        .route("/", web::get().to(home::index))
        .route(
            "/_diag/source-replay/{label}",
            web::get().to(replay::source_replay),
        )
        .route(
            "/_diag/source-replay/page/{name}.html",
            web::get().to(replay::source_replay_page),
        )
        .route(
            "/_diag/source-replay/ajax/{name}",
            web::get().to(replay::source_replay_ajax),
        )
        .route("/categories", web::get().to(listings::categories))
        .route("/health", web::get().to(health::health_check))
        .route("/pornstars", web::get().to(listings::pornstars_list))
        .route(
            "/pornstars/{page}",
            web::get().to(listings::pornstars_list_page),
        )
        .route("/channels", web::get().to(listings::channels_list))
        .route(
            "/channels/{page}",
            web::get().to(listings::channels_list_page),
        )
        .route("/tags", web::get().to(listings::tags_hub))
        .route(
            "/ajax/search_cats_tags_queries",
            web::post().to(ajax::search_cats_tags_queries),
        )
        .route("/ajax/search_help", web::post().to(ajax::search_help))
        .route(
            "/ajax/search_{search_type}",
            web::post().to(ajax::search_entity_page),
        )
        .route("/ajax/more_videos_3", web::post().to(ajax::more_videos_3))
        .route("/ajax/comments", web::post().to(ajax::post_comments))
        .route(
            "/ajax/more_comments",
            web::post().to(ajax::post_more_comments),
        )
        .route(
            "/ajax/add_hit/more_videos",
            web::post().to(ajax::add_hit_more_videos),
        )
        .route(
            "/ajax/add_hit/favourite",
            web::post().to(ajax::add_hit_favourite),
        )
        .route("/ajax/add_vote_v3", web::post().to(ajax::add_vote_v3))
        .route(
            "/ajax/update_pornstars",
            web::post().to(ajax::update_pornstars),
        )
        .route(
            "/ajax/update_channels",
            web::post().to(ajax::update_channels),
        )
        .route("/ajax/update_tags", web::post().to(ajax::update_tags))
        .route(
            "/ajax/update_watching_now",
            web::post().to(ajax::update_watching_now),
        )
        .route(
            "/ajax/update_newest_videos",
            web::post().to(ajax::update_newest_videos),
        )
        .route("/ajax", web::get().to(ajax::ajax_reserved))
        .route("/ajax/{tail:.*}", web::get().to(ajax::ajax_reserved))
        .route("/channel/{slug}", web::get().to(entities::channel_profile))
        .route(
            r"/channel/{slug}/{page:\d+}",
            web::get().to(entities::channel_profile_page),
        )
        .route(
            "/pornstar/{slug}",
            web::get().to(entities::pornstar_profile),
        )
        .route(
            r"/pornstar/{slug}/{page:\d+}",
            web::get().to(entities::pornstar_profile_page),
        )
        .route("/search", web::get().to(search::search_redirect))
        .route("/videos/{query}", web::get().to(search::videos_search))
        .route(
            r"/videos/{query}/{page:\d+}",
            web::get().to(search::videos_search_page),
        )
        // Regex / parameterized routes before category slug catch-all
        .route("/videofile/{token}", web::get().to(video::videofile))
        .route("/embeded/{slug}.html", web::get().to(video::embeded_html))
        .route("/video/{slug}.html", web::get().to(video::video_html))
        .route("/page/{name}.html", web::get().to(legal::page_static))
        .route(r"/{page:\d+}", web::get().to(home::home_page_num))
        // Category / tag listing pagination must register before `/{slug}` and after
        // `/{page:\d+}` so numeric home pages (e.g. `/2`) never collide.
        .route(
            r"/{slug}/{page:\d+}",
            web::get().to(listings::category_slug_page),
        )
        // Category / tag listing fallback (single path segment, non-numeric slugs)
        .route("/{slug}", web::get().to(listings::category_slug));
}
