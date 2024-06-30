use rocket::{routes, Route};

use crate::handlers::project_handler::{create, get, get_all, delete};

pub fn routes() -> Vec<Route> {
    routes![create, get, get_all, delete]
}
