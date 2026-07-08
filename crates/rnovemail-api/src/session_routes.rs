use axum::{
    Form, Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use rnovemail_admin::{Lang, LoginScopeView, PageContext, Theme};
use rnovemail_domain::EmailAddress;
use serde::Deserialize;

use crate::{AppState, middleware::ApiRejection, router::SESSION_COOKIE, session::SessionRole};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", get(login_page).post(create_session))
        .route("/admin/login", get(admin_login_redirect))
        .route("/logout", post(logout))
}

#[derive(Deserialize)]
struct LoginQuery {
    lang: Option<String>,
    next: Option<String>,
    scope: Option<String>,
    theme: Option<String>,
}

#[derive(Deserialize)]
struct LoginForm {
    identity: Option<String>,
    lang: Option<String>,
    next: Option<String>,
    scope: String,
    secret: String,
    theme: Option<String>,
}

async fn login_page(Query(query): Query<LoginQuery>) -> Response {
    let ctx = page_context(&query);
    let scope = login_scope(query.scope.as_deref());
    rnovemail_admin::login_page(&ctx, scope, false).into_response()
}

async fn admin_login_redirect() -> Redirect {
    Redirect::to("/login?scope=admin&next=%2Fadmin")
}

async fn create_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<LoginForm>,
) -> Response {
    let key = risk_key(&form, &headers);
    if let Err(error) = state.ensure_login_allowed(&key) {
        return error.into_response();
    }
    match authenticate(&state, &headers, &form) {
        Ok(session_id) => login_success(&state, &headers, &form, &key, &session_id),
        Err(error) => login_failure(&state, &form, key, error),
    }
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let _ = state.remove_session(&headers);
    (
        [(SET_COOKIE, expired_cookie())],
        Redirect::to("/login?scope=user&next=%2Fportal"),
    )
        .into_response()
}

fn authenticate(
    state: &AppState,
    headers: &HeaderMap,
    form: &LoginForm,
) -> Result<String, ApiRejection> {
    match form.scope.as_str() {
        "admin" => authenticate_admin(state, headers, form),
        "user" => authenticate_user(state, headers, form),
        _ => Err(ApiRejection::BadRequest),
    }
}

fn authenticate_admin(
    state: &AppState,
    headers: &HeaderMap,
    form: &LoginForm,
) -> Result<String, ApiRejection> {
    state.verify_admin_token(&form.secret)?;
    state.create_session(SessionRole::Admin, "admin".to_string(), headers)
}

fn authenticate_user(
    state: &AppState,
    headers: &HeaderMap,
    form: &LoginForm,
) -> Result<String, ApiRejection> {
    let identity = form.identity.as_deref().ok_or(ApiRejection::BadRequest)?;
    let email = EmailAddress::parse(identity).map_err(|_| ApiRejection::BadRequest)?;
    let user = state.verify_user_login(&email, &form.secret)?;
    state.create_session(
        SessionRole::User,
        user.primary_email().as_str().to_string(),
        headers,
    )
}

fn login_success(
    state: &AppState,
    headers: &HeaderMap,
    form: &LoginForm,
    key: &str,
    session_id: &str,
) -> Response {
    let _ = state.record_login_success(key);
    (
        [(SET_COOKIE, session_cookie(session_id, headers))],
        Redirect::to(&safe_next(form.next.as_deref(), &form.scope)),
    )
        .into_response()
}

fn login_failure(
    state: &AppState,
    form: &LoginForm,
    key: String,
    _error: ApiRejection,
) -> Response {
    let _ = state.record_login_failure(key);
    let ctx = PageContext {
        lang: Lang::parse(form.lang.as_deref()),
        theme: Theme::parse(form.theme.as_deref()),
        next: safe_next(form.next.as_deref(), &form.scope),
    };
    (
        StatusCode::UNAUTHORIZED,
        rnovemail_admin::login_page(&ctx, login_scope(Some(&form.scope)), true),
    )
        .into_response()
}

fn page_context(query: &LoginQuery) -> PageContext {
    PageContext {
        lang: Lang::parse(query.lang.as_deref()),
        theme: Theme::parse(query.theme.as_deref()),
        next: safe_next(
            query.next.as_deref(),
            query.scope.as_deref().unwrap_or("user"),
        ),
    }
}

fn login_scope(scope: Option<&str>) -> LoginScopeView {
    match scope {
        Some("admin") => LoginScopeView::Admin,
        _ => LoginScopeView::User,
    }
}

fn safe_next(next: Option<&str>, scope: &str) -> String {
    let fallback = match scope {
        "admin" => "/admin",
        _ => "/portal",
    };
    match next.filter(|value| value.starts_with('/') && !value.starts_with("//")) {
        Some(value) => value.to_string(),
        None => fallback.to_string(),
    }
}

fn session_cookie(session_id: &str, headers: &HeaderMap) -> String {
    let secure = secure_suffix(headers);
    format!("{SESSION_COOKIE}={session_id}; Path=/; HttpOnly; SameSite=Lax; Max-Age=28800{secure}")
}

fn expired_cookie() -> String {
    format!("{SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}

fn secure_suffix(headers: &HeaderMap) -> &'static str {
    match forwarded_proto(headers).eq_ignore_ascii_case("https") {
        true => "; Secure",
        false => "",
    }
}

fn forwarded_proto(headers: &HeaderMap) -> &str {
    headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
}

fn risk_key(form: &LoginForm, headers: &HeaderMap) -> String {
    let identity = form.identity.as_deref().unwrap_or("admin");
    let agent = headers
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    format!("{}:{identity}:{agent}", form.scope)
}
