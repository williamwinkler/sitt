pub mod database;
pub mod project_repository;
pub mod time_track_repository;

pub enum DbErrors {
    NotFound,
    FailedConvertion(String),
    UnknownError,
}
