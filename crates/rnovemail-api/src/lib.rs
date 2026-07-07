mod admin_routes;
mod mail_routes;
mod middleware;
mod openapi;
mod router;
mod webhook_routes;

pub use openapi::{ApiDoc, openapi};
pub use router::{AppState, build_router};
