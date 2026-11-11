use async_trait::async_trait;

use crate::domain::entities::analytics::{
    BucketStats, CatalogueOverview, GenreStats, MovieFinancials, RankedEntity, RatedMovie,
    YearStats,
};
use crate::domain::entities::movie::Genre;
use crate::domain::errors::MovieError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinancialOrder {
    Revenue,
    Profit,
    Roi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankingOrder {
    MovieCount,
    Rating,
    Revenue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FinancialFilter {
    pub limit: usize,
    pub genre: Option<Genre>,
    pub min_budget: u64,
    pub order: FinancialOrder,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RatingFilter {
    pub limit: usize,
    pub genre: Option<Genre>,
    pub min_votes: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankingFilter {
    pub limit: usize,
    pub min_movies: u32,
    pub order: RankingOrder,
}

#[async_trait]
pub trait AnalyticsRepository: Send + Sync {
    async fn overview(&self) -> Result<CatalogueOverview, MovieError>;
    async fn genre_stats(&self) -> Result<Vec<GenreStats>, MovieError>;
    async fn yearly_stats(
        &self,
        from: Option<i32>,
        to: Option<i32>,
    ) -> Result<Vec<YearStats>, MovieError>;
    async fn top_financials(
        &self,
        filter: &FinancialFilter,
    ) -> Result<Vec<MovieFinancials>, MovieError>;
    async fn top_rated(&self, filter: &RatingFilter) -> Result<Vec<RatedMovie>, MovieError>;
    async fn top_directors(
        &self,
        filter: &RankingFilter,
    ) -> Result<Vec<RankedEntity<u32>>, MovieError>;
    async fn top_actors(
        &self,
        filter: &RankingFilter,
    ) -> Result<Vec<RankedEntity<u32>>, MovieError>;
    async fn top_companies(
        &self,
        filter: &RankingFilter,
    ) -> Result<Vec<RankedEntity<u32>>, MovieError>;
    async fn top_countries(
        &self,
        filter: &RankingFilter,
    ) -> Result<Vec<RankedEntity<String>>, MovieError>;
    async fn budget_tiers(&self) -> Result<Vec<BucketStats>, MovieError>;
    async fn runtime_distribution(&self) -> Result<Vec<BucketStats>, MovieError>;
}
