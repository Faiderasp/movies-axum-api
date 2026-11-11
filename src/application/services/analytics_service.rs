use std::sync::Arc;

use crate::application::dto::analytics_dto::{
    BucketStatsResponse, CountryStatsResponse, GenreStatsResponse, MostProfitableQuery,
    MovieFinancialsResponse, OverviewResponse, RankedEntityResponse, RankingQuery,
    RatedMovieResponse, TopGrossingQuery, TopRatedQuery, YearRangeQuery, YearStatsResponse,
};
use crate::domain::errors::MovieError;
use crate::domain::repositories::analytics_repository::AnalyticsRepository;

#[derive(Clone)]
pub struct AnalyticsService {
    repository: Arc<dyn AnalyticsRepository>,
}

fn into_responses<T, R: From<T>>(items: Vec<T>) -> Vec<R> {
    items.into_iter().map(Into::into).collect()
}

impl AnalyticsService {
    pub fn new(repository: Arc<dyn AnalyticsRepository>) -> Self {
        Self { repository }
    }

    pub async fn overview(&self) -> Result<OverviewResponse, MovieError> {
        self.repository.overview().await.map(Into::into)
    }

    pub async fn genres(&self) -> Result<Vec<GenreStatsResponse>, MovieError> {
        self.repository.genre_stats().await.map(into_responses)
    }

    pub async fn years(&self, query: YearRangeQuery) -> Result<Vec<YearStatsResponse>, MovieError> {
        self.repository
            .yearly_stats(query.from, query.to)
            .await
            .map(into_responses)
    }

    pub async fn top_grossing(
        &self,
        query: TopGrossingQuery,
    ) -> Result<Vec<MovieFinancialsResponse>, MovieError> {
        self.repository
            .top_financials(&(&query).into())
            .await
            .map(into_responses)
    }

    pub async fn most_profitable(
        &self,
        query: MostProfitableQuery,
    ) -> Result<Vec<MovieFinancialsResponse>, MovieError> {
        self.repository
            .top_financials(&(&query).into())
            .await
            .map(into_responses)
    }

    pub async fn top_rated(
        &self,
        query: TopRatedQuery,
    ) -> Result<Vec<RatedMovieResponse>, MovieError> {
        self.repository
            .top_rated(&(&query).into())
            .await
            .map(into_responses)
    }

    pub async fn directors(
        &self,
        query: RankingQuery,
    ) -> Result<Vec<RankedEntityResponse>, MovieError> {
        self.repository
            .top_directors(&(&query).into())
            .await
            .map(into_responses)
    }

    pub async fn actors(
        &self,
        query: RankingQuery,
    ) -> Result<Vec<RankedEntityResponse>, MovieError> {
        self.repository
            .top_actors(&(&query).into())
            .await
            .map(into_responses)
    }

    pub async fn companies(
        &self,
        query: RankingQuery,
    ) -> Result<Vec<RankedEntityResponse>, MovieError> {
        self.repository
            .top_companies(&(&query).into())
            .await
            .map(into_responses)
    }

    pub async fn countries(
        &self,
        query: RankingQuery,
    ) -> Result<Vec<CountryStatsResponse>, MovieError> {
        self.repository
            .top_countries(&(&query).into())
            .await
            .map(into_responses)
    }

    pub async fn budget_tiers(&self) -> Result<Vec<BucketStatsResponse>, MovieError> {
        self.repository.budget_tiers().await.map(into_responses)
    }

    pub async fn runtime_distribution(&self) -> Result<Vec<BucketStatsResponse>, MovieError> {
        self.repository
            .runtime_distribution()
            .await
            .map(into_responses)
    }
}
