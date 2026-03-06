use std::borrow::Cow;

use krill_common::{KrillError, KrillResult};
use lettre::{message::Mailbox, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::EmailEnvelopeDetails;

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

    pub async fn send(
        &self,
        message: &EmailEnvelopeDetails<'_>,
    ) -> KrillResult<lettre::transport::smtp::response::Response> {
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
            .subject(message.subject)
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(message.body.to_string())
            .map_err(|error| KrillError::Mailer(error.to_string()))?;

        self.mailer
            .send(email)
            .await
            .map_err(|error| KrillError::MailDelivery(error.to_string()))
    }
}
