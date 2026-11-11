use async_trait::async_trait;

use crate::domain::entities::movie::{Genre, Movie, MovieUpdate};
use crate::domain::errors::MovieError;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MovieFilter {
    pub genre: Option<Genre>,
    pub keyword: Option<String>,
    pub language: Option<String>,
    pub country: Option<String>,
    pub search: Option<String>,
}

#[async_trait]
pub trait MovieRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Movie>, MovieError>;
    async fn find_paginated(&self, limit: usize, offset: usize, filter: &MovieFilter)
    -> Vec<Movie>;
    async fn find_by_id(&self, id: u32) -> Result<Option<Movie>, MovieError>;
    async fn create(&self, movie: Movie) -> Result<Movie, MovieError>;
    async fn update(&self, id: u32, update: MovieUpdate) -> Result<Movie, MovieError>;
    async fn delete(&self, id: u32) -> Result<(), MovieError>;
}
