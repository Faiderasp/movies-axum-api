use std::sync::Arc;

use axum::Router;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::application::services::analytics_service::AnalyticsService;
use crate::application::services::hash_service::Argon2HashService;
use crate::application::services::movie_service::MovieService;
use crate::application::services::user_service::UserService;
use crate::infrastructure::postgres::postgres;
use crate::infrastructure::postgres::repositories::postgres_analytics_repository::PostgresAnalyticsRepository;
use crate::infrastructure::postgres::repositories::postgres_movie_repository::PostgresMovieRepository;
use crate::infrastructure::postgres::repositories::postgres_user_repository::PostgresUserRepository;
use crate::presentation::api::openapi::ApiDoc;
use crate::presentation::api::routers::{analytics, health, movies, users};
use crate::presentation::api::state::AppState;

pub async fn run() -> Router {
    let pool = postgres::initialize_database().await;

    let movie_repository = PostgresMovieRepository::new(pool.clone());
    let analytics_repository = PostgresAnalyticsRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool);
    let hasher_service = Argon2HashService::new();
    let movie_service = Arc::new(MovieService::new(Arc::new(movie_repository)));
    let user_service = Arc::new(UserService::new(
        Arc::new(user_repository),
        Arc::new(hasher_service),
    ));
    let analytics_service = Arc::new(AnalyticsService::new(Arc::new(analytics_repository)));
    let state = AppState {
        analytics_service,
        movie_service,
        user_service,
    };

    let api_router: Router = Router::new()
        .nest("/analytics", analytics::router())
        .nest("/health", health::router())
        .nest("/movies", movies::router())
        .nest("/users", users::router())
        .layer(CorsLayer::permissive())
        .with_state(state);

    Router::new().nest("/api/v1", api_router).merge(
        SwaggerUi::new("/api/v1/docs").url("/api/v1/api-docs/openapi.json", ApiDoc::openapi()),
    )
}
