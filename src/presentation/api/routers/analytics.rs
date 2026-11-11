use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

use crate::application::dto::analytics_dto::{
    BucketStatsResponse, CountryStatsResponse, GenreStatsResponse, MostProfitableQuery,
    MovieFinancialsResponse, OverviewResponse, RankedEntityResponse, RankingQuery,
    RatedMovieResponse, TopGrossingQuery, TopRatedQuery, YearRangeQuery, YearStatsResponse,
};
use crate::presentation::api::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/overview", get(overview))
        .route("/genres", get(genres))
        .route("/years", get(years))
        .route("/movies/top-grossing", get(top_grossing))
        .route("/movies/most-profitable", get(most_profitable))
        .route("/movies/top-rated", get(top_rated))
        .route("/directors", get(directors))
        .route("/actors", get(actors))
        .route("/companies", get(companies))
        .route("/countries", get(countries))
        .route("/budget-tiers", get(budget_tiers))
        .route("/runtimes", get(runtimes))
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/overview",
    tag = "analytics",
    responses(
        (status = 200, description = "Catalogue-wide totals and averages", body = OverviewResponse)
    )
)]
pub(crate) async fn overview(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state.analytics_service.overview().await.map(Json)
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/genres",
    tag = "analytics",
    responses(
        (status = 200, description = "Statistics per genre, most common first", body = [GenreStatsResponse])
    )
)]
pub(crate) async fn genres(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state.analytics_service.genres().await.map(Json)
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/years",
    tag = "analytics",
    params(YearRangeQuery),
    responses(
        (status = 200, description = "Statistics per release year", body = [YearStatsResponse])
    )
)]
pub(crate) async fn years(
    State(state): State<AppState>,
    Query(query): Query<YearRangeQuery>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state.analytics_service.years(query).await.map(Json)
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/movies/top-grossing",
    tag = "analytics",
    params(TopGrossingQuery),
    responses(
        (status = 200, description = "Movies with the highest revenue", body = [MovieFinancialsResponse])
    )
)]
pub(crate) async fn top_grossing(
    State(state): State<AppState>,
    Query(query): Query<TopGrossingQuery>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state.analytics_service.top_grossing(query).await.map(Json)
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/movies/most-profitable",
    tag = "analytics",
    params(MostProfitableQuery),
    responses(
        (status = 200, description = "Movies with the highest profit or ROI", body = [MovieFinancialsResponse])
    )
)]
pub(crate) async fn most_profitable(
    State(state): State<AppState>,
    Query(query): Query<MostProfitableQuery>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state
        .analytics_service
        .most_profitable(query)
        .await
        .map(Json)
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/movies/top-rated",
    tag = "analytics",
    params(TopRatedQuery),
    responses(
        (status = 200, description = "Movies with the highest vote-weighted rating", body = [RatedMovieResponse])
    )
)]
pub(crate) async fn top_rated(
    State(state): State<AppState>,
    Query(query): Query<TopRatedQuery>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state.analytics_service.top_rated(query).await.map(Json)
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/directors",
    tag = "analytics",
    params(RankingQuery),
    responses(
        (status = 200, description = "Director rankings", body = [RankedEntityResponse])
    )
)]
pub(crate) async fn directors(
    State(state): State<AppState>,
    Query(query): Query<RankingQuery>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state.analytics_service.directors(query).await.map(Json)
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/actors",
    tag = "analytics",
    params(RankingQuery),
    responses(
        (status = 200, description = "Actor rankings", body = [RankedEntityResponse])
    )
)]
pub(crate) async fn actors(
    State(state): State<AppState>,
    Query(query): Query<RankingQuery>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state.analytics_service.actors(query).await.map(Json)
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/companies",
    tag = "analytics",
    params(RankingQuery),
    responses(
        (status = 200, description = "Production company rankings", body = [RankedEntityResponse])
    )
)]
pub(crate) async fn companies(
    State(state): State<AppState>,
    Query(query): Query<RankingQuery>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state.analytics_service.companies(query).await.map(Json)
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/countries",
    tag = "analytics",
    params(RankingQuery),
    responses(
        (status = 200, description = "Production country rankings", body = [CountryStatsResponse])
    )
)]
pub(crate) async fn countries(
    State(state): State<AppState>,
    Query(query): Query<RankingQuery>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state.analytics_service.countries(query).await.map(Json)
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/budget-tiers",
    tag = "analytics",
    responses(
        (status = 200, description = "Ratings and returns grouped by budget size", body = [BucketStatsResponse])
    )
)]
pub(crate) async fn budget_tiers(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state.analytics_service.budget_tiers().await.map(Json)
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/runtimes",
    tag = "analytics",
    responses(
        (status = 200, description = "Ratings and returns grouped by runtime in minutes", body = [BucketStatsResponse])
    )
)]
pub(crate) async fn runtimes(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state
        .analytics_service
        .runtime_distribution()
        .await
        .map(Json)
}
