use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths())]
pub struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
