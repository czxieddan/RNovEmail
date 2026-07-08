use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    healthz_doc,
    readyz_doc,
    create_user_doc,
    create_domain_doc,
    create_provider_doc,
    create_mailbox_doc,
    send_mail_doc,
    send_portal_mail_doc,
    get_outbound_doc,
    get_inbound_doc,
    ingest_webhook_doc
))]
pub struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

#[allow(dead_code)]
#[utoipa::path(get, path = "/healthz", responses((status = 200, description = "healthy")))]
fn healthz_doc() {}

#[allow(dead_code)]
#[utoipa::path(get, path = "/readyz", responses((status = 200, description = "ready")))]
fn readyz_doc() {}

#[allow(dead_code)]
#[utoipa::path(post, path = "/api/v1/admin/users", responses((status = 200, description = "user assigned")))]
fn create_user_doc() {}

#[allow(dead_code)]
#[utoipa::path(post, path = "/api/v1/admin/domains", responses((status = 200, description = "domain assigned")))]
fn create_domain_doc() {}

#[allow(dead_code)]
#[utoipa::path(post, path = "/api/v1/admin/provider-accounts", responses((status = 200, description = "provider account assigned")))]
fn create_provider_doc() {}

#[allow(dead_code)]
#[utoipa::path(post, path = "/api/v1/admin/mailboxes", responses((status = 200, description = "mailbox assigned")))]
fn create_mailbox_doc() {}

#[allow(dead_code)]
#[utoipa::path(post, path = "/api/v1/mail/send", responses((status = 200, description = "send queued")))]
fn send_mail_doc() {}

#[allow(dead_code)]
#[utoipa::path(post, path = "/api/v1/portal/mail/send", responses((status = 200, description = "user mail sent")))]
fn send_portal_mail_doc() {}

#[allow(dead_code)]
#[utoipa::path(get, path = "/api/v1/mail/outbound/{id}", responses((status = 200, description = "outbound message")))]
fn get_outbound_doc() {}

#[allow(dead_code)]
#[utoipa::path(get, path = "/api/v1/mail/inbound/{id}", responses((status = 200, description = "inbound message")))]
fn get_inbound_doc() {}

#[allow(dead_code)]
#[utoipa::path(post, path = "/api/v1/webhooks/{provider}/{account_id}", responses((status = 200, description = "webhook accepted")))]
fn ingest_webhook_doc() {}
