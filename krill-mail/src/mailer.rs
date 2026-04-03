use std::borrow::Cow;

use krill_common::{KrillError, KrillResult};
use lettre::{
    message::Mailbox, transport::smtp::response::Severity, AsyncSmtpTransport, AsyncTransport,
    Message, Tokio1Executor,
};

use crate::EmailEnvelopeDetails;

#[derive(Debug)]
pub struct KrillSmtps<'a> {
    pub(crate) from: Mailbox,
    pub(crate) reply_to: Option<Mailbox>,
    pub(crate) hello_name: Option<Cow<'a, str>>,
    pub(crate) mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl<'a> KrillSmtps<'a> {
    pub fn from(&self) -> &Mailbox {
        &self.from
    }

    pub fn reply_to(&self) -> Option<&Mailbox> {
        self.reply_to.as_ref()
    }

    pub fn hello_name(&self) -> Option<&str> {
        self.hello_name.as_ref().map(|value| value.as_ref())
    }

    pub fn mailer(&self) -> &AsyncSmtpTransport<Tokio1Executor> {
        &self.mailer
    }

    pub async fn send(&self, message: &EmailEnvelopeDetails) -> KrillResult<()> {
        let email = Message::builder().from(self.from().clone());

        let email = if let Some(reply_to) = self.reply_to() {
            email.reply_to(reply_to.clone())
        } else {
            email
        };

        let email = email
            .to(message
                .to
                .parse::<Mailbox>()
                .map_err(|error| KrillError::Mailer(error.to_string()))?)
            .subject(message.subject.as_str())
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(message.body.to_string())
            .map_err(|error| KrillError::Mailer(error.to_string()))?;

        let response = self
            .mailer
            .send(email)
            .await
            .map_err(|error| KrillError::MailDelivery(error.to_string()))?;

        if response.code().severity == Severity::PositiveCompletion {
            Ok(())
        } else {
            Err(KrillError::Smtps(
                response.message().map(|msg| msg.to_string()).collect(),
            ))
        }
    }

    pub async fn test_connection(&self) -> KrillResult<bool> {
        self.mailer
            .test_connection()
            .await
            .map_err(|error| KrillError::Mailer(error.to_string()))
    }
}
