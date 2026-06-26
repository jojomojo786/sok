use crate::db::DbPool;
use crate::logging::log_best_effort_db_skip;

/// Best-effort analytics hook for bookmark / favourite UI hits.
///
/// The mirrored client fires `POST /ajax/add_hit/favourite` without reading the response.
/// Catalog schema has `metric_snapshots` for entity-scoped periods, but there is no
/// first-class favourite/bookmark entity to attach this hit to, so persistence is a no-op.
pub async fn record_favourite_hit_best_effort(pool: &DbPool) {
    if let Err(e) = try_record_favourite_hit(pool).await {
        log_best_effort_db_skip("add_hit_favourite", &e);
    }
}

async fn try_record_favourite_hit(_pool: &DbPool) -> Result<(), sqlx::Error> {
    Ok(())
}
