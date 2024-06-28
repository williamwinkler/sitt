use rocket::{Route, routes};

use crate::handlers::project_handler::{create, get_all};

pub fn routes() -> Vec<Route> {
    routes![create, get_all]
}
