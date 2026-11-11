use std::sync::Arc;

use crate::application::services::{
    analytics_service::AnalyticsService, movie_service::MovieService, user_service::UserService,
};

#[derive(Clone)]
pub struct AppState {
    pub analytics_service: Arc<AnalyticsService>,
    pub movie_service: Arc<MovieService>,
    pub user_service: Arc<UserService>,
}
