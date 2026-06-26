//! Integration-style checks that home grid and listing data can be represented from fixtures.

use sok::models::video::fixtures::{sample_home_grid, sample_video_detail};
use sok::models::{VideoDetail, VideoThumb};

#[test]
fn home_grid_fixture_models_typed_thumb_vector() {
    let videos: Vec<VideoThumb> = sample_home_grid();
    assert_eq!(videos.len(), 2);

    for thumb in &videos {
        assert!(!thumb.slug.is_empty());
        assert!(!thumb.title.is_empty());
        assert!(thumb.thumb_url.ends_with(".jpg"));
        assert!(thumb.preview_mp4.contains(".mp4"));
        assert!(thumb.duration_seconds > 0);
        assert!(thumb.page_path().starts_with("/video/"));
    }
}

#[test]
fn sample_listing_row_matches_mirror_style_labels() {
    let grid = sample_home_grid();
    let with_comments = grid
        .iter()
        .find(|v| v.comments > 0)
        .expect("fixture includes commented thumb");

    assert_eq!(with_comments.views_label(), "234K");
    assert_eq!(with_comments.likes_label(), "78%");
    assert_eq!(with_comments.comments, 12);
}

#[test]
fn video_detail_fixture_is_distinct_from_thumb_only() {
    let detail: VideoDetail = sample_video_detail();
    assert!(detail.schema_description().is_some());
    assert!(detail.uploaded_at.is_some());
    assert!(detail.stream_token.is_some());
    assert_eq!(detail.thumb.views, 117_275);
    assert_eq!(detail.schema_duration(), "PT11M43S");
    assert_eq!(
        detail.schema_date_published().as_deref(),
        Some("2026-03-09")
    );
}
