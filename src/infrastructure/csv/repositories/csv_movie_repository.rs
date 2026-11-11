use std::collections::HashMap;

use async_trait::async_trait;
use csv::StringRecord;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::domain::entities::movie::{CrewMember, Gender, Genre, Movie, MovieStatus, MovieUpdate};
use crate::domain::errors::MovieError;
use crate::domain::repositories::movie_repository::{MovieFilter, MovieRepository};
use crate::infrastructure::csv::csv_parser;

pub struct CsvMovieRepository {
    movies: RwLock<Vec<Movie>>,
}

impl CsvMovieRepository {
    pub fn from_csv_file(path: &str) -> Self {
        let mut reader = csv::Reader::from_path(path)
            .unwrap_or_else(|error| panic!("failed to open csv database at {path}: {error}"));

        let headers = reader
            .headers()
            .unwrap_or_else(|error| panic!("failed to read csv headers: {error}"))
            .iter()
            .enumerate()
            .map(|(index, name)| (name.to_string(), index))
            .collect::<HashMap<_, _>>();

        let movies = reader
            .records()
            .map(|record| {
                let record =
                    record.unwrap_or_else(|error| panic!("failed to read csv row: {error}"));
                record_to_movie(&record, &headers)
            })
            .collect();

        Self {
            movies: RwLock::new(movies),
        }
    }
}

#[async_trait]
impl MovieRepository for CsvMovieRepository {
    async fn find_all(&self) -> Result<Vec<Movie>, MovieError> {
        Ok(self.movies.read().await.clone())
    }

    async fn find_paginated(
        &self,
        limit: usize,
        offset: usize,
        filter: &MovieFilter,
    ) -> Vec<Movie> {
        self.movies
            .read()
            .await
            .iter()
            .filter(|movie| matches_filter(movie, filter))
            .skip(offset)
            .take(limit)
            .cloned()
            .collect()
    }

    async fn find_by_id(&self, id: u32) -> Result<Option<Movie>, MovieError> {
        Ok(self
            .movies
            .read()
            .await
            .iter()
            .find(|movie| movie.id == id)
            .cloned())
    }

    async fn create(&self, movie: Movie) -> Result<Movie, MovieError> {
        let mut movies = self.movies.write().await;
        if movies.iter().any(|existing| existing.id == movie.id) {
            return Err(MovieError::AlreadyExists(movie.id));
        }
        movies.push(movie.clone());
        Ok(movie)
    }

    async fn update(&self, id: u32, update: MovieUpdate) -> Result<Movie, MovieError> {
        let mut movies = self.movies.write().await;
        let movie = movies
            .iter_mut()
            .find(|movie| movie.id == id)
            .ok_or(MovieError::NotFound(id))?;
        update.apply_to(movie);
        Ok(movie.clone())
    }

    async fn delete(&self, id: u32) -> Result<(), MovieError> {
        let mut movies = self.movies.write().await;
        let position = movies
            .iter()
            .position(|movie| movie.id == id)
            .ok_or(MovieError::NotFound(id))?;
        movies.remove(position);
        Ok(())
    }
}

fn matches_filter(movie: &Movie, filter: &MovieFilter) -> bool {
    if let Some(genre) = filter.genre
        && !movie.genres.contains(&genre)
    {
        return false;
    }

    if let Some(keyword) = &filter.keyword {
        let keyword = keyword.to_lowercase();
        if !movie
            .keywords
            .iter()
            .any(|value| value.to_lowercase().contains(&keyword))
        {
            return false;
        }
    }

    if let Some(language) = &filter.language
        && !movie.original_language.eq_ignore_ascii_case(language)
    {
        return false;
    }

    if let Some(country) = &filter.country {
        let country_lower = country.to_lowercase();
        if !movie.production_countries.iter().any(|value| {
            value.iso_3166_1.eq_ignore_ascii_case(country)
                || value.name.to_lowercase().contains(&country_lower)
        }) {
            return false;
        }
    }

    if let Some(search) = &filter.search {
        let search = search.to_lowercase();
        if !movie.title.to_lowercase().contains(&search)
            && !movie.original_title.to_lowercase().contains(&search)
        {
            return false;
        }
    }

    true
}

fn field<'a>(record: &'a StringRecord, headers: &HashMap<String, usize>, name: &str) -> &'a str {
    headers
        .get(name)
        .and_then(|&index| record.get(index))
        .unwrap_or_default()
        .trim()
}

fn record_to_movie(record: &StringRecord, headers: &HashMap<String, usize>) -> Movie {
    let non_empty = |value: &str| (!value.is_empty()).then(|| value.to_string());

    let runtime = field(record, headers, "runtime");
    let release_date = field(record, headers, "release_date");

    Movie {
        index: field(record, headers, "index").parse().unwrap_or_default(),
        budget: field(record, headers, "budget").parse().unwrap_or_default(),
        genres: parse_genres(field(record, headers, "genres")),
        homepage: non_empty(field(record, headers, "homepage")),
        id: field(record, headers, "id").parse().unwrap_or_default(),
        keywords: field(record, headers, "keywords")
            .split_whitespace()
            .map(String::from)
            .collect(),
        original_language: field(record, headers, "original_language").to_string(),
        original_title: field(record, headers, "original_title").to_string(),
        overview: non_empty(field(record, headers, "overview")),
        popularity: field(record, headers, "popularity")
            .parse()
            .unwrap_or_default(),
        production_companies: parse_json_field(field(record, headers, "production_companies")),
        production_countries: parse_json_field(field(record, headers, "production_countries")),
        release_date: (!release_date.is_empty())
            .then(|| release_date.parse().ok())
            .flatten(),
        revenue: field(record, headers, "revenue")
            .parse()
            .unwrap_or_default(),
        runtime: (!runtime.is_empty())
            .then(|| runtime.parse::<f64>().ok())
            .flatten()
            .map(|value| value as u32),
        spoken_languages: parse_json_field(field(record, headers, "spoken_languages")),
        status: parse_status(field(record, headers, "status")),
        tagline: non_empty(field(record, headers, "tagline")),
        title: field(record, headers, "title").to_string(),
        vote_average: field(record, headers, "vote_average")
            .parse()
            .unwrap_or_default(),
        vote_count: field(record, headers, "vote_count")
            .parse()
            .unwrap_or_default(),
        cast: field(record, headers, "cast")
            .split_whitespace()
            .map(String::from)
            .collect(),
        crew: parse_crew(field(record, headers, "crew")),
        director: non_empty(field(record, headers, "director")),
    }
}

fn parse_json_field<T: serde::de::DeserializeOwned + Default>(raw: &str) -> T {
    serde_json::from_str(raw).unwrap_or_default()
}

fn parse_genres(raw: &str) -> Vec<Genre> {
    raw.replace("Science Fiction", "ScienceFiction")
        .replace("TV Movie", "TvMovie")
        .split_whitespace()
        .filter_map(|token| match token {
            "Action" => Some(Genre::Action),
            "Adventure" => Some(Genre::Adventure),
            "Animation" => Some(Genre::Animation),
            "Comedy" => Some(Genre::Comedy),
            "Crime" => Some(Genre::Crime),
            "Documentary" => Some(Genre::Documentary),
            "Drama" => Some(Genre::Drama),
            "Family" => Some(Genre::Family),
            "Fantasy" => Some(Genre::Fantasy),
            "Foreign" => Some(Genre::Foreign),
            "History" => Some(Genre::History),
            "Horror" => Some(Genre::Horror),
            "Music" => Some(Genre::Music),
            "Mystery" => Some(Genre::Mystery),
            "Romance" => Some(Genre::Romance),
            "ScienceFiction" => Some(Genre::ScienceFiction),
            "Thriller" => Some(Genre::Thriller),
            "TvMovie" => Some(Genre::TvMovie),
            "War" => Some(Genre::War),
            "Western" => Some(Genre::Western),
            _ => None,
        })
        .collect()
}

fn parse_status(raw: &str) -> MovieStatus {
    match raw {
        "Released" => MovieStatus::Released,
        "Rumored" => MovieStatus::Rumored,
        _ => MovieStatus::PostProduction,
    }
}

fn parse_crew(raw: &str) -> Vec<CrewMember> {
    let Value::Array(items) = csv_parser::parse(raw) else {
        return Vec::new();
    };

    items
        .into_iter()
        .filter_map(|item| {
            let object = item.as_object()?;
            Some(CrewMember {
                id: object.get("id")?.as_u64()? as u32,
                name: object.get("name")?.as_str()?.to_string(),
                gender: parse_gender(object.get("gender")?.as_i64().unwrap_or_default()),
                department: object.get("department")?.as_str()?.to_string(),
                job: object.get("job")?.as_str()?.to_string(),
                credit_id: object.get("credit_id")?.as_str()?.to_string(),
            })
        })
        .collect()
}

fn parse_gender(value: i64) -> Gender {
    match value {
        1 => Gender::Female,
        2 => Gender::Male,
        _ => Gender::Unspecified,
    }
}
