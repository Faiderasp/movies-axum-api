use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgPool};

use crate::domain::entities::analytics::{
    BucketStats, CatalogueOverview, GenreStats, MovieFinancials, RankedEntity, RatedMovie,
    YearStats,
};
use crate::domain::errors::MovieError;
use crate::domain::repositories::analytics_repository::{
    AnalyticsRepository, FinancialFilter, FinancialOrder, RankingFilter, RankingOrder, RatingFilter,
};
use crate::infrastructure::postgres::repositories::postgres_movie_repository::{
    genre_to_str, parse_genre,
};

/// Restricts a query aliasing `movies` as `m` to a genre bound as text; NULL disables it.
const GENRE_CLAUSE: &str = "EXISTS (SELECT 1 FROM movie_genres mg \
    JOIN genres g ON g.id = mg.genre_id WHERE mg.movie_id = m.id AND g.name = {param})";

const ROI_EXPR: &str = "(m.revenue - m.budget)::float8 / NULLIF(m.budget, 0)";

const BUDGET_TIERS: &str = "('Micro (< $1M)', 0, 1000000, 1), \
    ('Low ($1M - $10M)', 1000000, 10000000, 2), \
    ('Mid ($10M - $50M)', 10000000, 50000000, 3), \
    ('High ($50M - $100M)', 50000000, 100000000, 4), \
    ('Blockbuster (>= $100M)', 100000000, NULL, 5)";

const RUNTIME_TIERS: &str = "('Short (< 90 min)', 0, 90, 1), \
    ('Standard (90 - 120 min)', 90, 120, 2), \
    ('Long (120 - 150 min)', 120, 150, 3), \
    ('Very long (150 - 180 min)', 150, 180, 4), \
    ('Epic (>= 180 min)', 180, NULL, 5)";

pub struct PostgresAnalyticsRepository {
    pool: PgPool,
}

impl PostgresAnalyticsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn ranking<R, E>(
        &self,
        credits: &str,
        entities: &str,
        filter: &RankingFilter,
    ) -> Result<Vec<E>, MovieError>
    where
        R: for<'r> FromRow<'r, PgRow> + Send + Unpin + Into<E>,
    {
        let order = match filter.order {
            RankingOrder::MovieCount => "movie_count",
            RankingOrder::Rating => "average_rating",
            RankingOrder::Revenue => "total_revenue",
        };
        let sql = format!(
            r#"SELECT e.id, e.name,
                      COUNT(*) AS movie_count,
                      AVG(m.vote_average) FILTER (WHERE m.vote_count > 0) AS average_rating,
                      COALESCE(SUM(m.budget), 0)::bigint AS total_budget,
                      COALESCE(SUM(m.revenue), 0)::bigint AS total_revenue,
                      AVG(m.revenue::float8) FILTER (WHERE m.revenue > 0) AS average_revenue
               FROM ({credits}) c
               JOIN ({entities}) e ON e.id = c.entity_id
               JOIN movies m ON m.id = c.movie_id
               GROUP BY e.id, e.name
               HAVING COUNT(*) >= $1
               ORDER BY {order} DESC NULLS LAST, e.name
               LIMIT $2"#
        );

        let rows = sqlx::query_as::<_, R>(sqlx::AssertSqlSafe(sql))
            .bind(filter.min_movies as i64)
            .bind(filter.limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(db_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn buckets(&self, column: &str, tiers: &str) -> Result<Vec<BucketStats>, MovieError> {
        let sql = format!(
            r#"WITH tiers(label, lower_bound, upper_bound, position) AS (VALUES {tiers})
               SELECT t.label,
                      t.lower_bound::float8 AS lower_bound,
                      t.upper_bound::float8 AS upper_bound,
                      COUNT(m.id) AS movie_count,
                      AVG(m.vote_average) FILTER (WHERE m.vote_count > 0) AS average_rating,
                      AVG(m.budget::float8) FILTER (WHERE m.budget > 0) AS average_budget,
                      AVG(m.revenue::float8) FILTER (WHERE m.revenue > 0) AS average_revenue,
                      percentile_cont(0.5) WITHIN GROUP (ORDER BY {ROI_EXPR})
                          FILTER (WHERE m.budget > 0 AND m.revenue > 0) AS median_roi,
                      AVG((m.revenue > m.budget)::int::float8)
                          FILTER (WHERE m.budget > 0 AND m.revenue > 0) AS profitable_share
               FROM tiers t
               LEFT JOIN movies m
                   ON m.{column} > 0
                  AND m.{column} >= t.lower_bound
                  AND (t.upper_bound IS NULL OR m.{column} < t.upper_bound)
               GROUP BY t.label, t.lower_bound, t.upper_bound, t.position
               ORDER BY t.position"#
        );

        let rows = sqlx::query_as::<_, BucketRow>(sqlx::AssertSqlSafe(sql))
            .fetch_all(&self.pool)
            .await
            .map_err(db_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[async_trait]
impl AnalyticsRepository for PostgresAnalyticsRepository {
    async fn overview(&self) -> Result<CatalogueOverview, MovieError> {
        let sql = format!(
            r#"SELECT COUNT(*) AS total_movies,
                      COUNT(*) FILTER (WHERE m.status = 'Released') AS released_movies,
                      COALESCE(SUM(m.budget), 0)::bigint AS total_budget,
                      COALESCE(SUM(m.revenue), 0)::bigint AS total_revenue,
                      COALESCE(SUM(m.vote_count), 0)::bigint AS total_votes,
                      AVG(m.budget::float8) FILTER (WHERE m.budget > 0) AS average_budget,
                      AVG(m.revenue::float8) FILTER (WHERE m.revenue > 0) AS average_revenue,
                      AVG(m.vote_average) FILTER (WHERE m.vote_count > 0) AS average_rating,
                      AVG(m.runtime) FILTER (WHERE m.runtime > 0) AS average_runtime,
                      percentile_cont(0.5) WITHIN GROUP (ORDER BY {ROI_EXPR})
                          FILTER (WHERE m.budget > 0 AND m.revenue > 0) AS median_roi,
                      AVG((m.revenue > m.budget)::int::float8)
                          FILTER (WHERE m.budget > 0 AND m.revenue > 0) AS profitable_share,
                      MIN(m.release_date) AS earliest_release,
                      MAX(m.release_date) AS latest_release,
                      (SELECT COUNT(*) FROM genres) AS genre_count,
                      COUNT(DISTINCT m.original_language) AS language_count,
                      (SELECT COUNT(DISTINCT country_iso) FROM movie_production_countries) AS country_count,
                      (SELECT COUNT(DISTINCT company_id) FROM movie_production_companies) AS company_count
               FROM movies m"#
        );

        sqlx::query_as::<_, OverviewRow>(sqlx::AssertSqlSafe(sql))
            .fetch_one(&self.pool)
            .await
            .map(Into::into)
            .map_err(db_error)
    }

    async fn genre_stats(&self) -> Result<Vec<GenreStats>, MovieError> {
        let sql = format!(
            r#"SELECT g.name,
                      COUNT(*) AS movie_count,
                      AVG(m.vote_average) FILTER (WHERE m.vote_count > 0) AS average_rating,
                      AVG(m.runtime) FILTER (WHERE m.runtime > 0) AS average_runtime,
                      AVG(m.budget::float8) FILTER (WHERE m.budget > 0) AS average_budget,
                      AVG(m.revenue::float8) FILTER (WHERE m.revenue > 0) AS average_revenue,
                      COALESCE(SUM(m.revenue), 0)::bigint AS total_revenue,
                      percentile_cont(0.5) WITHIN GROUP (ORDER BY {ROI_EXPR})
                          FILTER (WHERE m.budget > 0 AND m.revenue > 0) AS median_roi
               FROM movie_genres mg
               JOIN genres g ON g.id = mg.genre_id
               JOIN movies m ON m.id = mg.movie_id
               GROUP BY g.name
               ORDER BY movie_count DESC, g.name"#
        );

        let rows = sqlx::query_as::<_, GenreStatsRow>(sqlx::AssertSqlSafe(sql))
            .fetch_all(&self.pool)
            .await
            .map_err(db_error)?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                Some(GenreStats {
                    genre: parse_genre(&row.name)?,
                    movie_count: row.movie_count as u64,
                    average_rating: row.average_rating,
                    average_runtime: row.average_runtime,
                    average_budget: row.average_budget,
                    average_revenue: row.average_revenue,
                    total_revenue: row.total_revenue as u64,
                    median_roi: row.median_roi,
                })
            })
            .collect())
    }

    async fn yearly_stats(
        &self,
        from: Option<i32>,
        to: Option<i32>,
    ) -> Result<Vec<YearStats>, MovieError> {
        let rows = sqlx::query_as::<_, YearStatsRow>(
            r#"SELECT EXTRACT(YEAR FROM m.release_date)::int AS year,
                      COUNT(*) AS movie_count,
                      COALESCE(SUM(m.budget), 0)::bigint AS total_budget,
                      COALESCE(SUM(m.revenue), 0)::bigint AS total_revenue,
                      AVG(m.vote_average) FILTER (WHERE m.vote_count > 0) AS average_rating,
                      AVG(m.runtime) FILTER (WHERE m.runtime > 0) AS average_runtime
               FROM movies m
               WHERE m.release_date IS NOT NULL
                 AND ($1::int IS NULL OR EXTRACT(YEAR FROM m.release_date) >= $1)
                 AND ($2::int IS NULL OR EXTRACT(YEAR FROM m.release_date) <= $2)
               GROUP BY 1
               ORDER BY 1"#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn top_financials(
        &self,
        filter: &FinancialFilter,
    ) -> Result<Vec<MovieFinancials>, MovieError> {
        let order = match filter.order {
            FinancialOrder::Revenue => "m.revenue",
            FinancialOrder::Profit => "profit",
            FinancialOrder::Roi => "roi",
        };
        let genre_clause = GENRE_CLAUSE.replace("{param}", "$2");
        let sql = format!(
            r#"SELECT m.id, m.title, m.release_date,
                      COALESCE(m.budget, 0) AS budget,
                      COALESCE(m.revenue, 0) AS revenue,
                      COALESCE(m.revenue, 0) - COALESCE(m.budget, 0) AS profit,
                      {ROI_EXPR} AS roi
               FROM movies m
               WHERE m.revenue > 0
                 AND COALESCE(m.budget, 0) >= $1
                 AND ($2::text IS NULL OR {genre_clause})
               ORDER BY {order} DESC NULLS LAST, m.id
               LIMIT $3"#
        );

        let rows = sqlx::query_as::<_, FinancialsRow>(sqlx::AssertSqlSafe(sql))
            .bind(filter.min_budget as i64)
            .bind(filter.genre.map(genre_to_str))
            .bind(filter.limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(db_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn top_rated(&self, filter: &RatingFilter) -> Result<Vec<RatedMovie>, MovieError> {
        // IMDb-style Bayesian average: pulls movies with few votes towards the catalogue mean.
        let genre_clause = GENRE_CLAUSE.replace("{param}", "$2");
        let sql = format!(
            r#"WITH baseline AS (
                   SELECT AVG(vote_average) AS mean FROM movies WHERE vote_count > 0
               )
               SELECT m.id, m.title, m.release_date,
                      COALESCE(m.vote_average, 0) AS vote_average,
                      m.vote_count,
                      m.vote_count::float8 / (m.vote_count + $1) * COALESCE(m.vote_average, 0)
                          + $1::float8 / (m.vote_count + $1) * b.mean AS weighted_rating
               FROM movies m
               CROSS JOIN baseline b
               WHERE m.vote_count >= $1
                 AND ($2::text IS NULL OR {genre_clause})
               ORDER BY weighted_rating DESC NULLS LAST, m.id
               LIMIT $3"#
        );

        let rows = sqlx::query_as::<_, RatedRow>(sqlx::AssertSqlSafe(sql))
            .bind(filter.min_votes as i64)
            .bind(filter.genre.map(genre_to_str))
            .bind(filter.limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(db_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn top_directors(
        &self,
        filter: &RankingFilter,
    ) -> Result<Vec<RankedEntity<u32>>, MovieError> {
        self.ranking::<RankingRow<i64>, _>(
            "SELECT DISTINCT movie_id, person_id AS entity_id FROM movie_crew WHERE job = 'Director'",
            "SELECT id, name FROM people",
            filter,
        )
        .await
    }

    async fn top_actors(
        &self,
        filter: &RankingFilter,
    ) -> Result<Vec<RankedEntity<u32>>, MovieError> {
        self.ranking::<RankingRow<i64>, _>(
            "SELECT movie_id, person_id AS entity_id FROM movie_cast",
            "SELECT id, name FROM people",
            filter,
        )
        .await
    }

    async fn top_companies(
        &self,
        filter: &RankingFilter,
    ) -> Result<Vec<RankedEntity<u32>>, MovieError> {
        self.ranking::<RankingRow<i64>, _>(
            "SELECT movie_id, company_id AS entity_id FROM movie_production_companies",
            "SELECT id, name FROM production_companies",
            filter,
        )
        .await
    }

    async fn top_countries(
        &self,
        filter: &RankingFilter,
    ) -> Result<Vec<RankedEntity<String>>, MovieError> {
        self.ranking::<RankingRow<String>, _>(
            "SELECT movie_id, country_iso AS entity_id FROM movie_production_countries",
            "SELECT iso_3166_1 AS id, name FROM production_countries",
            filter,
        )
        .await
    }

    async fn budget_tiers(&self) -> Result<Vec<BucketStats>, MovieError> {
        self.buckets("budget", BUDGET_TIERS).await
    }

    async fn runtime_distribution(&self) -> Result<Vec<BucketStats>, MovieError> {
        self.buckets("runtime", RUNTIME_TIERS).await
    }
}

fn db_error(error: sqlx::Error) -> MovieError {
    MovieError::RepositoryError(error.to_string())
}

#[derive(FromRow)]
struct OverviewRow {
    total_movies: i64,
    released_movies: i64,
    total_budget: i64,
    total_revenue: i64,
    total_votes: i64,
    average_budget: Option<f64>,
    average_revenue: Option<f64>,
    average_rating: Option<f64>,
    average_runtime: Option<f64>,
    median_roi: Option<f64>,
    profitable_share: Option<f64>,
    earliest_release: Option<NaiveDate>,
    latest_release: Option<NaiveDate>,
    genre_count: i64,
    language_count: i64,
    country_count: i64,
    company_count: i64,
}

impl From<OverviewRow> for CatalogueOverview {
    fn from(row: OverviewRow) -> Self {
        CatalogueOverview {
            total_movies: row.total_movies as u64,
            released_movies: row.released_movies as u64,
            total_budget: row.total_budget as u64,
            total_revenue: row.total_revenue as u64,
            total_votes: row.total_votes as u64,
            average_budget: row.average_budget,
            average_revenue: row.average_revenue,
            average_rating: row.average_rating,
            average_runtime: row.average_runtime,
            median_roi: row.median_roi,
            profitable_share: row.profitable_share,
            earliest_release: row.earliest_release,
            latest_release: row.latest_release,
            genre_count: row.genre_count as u64,
            language_count: row.language_count as u64,
            country_count: row.country_count as u64,
            company_count: row.company_count as u64,
        }
    }
}

#[derive(FromRow)]
struct GenreStatsRow {
    name: String,
    movie_count: i64,
    average_rating: Option<f64>,
    average_runtime: Option<f64>,
    average_budget: Option<f64>,
    average_revenue: Option<f64>,
    total_revenue: i64,
    median_roi: Option<f64>,
}

#[derive(FromRow)]
struct YearStatsRow {
    year: i32,
    movie_count: i64,
    total_budget: i64,
    total_revenue: i64,
    average_rating: Option<f64>,
    average_runtime: Option<f64>,
}

impl From<YearStatsRow> for YearStats {
    fn from(row: YearStatsRow) -> Self {
        YearStats {
            year: row.year,
            movie_count: row.movie_count as u64,
            total_budget: row.total_budget as u64,
            total_revenue: row.total_revenue as u64,
            average_rating: row.average_rating,
            average_runtime: row.average_runtime,
        }
    }
}

#[derive(FromRow)]
struct FinancialsRow {
    id: i64,
    title: Option<String>,
    release_date: Option<NaiveDate>,
    budget: i64,
    revenue: i64,
    profit: i64,
    roi: Option<f64>,
}

impl From<FinancialsRow> for MovieFinancials {
    fn from(row: FinancialsRow) -> Self {
        MovieFinancials {
            id: row.id as u32,
            title: row.title.unwrap_or_default(),
            release_date: row.release_date,
            budget: row.budget as u64,
            revenue: row.revenue as u64,
            profit: row.profit,
            roi: row.roi,
        }
    }
}

#[derive(FromRow)]
struct RatedRow {
    id: i64,
    title: Option<String>,
    release_date: Option<NaiveDate>,
    vote_average: f64,
    vote_count: i64,
    weighted_rating: f64,
}

impl From<RatedRow> for RatedMovie {
    fn from(row: RatedRow) -> Self {
        RatedMovie {
            id: row.id as u32,
            title: row.title.unwrap_or_default(),
            release_date: row.release_date,
            vote_average: row.vote_average,
            vote_count: row.vote_count as u32,
            weighted_rating: row.weighted_rating,
        }
    }
}

#[derive(FromRow)]
struct RankingRow<K> {
    id: K,
    name: String,
    movie_count: i64,
    average_rating: Option<f64>,
    total_budget: i64,
    total_revenue: i64,
    average_revenue: Option<f64>,
}

impl From<RankingRow<i64>> for RankedEntity<u32> {
    fn from(row: RankingRow<i64>) -> Self {
        RankedEntity {
            id: row.id as u32,
            name: row.name,
            movie_count: row.movie_count as u64,
            average_rating: row.average_rating,
            total_budget: row.total_budget as u64,
            total_revenue: row.total_revenue as u64,
            average_revenue: row.average_revenue,
        }
    }
}

impl From<RankingRow<String>> for RankedEntity<String> {
    fn from(row: RankingRow<String>) -> Self {
        RankedEntity {
            id: row.id,
            name: row.name,
            movie_count: row.movie_count as u64,
            average_rating: row.average_rating,
            total_budget: row.total_budget as u64,
            total_revenue: row.total_revenue as u64,
            average_revenue: row.average_revenue,
        }
    }
}

#[derive(FromRow)]
struct BucketRow {
    label: String,
    lower_bound: f64,
    upper_bound: Option<f64>,
    movie_count: i64,
    average_rating: Option<f64>,
    average_budget: Option<f64>,
    average_revenue: Option<f64>,
    median_roi: Option<f64>,
    profitable_share: Option<f64>,
}

impl From<BucketRow> for BucketStats {
    fn from(row: BucketRow) -> Self {
        BucketStats {
            label: row.label,
            lower_bound: row.lower_bound,
            upper_bound: row.upper_bound,
            movie_count: row.movie_count as u64,
            average_rating: row.average_rating,
            average_budget: row.average_budget,
            average_revenue: row.average_revenue,
            median_roi: row.median_roi,
            profitable_share: row.profitable_share,
        }
    }
}
