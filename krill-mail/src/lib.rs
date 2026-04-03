mod email_builder;
pub use email_builder::*;

mod transport_builder;
pub use transport_builder::*;

mod mailer;
pub use mailer::*;

#[cfg(test)]
mod test_smtp_service {

    use crate::*;

    // TODO: Finish tests
    //#[test]
    fn test_secure_smtp() {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async move {
            let mut mailer = KrillSmtpsBuilder::new();
            mailer
                .set_from("Support <support@domain.tld>")
                .set_hello_name("domain.tld")
                .set_reply_to("Support <support@domain.tld>");
            let mailer = mailer
                .build("smtps://username@domain.tld:password@smtp.domain-for-smtp-provider.tld:465")
                .unwrap();

            let my_mail = EmailEnvelopeDetails::new()
                .set_to("Foo Bar <foo@example.com>")
                .set_subject("Mail Completed")
                .set_body("Has been successful");
            let outcome = mailer.send(&my_mail).await.unwrap();

            dbg!(&outcome);
        });
    }
}
