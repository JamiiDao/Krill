use krill_common::{KrillError, KrillResult};
use lettre::{message::Mailbox, AsyncSmtpTransport, Tokio1Executor};

use crate::KrillSmtps;

#[derive(Debug, Default)]
pub struct KrillSmtpsBuilder<'a> {
    from: &'a str,
    reply_to: Option<&'a str>,
    hello_name: Option<&'a str>,
}

impl<'a> KrillSmtpsBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_from(&mut self, from: &'a str) -> &mut Self {
        self.from = from;

        self
    }
    pub fn set_reply_to(&mut self, reply_to: &'a str) -> &mut Self {
        self.reply_to.replace(reply_to);

        self
    }

    pub fn set_hello_name(&mut self, hello_name: &'a str) -> &mut Self {
        self.hello_name.replace(hello_name);

        self
    }

    pub async fn build(self, smtps_uri: &'a str) -> KrillResult<KrillSmtps<'a>> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::from_url(smtps_uri)
            .map_err(|error| KrillError::Mailer(error.to_string()))?;

        let mailer = if let Some(hello_name) = self.hello_name {
            transport.hello_name(lettre::transport::smtp::extension::ClientId::Domain(
                hello_name.to_string(),
            ))
        } else {
            transport
        };

        let mailer = mailer.build();

        mailer
            .test_connection()
            .await
            .map_err(|error| KrillError::Mailer(error.to_string()))?;

        Ok(KrillSmtps {
            from: self
                .from
                .parse::<Mailbox>()
                .map_err(|error| KrillError::Mailer(error.to_string()))?,
            reply_to: self
                .reply_to
                .map(|value| {
                    value
                        .parse::<Mailbox>()
                        .map_err(|error| KrillError::Mailer(error.to_string()))
                })
                .transpose()?,
            hello_name: self.hello_name.map(|value| value.into()),
            mailer,
        })
    }
}
