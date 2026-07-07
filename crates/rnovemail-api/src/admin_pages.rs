use axum::{Router, response::IntoResponse, routing::get};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin", get(dashboard))
        .route("/admin/login", get(login))
        .route("/admin/users", get(users))
        .route("/admin/domains", get(domains))
        .route("/admin/providers", get(providers))
        .route("/admin/mailboxes", get(mailboxes))
        .route("/admin/audit", get(audit))
}

async fn dashboard() -> impl IntoResponse {
    rnovemail_admin::dashboard_page()
}

async fn login() -> impl IntoResponse {
    rnovemail_admin::login_page()
}

async fn users() -> impl IntoResponse {
    rnovemail_admin::users_page()
}

async fn domains() -> impl IntoResponse {
    rnovemail_admin::domains_page()
}

async fn providers() -> impl IntoResponse {
    rnovemail_admin::providers_page()
}

async fn mailboxes() -> impl IntoResponse {
    rnovemail_admin::mailboxes_page()
}

async fn audit() -> impl IntoResponse {
    rnovemail_admin::audit_page()
}
