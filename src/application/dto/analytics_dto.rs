use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::application::dto::movie_dto::{GenreRequest, GenreResponse};
use crate::domain::entities::analytics::{
    BucketStats, CatalogueOverview, GenreStats, MovieFinancials, RankedEntity, RatedMovie,
    YearStats,
};
use crate::domain::repositories::analytics_repository::{
    FinancialFilter, FinancialOrder, RankingFilter, RankingOrder, RatingFilter,
};

const MAX_LIMIT: usize = 100;

fn default_limit() -> usize {
    10
}

fn clamp_limit(limit: usize) -> usize {
    limit.clamp(1, MAX_LIMIT)
}

fn default_min_profitable_budget() -> u64 {
    1_000_000
}

fn default_min_votes() -> u32 {
    1_000
}

fn default_min_movies() -> u32 {
    3
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct YearRangeQuery {
    #[serde(default)]
    #[param(example = 1990)]
    pub from: Option<i32>,
    #[serde(default)]
    #[param(example = 2016)]
    pub to: Option<i32>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct TopGrossingQuery {
    /// Number of results, between 1 and 100
    #[serde(default = "default_limit")]
    #[param(example = 10)]
    pub limit: usize,
    #[serde(default)]
    #[param(example = "Action")]
    pub genre: Option<GenreRequest>,
}

impl From<&TopGrossingQuery> for FinancialFilter {
    fn from(query: &TopGrossingQuery) -> Self {
        FinancialFilter {
            limit: clamp_limit(query.limit),
            genre: query.genre.map(Into::into),
            min_budget: 0,
            order: FinancialOrder::Revenue,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProfitSortRequest {
    #[default]
    Profit,
    Roi,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct MostProfitableQuery {
    /// Number of results, between 1 and 100
    #[serde(default = "default_limit")]
    #[param(example = 10)]
    pub limit: usize,
    #[serde(default)]
    #[param(example = "Horror")]
    pub genre: Option<GenreRequest>,
    /// Rank by absolute profit or by return on investment
    #[serde(default)]
    pub sort_by: ProfitSortRequest,
    /// Ignore movies with a smaller budget, which filters out unreliable ROI outliers
    #[serde(default = "default_min_profitable_budget")]
    #[param(example = 1000000)]
    pub min_budget: u64,
}

impl From<&MostProfitableQuery> for FinancialFilter {
    fn from(query: &MostProfitableQuery) -> Self {
        FinancialFilter {
            limit: clamp_limit(query.limit),
            genre: query.genre.map(Into::into),
            // Profit is meaningless without a known budget
            min_budget: query.min_budget.max(1),
            order: match query.sort_by {
                ProfitSortRequest::Profit => FinancialOrder::Profit,
                ProfitSortRequest::Roi => FinancialOrder::Roi,
            },
        }
    }
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct TopRatedQuery {
    /// Number of results, between 1 and 100
    #[serde(default = "default_limit")]
    #[param(example = 10)]
    pub limit: usize,
    #[serde(default)]
    #[param(example = "Drama")]
    pub genre: Option<GenreRequest>,
    /// Minimum votes to be listed, also used as the weight of the catalogue mean
    #[serde(default = "default_min_votes")]
    #[param(example = 1000)]
    pub min_votes: u32,
}

impl From<&TopRatedQuery> for RatingFilter {
    fn from(query: &TopRatedQuery) -> Self {
        RatingFilter {
            limit: clamp_limit(query.limit),
            genre: query.genre.map(Into::into),
            min_votes: query.min_votes.max(1),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RankingSortRequest {
    #[default]
    MovieCount,
    Rating,
    Revenue,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct RankingQuery {
    /// Number of results, between 1 and 100
    #[serde(default = "default_limit")]
    #[param(example = 10)]
    pub limit: usize,
    /// Minimum number of movies to be listed
    #[serde(default = "default_min_movies")]
    #[param(example = 3)]
    pub min_movies: u32,
    #[serde(default)]
    pub sort_by: RankingSortRequest,
}

impl From<&RankingQuery> for RankingFilter {
    fn from(query: &RankingQuery) -> Self {
        RankingFilter {
            limit: clamp_limit(query.limit),
            min_movies: query.min_movies.max(1),
            order: match query.sort_by {
                RankingSortRequest::MovieCount => RankingOrder::MovieCount,
                RankingSortRequest::Rating => RankingOrder::Rating,
                RankingSortRequest::Revenue => RankingOrder::Revenue,
            },
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OverviewResponse {
    pub total_movies: u64,
    pub released_movies: u64,
    pub total_budget: u64,
    pub total_revenue: u64,
    pub total_votes: u64,
    /// Averages ignore movies where the value is unknown (zero)
    pub average_budget: Option<f64>,
    pub average_revenue: Option<f64>,
    pub average_rating: Option<f64>,
    pub average_runtime: Option<f64>,
    /// Median of (revenue - budget) / budget over movies with known budget and revenue
    pub median_roi: Option<f64>,
    /// Share of movies with known budget and revenue that made a profit, from 0 to 1
    pub profitable_share: Option<f64>,
    pub earliest_release: Option<NaiveDate>,
    pub latest_release: Option<NaiveDate>,
    pub genre_count: u64,
    pub language_count: u64,
    pub country_count: u64,
    pub company_count: u64,
}

impl From<CatalogueOverview> for OverviewResponse {
    fn from(overview: CatalogueOverview) -> Self {
        OverviewResponse {
            total_movies: overview.total_movies,
            released_movies: overview.released_movies,
            total_budget: overview.total_budget,
            total_revenue: overview.total_revenue,
            total_votes: overview.total_votes,
            average_budget: overview.average_budget,
            average_revenue: overview.average_revenue,
            average_rating: overview.average_rating,
            average_runtime: overview.average_runtime,
            median_roi: overview.median_roi,
            profitable_share: overview.profitable_share,
            earliest_release: overview.earliest_release,
            latest_release: overview.latest_release,
            genre_count: overview.genre_count,
            language_count: overview.language_count,
            country_count: overview.country_count,
            company_count: overview.company_count,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GenreStatsResponse {
    pub genre: GenreResponse,
    pub movie_count: u64,
    pub average_rating: Option<f64>,
    pub average_runtime: Option<f64>,
    pub average_budget: Option<f64>,
    pub average_revenue: Option<f64>,
    pub total_revenue: u64,
    pub median_roi: Option<f64>,
}

impl From<GenreStats> for GenreStatsResponse {
    fn from(stats: GenreStats) -> Self {
        GenreStatsResponse {
            genre: stats.genre.into(),
            movie_count: stats.movie_count,
            average_rating: stats.average_rating,
            average_runtime: stats.average_runtime,
            average_budget: stats.average_budget,
            average_revenue: stats.average_revenue,
            total_revenue: stats.total_revenue,
            median_roi: stats.median_roi,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct YearStatsResponse {
    pub year: i32,
    pub movie_count: u64,
    pub total_budget: u64,
    pub total_revenue: u64,
    pub average_rating: Option<f64>,
    pub average_runtime: Option<f64>,
}

impl From<YearStats> for YearStatsResponse {
    fn from(stats: YearStats) -> Self {
        YearStatsResponse {
            year: stats.year,
            movie_count: stats.movie_count,
            total_budget: stats.total_budget,
            total_revenue: stats.total_revenue,
            average_rating: stats.average_rating,
            average_runtime: stats.average_runtime,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MovieFinancialsResponse {
    pub id: u32,
    pub title: String,
    pub release_date: Option<NaiveDate>,
    pub budget: u64,
    pub revenue: u64,
    pub profit: i64,
    /// (revenue - budget) / budget, absent when the budget is unknown
    pub roi: Option<f64>,
}

impl From<MovieFinancials> for MovieFinancialsResponse {
    fn from(movie: MovieFinancials) -> Self {
        MovieFinancialsResponse {
            id: movie.id,
            title: movie.title,
            release_date: movie.release_date,
            budget: movie.budget,
            revenue: movie.revenue,
            profit: movie.profit,
            roi: movie.roi,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RatedMovieResponse {
    pub id: u32,
    pub title: String,
    pub release_date: Option<NaiveDate>,
    pub vote_average: f64,
    pub vote_count: u32,
    /// Bayesian average that weighs the rating by its number of votes
    pub weighted_rating: f64,
}

impl From<RatedMovie> for RatedMovieResponse {
    fn from(movie: RatedMovie) -> Self {
        RatedMovieResponse {
            id: movie.id,
            title: movie.title,
            release_date: movie.release_date,
            vote_average: movie.vote_average,
            vote_count: movie.vote_count,
            weighted_rating: movie.weighted_rating,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RankedEntityResponse {
    pub id: u32,
    pub name: String,
    pub movie_count: u64,
    pub average_rating: Option<f64>,
    pub total_budget: u64,
    pub total_revenue: u64,
    pub average_revenue: Option<f64>,
}

impl From<RankedEntity<u32>> for RankedEntityResponse {
    fn from(entity: RankedEntity<u32>) -> Self {
        RankedEntityResponse {
            id: entity.id,
            name: entity.name,
            movie_count: entity.movie_count,
            average_rating: entity.average_rating,
            total_budget: entity.total_budget,
            total_revenue: entity.total_revenue,
            average_revenue: entity.average_revenue,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CountryStatsResponse {
    pub iso_3166_1: String,
    pub name: String,
    pub movie_count: u64,
    pub average_rating: Option<f64>,
    pub total_budget: u64,
    pub total_revenue: u64,
    pub average_revenue: Option<f64>,
}

impl From<RankedEntity<String>> for CountryStatsResponse {
    fn from(entity: RankedEntity<String>) -> Self {
        CountryStatsResponse {
            iso_3166_1: entity.id,
            name: entity.name,
            movie_count: entity.movie_count,
            average_rating: entity.average_rating,
            total_budget: entity.total_budget,
            total_revenue: entity.total_revenue,
            average_revenue: entity.average_revenue,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BucketStatsResponse {
    pub label: String,
    /// Inclusive lower bound of the bucket
    pub lower_bound: f64,
    /// Exclusive upper bound of the bucket, absent for the last one
    pub upper_bound: Option<f64>,
    pub movie_count: u64,
    pub average_rating: Option<f64>,
    pub average_budget: Option<f64>,
    pub average_revenue: Option<f64>,
    pub median_roi: Option<f64>,
    pub profitable_share: Option<f64>,
}

impl From<BucketStats> for BucketStatsResponse {
    fn from(stats: BucketStats) -> Self {
        BucketStatsResponse {
            label: stats.label,
            lower_bound: stats.lower_bound,
            upper_bound: stats.upper_bound,
            movie_count: stats.movie_count,
            average_rating: stats.average_rating,
            average_budget: stats.average_budget,
            average_revenue: stats.average_revenue,
            median_roi: stats.median_roi,
            profitable_share: stats.profitable_share,
        }
    }
}
