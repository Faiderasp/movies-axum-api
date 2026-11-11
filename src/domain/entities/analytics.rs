use chrono::NaiveDate;

use crate::domain::entities::movie::Genre;

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogueOverview {
    pub total_movies: u64,
    pub released_movies: u64,
    pub total_budget: u64,
    pub total_revenue: u64,
    pub total_votes: u64,
    pub average_budget: Option<f64>,
    pub average_revenue: Option<f64>,
    pub average_rating: Option<f64>,
    pub average_runtime: Option<f64>,
    pub median_roi: Option<f64>,
    pub profitable_share: Option<f64>,
    pub earliest_release: Option<NaiveDate>,
    pub latest_release: Option<NaiveDate>,
    pub genre_count: u64,
    pub language_count: u64,
    pub country_count: u64,
    pub company_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenreStats {
    pub genre: Genre,
    pub movie_count: u64,
    pub average_rating: Option<f64>,
    pub average_runtime: Option<f64>,
    pub average_budget: Option<f64>,
    pub average_revenue: Option<f64>,
    pub total_revenue: u64,
    pub median_roi: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YearStats {
    pub year: i32,
    pub movie_count: u64,
    pub total_budget: u64,
    pub total_revenue: u64,
    pub average_rating: Option<f64>,
    pub average_runtime: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MovieFinancials {
    pub id: u32,
    pub title: String,
    pub release_date: Option<NaiveDate>,
    pub budget: u64,
    pub revenue: u64,
    pub profit: i64,
    pub roi: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RatedMovie {
    pub id: u32,
    pub title: String,
    pub release_date: Option<NaiveDate>,
    pub vote_average: f64,
    pub vote_count: u32,
    pub weighted_rating: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankedEntity<K> {
    pub id: K,
    pub name: String,
    pub movie_count: u64,
    pub average_rating: Option<f64>,
    pub total_budget: u64,
    pub total_revenue: u64,
    pub average_revenue: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BucketStats {
    pub label: String,
    pub lower_bound: f64,
    pub upper_bound: Option<f64>,
    pub movie_count: u64,
    pub average_rating: Option<f64>,
    pub average_budget: Option<f64>,
    pub average_revenue: Option<f64>,
    pub median_roi: Option<f64>,
    pub profitable_share: Option<f64>,
}
