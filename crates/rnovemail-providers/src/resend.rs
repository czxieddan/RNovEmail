use async_trait::async_trait;
use reqwest::Client;
use rnovemail_domain::{
    EmailAddress, InboundMessageAttachment, InboundMessageDetail, InboundMessageHeader,
    InboundMessageRaw, MessageId, ProviderType,
};
use rnovemail_webhook::SignatureVerifier;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use crate::{
    MailProvider, ProviderError, ProviderEvent, ProviderSendReceipt, ProviderWebhookRequest,
    SendMailRequest, VerifiedWebhook,
};

#[derive(Clone)]
pub struct ResendProvider {
    api_key: SecretString,
    client: Client,
    endpoint: String,
    webhook_verifier: Option<SignatureVerifier>,
}

impl ResendProvider {
    pub fn new(api_key: SecretString) -> Self {
        Self::with_endpoint(api_key, "https://api.resend.com")
    }

    pub fn with_endpoint(api_key: SecretString, endpoint: impl Into<String>) -> Self {
        Self {
            api_key,
            client: Client::new(),
            endpoint: endpoint.into(),
            webhook_verifier: None,
        }
    }

    pub fn with_webhook_secret(
        api_key: SecretString,
        webhook_secret: impl AsRef<str>,
    ) -> Result<Self, ProviderError> {
        let verifier = SignatureVerifier::from_svix_secret(webhook_secret)
            .map_err(|_| ProviderError::InvalidSignature)?;
        Ok(Self {
            api_key,
            client: Client::new(),
            endpoint: "https://api.resend.com".to_string(),
            webhook_verifier: Some(verifier),
        })
    }

    pub async fn retrieve_received_email(
        &self,
        email_id: &str,
    ) -> Result<InboundMessageDetail, ProviderError> {
        let response = self
            .client
            .get(received_email_endpoint(&self.endpoint, email_id))
            .bearer_auth(self.api_key.expose_secret())
            .send()
            .await
            .map_err(|_| ProviderError::ProviderRejected)?;
        ensure_success(response.status().is_success())?;
        let received = response
            .json::<ResendReceivedEmailResponse>()
            .await
            .map_err(|_| ProviderError::InvalidPayload)?;
        Ok(received.into_detail())
    }
}

#[async_trait]
impl MailProvider for ResendProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Resend
    }

    async fn send(&self, request: SendMailRequest) -> Result<ProviderSendReceipt, ProviderError> {
        let payload = ResendSendPayload::from_request(&request);
        let response = self
            .client
            .post(send_endpoint(&self.endpoint))
            .bearer_auth(self.api_key.expose_secret())
            .json(&payload)
            .send()
            .await
            .map_err(|_| ProviderError::ProviderRejected)?;
        ensure_success(response.status().is_success())?;
        let accepted = response
            .json::<ResendSendResponse>()
            .await
            .map_err(|_| ProviderError::InvalidPayload)?;
        Ok(ProviderSendReceipt {
            message_id: MessageId::new(),
            provider_message_id: accepted.id,
        })
    }

    fn verify_webhook(
        &self,
        request: ProviderWebhookRequest,
    ) -> Result<VerifiedWebhook, ProviderError> {
        ensure_webhook_headers(&request)?;
        let verifier = self
            .webhook_verifier
            .as_ref()
            .ok_or(ProviderError::InvalidSignature)?;
        verifier
            .verify(
                &request.id,
                &request.timestamp,
                &request.body,
                &request.signature,
            )
            .map_err(|_| ProviderError::InvalidSignature)?;
        Ok(VerifiedWebhook {
            provider: ProviderType::Resend,
            body: request.body,
        })
    }

    fn map_webhook(&self, verified: VerifiedWebhook) -> Result<Vec<ProviderEvent>, ProviderError> {
        let value = serde_json::from_slice::<serde_json::Value>(&verified.body)
            .map_err(|_| ProviderError::InvalidPayload)?;
        let event_id = extract_event_id(&value)?;
        let event = map_resend_event(&value, event_id)?;
        Ok(vec![event])
    }
}

#[derive(Serialize)]
struct ResendSendPayload<'a> {
    from: &'a str,
    to: Vec<&'a str>,
    subject: &'a str,
    text: &'a str,
    html: Option<&'a str>,
}

#[derive(Deserialize)]
struct ResendSendResponse {
    id: String,
}

impl<'a> ResendSendPayload<'a> {
    fn from_request(request: &'a SendMailRequest) -> Self {
        Self {
            from: request.from().as_str(),
            to: request.to().iter().map(|email| email.as_str()).collect(),
            subject: request.subject(),
            text: request.text(),
            html: request.html(),
        }
    }
}

fn send_endpoint(endpoint: &str) -> String {
    format!("{}/emails", endpoint.trim_end_matches('/'))
}

fn received_email_endpoint(endpoint: &str, email_id: &str) -> String {
    format!(
        "{}/emails/receiving/{email_id}",
        endpoint.trim_end_matches('/')
    )
}

fn ensure_success(success: bool) -> Result<(), ProviderError> {
    match success {
        true => Ok(()),
        false => Err(ProviderError::ProviderRejected),
    }
}

fn ensure_webhook_headers(request: &ProviderWebhookRequest) -> Result<(), ProviderError> {
    match request.id.is_empty() || request.signature.is_empty() || request.timestamp.is_empty() {
        true => Err(ProviderError::InvalidSignature),
        false => Ok(()),
    }
}

fn extract_event_id(value: &serde_json::Value) -> Result<String, ProviderError> {
    value
        .pointer("/data/email_id")
        .or_else(|| value.pointer("/data/id"))
        .or_else(|| value.get("id"))
        .and_then(|id| id.as_str())
        .map(str::to_owned)
        .ok_or(ProviderError::InvalidPayload)
}

fn map_resend_event(
    value: &serde_json::Value,
    provider_event_id: String,
) -> Result<ProviderEvent, ProviderError> {
    match event_type(value)? {
        "email.delivered" => Ok(ProviderEvent::Delivered { provider_event_id }),
        "email.bounced" => Ok(ProviderEvent::Bounced { provider_event_id }),
        "email.complained" => Ok(ProviderEvent::Complained { provider_event_id }),
        "email.received" => inbound_event(value, provider_event_id),
        _ => Err(ProviderError::InvalidPayload),
    }
}

fn event_type(value: &serde_json::Value) -> Result<&str, ProviderError> {
    value
        .get("type")
        .and_then(|kind| kind.as_str())
        .ok_or(ProviderError::InvalidPayload)
}

fn inbound_event(
    value: &serde_json::Value,
    provider_event_id: String,
) -> Result<ProviderEvent, ProviderError> {
    Ok(ProviderEvent::Inbound {
        provider_event_id,
        from: inbound_from(value)?,
        to: inbound_to(value)?,
        subject: inbound_subject(value).to_string(),
        text: inbound_text(value).to_string(),
    })
}

fn inbound_from(value: &serde_json::Value) -> Result<EmailAddress, ProviderError> {
    parse_address(
        value
            .pointer("/data/from")
            .and_then(address_value)
            .ok_or(ProviderError::InvalidPayload)?,
    )
}

fn inbound_to(value: &serde_json::Value) -> Result<Vec<EmailAddress>, ProviderError> {
    let recipients = value
        .pointer("/data/to")
        .and_then(|value| value.as_array())
        .ok_or(ProviderError::InvalidPayload)?;
    let parsed = recipients
        .iter()
        .filter_map(address_value)
        .map(parse_address)
        .collect::<Result<Vec<_>, _>>()?;
    reject_empty_inbound_recipients(parsed)
}

fn address_value(value: &serde_json::Value) -> Option<&str> {
    value
        .as_str()
        .or_else(|| value.get("email").and_then(|email| email.as_str()))
}

fn parse_address(value: &str) -> Result<EmailAddress, ProviderError> {
    EmailAddress::parse(value).map_err(|_| ProviderError::InvalidPayload)
}

fn reject_empty_inbound_recipients(
    recipients: Vec<EmailAddress>,
) -> Result<Vec<EmailAddress>, ProviderError> {
    match recipients.is_empty() {
        true => Err(ProviderError::InvalidPayload),
        false => Ok(recipients),
    }
}

fn inbound_subject(value: &serde_json::Value) -> &str {
    value
        .pointer("/data/subject")
        .and_then(|subject| subject.as_str())
        .unwrap_or("")
}

fn inbound_text(value: &serde_json::Value) -> &str {
    value
        .pointer("/data/text")
        .or_else(|| value.pointer("/data/text_body"))
        .or_else(|| value.pointer("/data/html"))
        .and_then(|text| text.as_str())
        .unwrap_or("")
}

#[derive(Deserialize)]
struct ResendReceivedEmailResponse {
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Vec<String>,
    #[serde(default)]
    cc: Vec<String>,
    #[serde(default)]
    bcc: Vec<String>,
    #[serde(default, alias = "replyTo")]
    reply_to: Vec<String>,
    #[serde(default)]
    subject: Option<String>,
    #[serde(default, alias = "text_body", alias = "textBody")]
    text: Option<String>,
    #[serde(default, alias = "html_body", alias = "htmlBody")]
    html: Option<String>,
    #[serde(default)]
    headers: Option<serde_json::Value>,
    #[serde(default)]
    attachments: Vec<ResendReceivedAttachment>,
    #[serde(default)]
    raw: Option<ResendReceivedRaw>,
}

impl ResendReceivedEmailResponse {
    fn into_detail(self) -> InboundMessageDetail {
        InboundMessageDetail {
            from: self.from.unwrap_or_default(),
            to: self.to,
            cc: self.cc,
            bcc: self.bcc,
            reply_to: self.reply_to,
            subject: self.subject.unwrap_or_default(),
            text: self.text.unwrap_or_default(),
            html: self.html,
            headers: received_headers(self.headers),
            attachments: self
                .attachments
                .into_iter()
                .map(ResendReceivedAttachment::into_attachment)
                .collect(),
            raw: self.raw.map(ResendReceivedRaw::into_raw),
        }
    }
}

#[derive(Deserialize)]
struct ResendReceivedAttachment {
    #[serde(default)]
    filename: Option<String>,
    #[serde(default, alias = "contentType")]
    content_type: Option<String>,
    #[serde(default, alias = "contentDisposition")]
    content_disposition: Option<String>,
    #[serde(default, alias = "contentId")]
    content_id: Option<String>,
}

impl ResendReceivedAttachment {
    fn into_attachment(self) -> InboundMessageAttachment {
        InboundMessageAttachment {
            filename: self.filename.unwrap_or_default(),
            content_type: self.content_type.unwrap_or_default(),
            content_disposition: self.content_disposition.unwrap_or_default(),
            content_id: self.content_id,
        }
    }
}

#[derive(Deserialize)]
struct ResendReceivedRaw {
    #[serde(default, alias = "downloadUrl")]
    download_url: Option<String>,
    #[serde(default, alias = "expiresAt")]
    expires_at: Option<String>,
}

impl ResendReceivedRaw {
    fn into_raw(self) -> InboundMessageRaw {
        InboundMessageRaw {
            download_url: self.download_url.unwrap_or_default(),
            expires_at: self.expires_at,
        }
    }
}

fn received_headers(value: Option<serde_json::Value>) -> Vec<InboundMessageHeader> {
    match value {
        Some(serde_json::Value::Array(values)) => array_headers(values),
        Some(serde_json::Value::Object(values)) => object_headers(values),
        _ => Vec::new(),
    }
}

fn array_headers(values: Vec<serde_json::Value>) -> Vec<InboundMessageHeader> {
    values.into_iter().filter_map(array_header).collect()
}

fn array_header(value: serde_json::Value) -> Option<InboundMessageHeader> {
    Some(InboundMessageHeader {
        name: value.get("name")?.as_str()?.to_string(),
        value: value.get("value")?.as_str()?.to_string(),
    })
}

fn object_headers(values: serde_json::Map<String, serde_json::Value>) -> Vec<InboundMessageHeader> {
    values
        .into_iter()
        .filter_map(|(name, value)| {
            value.as_str().map(|value| InboundMessageHeader {
                name,
                value: value.to_string(),
            })
        })
        .collect()
}
