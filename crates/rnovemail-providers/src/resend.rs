use async_trait::async_trait;
use reqwest::Client;
use rnovemail_domain::{MessageId, ProviderType};
use rnovemail_webhook::SignatureVerifier;
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;

use crate::{
    MailProvider, ProviderError, ProviderEvent, ProviderSendReceipt, ProviderWebhookRequest,
    SendMailRequest, VerifiedWebhook,
};

#[derive(Clone)]
pub struct ResendProvider {
    api_key: SecretString,
    client: Client,
    webhook_verifier: Option<SignatureVerifier>,
}

impl ResendProvider {
    pub fn new(api_key: SecretString) -> Self {
        Self {
            api_key,
            client: Client::new(),
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
            .post("https://api.resend.com/emails")
            .bearer_auth(self.api_key.expose_secret())
            .json(&payload)
            .send()
            .await
            .map_err(|_| ProviderError::ProviderRejected)?;
        ensure_success(response.status().is_success())?;
        Ok(ProviderSendReceipt {
            message_id: MessageId::new(),
            provider_message_id: "resend-accepted".to_string(),
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
        "email.received" => Ok(ProviderEvent::Inbound { provider_event_id }),
        _ => Err(ProviderError::InvalidPayload),
    }
}

fn event_type(value: &serde_json::Value) -> Result<&str, ProviderError> {
    value
        .get("type")
        .and_then(|kind| kind.as_str())
        .ok_or(ProviderError::InvalidPayload)
}
