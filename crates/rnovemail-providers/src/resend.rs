use async_trait::async_trait;
use reqwest::Client;
use rnovemail_domain::{MessageId, ProviderType};
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
}

impl ResendProvider {
    pub fn new(api_key: SecretString) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
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
        Ok(VerifiedWebhook {
            provider: ProviderType::Resend,
            body: request.body,
        })
    }

    fn map_webhook(&self, verified: VerifiedWebhook) -> Result<Vec<ProviderEvent>, ProviderError> {
        let body = String::from_utf8(verified.body).map_err(|_| ProviderError::InvalidPayload)?;
        let event_id = extract_event_id(&body);
        Ok(vec![ProviderEvent::Delivered {
            provider_event_id: event_id,
        }])
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
    match request.signature.is_empty() || request.timestamp.is_empty() {
        true => Err(ProviderError::InvalidSignature),
        false => Ok(()),
    }
}

fn extract_event_id(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("id")
                .and_then(|id| id.as_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "unknown".to_string())
}
