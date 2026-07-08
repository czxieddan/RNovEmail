use async_trait::async_trait;
use reqwest::Client;
use rnovemail_domain::{EmailAddress, MessageId, ProviderType};
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
