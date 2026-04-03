use dioxus::{fullstack::ServerEvents, prelude::*};

#[cfg(feature = "server")]
use krill_common::{KrillError, KrillResult, OrganizationInfo, ServerConfigurationState};
#[cfg(feature = "server")]
use krill_mail::KrillSmtpsBuilder;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use solana_tx_parser::{JsonRpcCluster, SolanaTxParserUtils};

#[cfg(feature = "server")]
use crate::{
    backend::{store, ADMIN_SECRET, SERVER_APP_STATE, SERVER_MAIL_CONNECTION},
    CacheOrgInfo,
};

use crate::ProgressStateToUiRecord;

#[server]
pub async fn verification_stream(
    data: ProgressStateToUiRecord,
) -> dioxus::Result<ServerEvents<ConfigVerificationOutcome>> {
    Ok(ServerEvents::new(
        |mut tx: dioxus_fullstack::SseTx<ConfigVerificationOutcome>| async move {
            if let Some(value) = ADMIN_SECRET.get() {
                if value.write().await.is_expired_after_60()
                    && tx
                        .send(ConfigVerificationOutcome::Failure(
                            "ADMIN PASSCODE already expired! Restart server to get a fresh one"
                                .to_string(),
                        ))
                        .await
                        .is_err()
                {
                    return;
                }
            } else {
                if tx
                    .send(ConfigVerificationOutcome::Failure(
                        "Unable to get `ADMIN_SECRET` value".to_string(),
                    ))
                    .await
                    .is_err()
                {
                    return;
                }
            }

            if let Err(error) = validate_details(
                data.smtp_info.as_ref(),
                data.api_key.as_ref(),
                data.passcode.as_ref(),
            ) {
                if tx
                    .send(ConfigVerificationOutcome::Failure(error.to_string()))
                    .await
                    .is_err()
                {
                    return;
                }
            }

            let org_info = match validate_org_details(data.org_info) {
                Ok(value) => value,
                Err(error) => {
                    if tx
                        .send(ConfigVerificationOutcome::Failure(error.to_string()))
                        .await
                        .is_err()
                    {
                        return;
                    }

                    return;
                }
            };

            let support_mail = org_info.support_mail.clone();
            let domain_name = data.domain_name.as_ref().unwrap().trim().to_string();

            if tx
                .send(ConfigVerificationOutcome::TestingSmtp)
                .await
                .is_err()
            {
                return;
            }

            match ConfigVerificationOutcome::test_smtp_config(
                data.smtp_info.as_ref().unwrap().trim(),
                org_info.name.clone(),
                support_mail,
                domain_name.to_string(),
            )
            .await
            {
                Ok(event) => {
                    if tx.send(event).await.is_err() {
                        return;
                    }
                }
                Err(error) => {
                    if tx
                        .send(ConfigVerificationOutcome::Failure(error.to_string()))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
            }

            if tx
                .send(ConfigVerificationOutcome::TestingApiKey)
                .await
                .is_err()
            {
                return;
            }
            match ConfigVerificationOutcome::test_api_key(data.api_key.as_ref().unwrap().trim())
                .await
            {
                Ok(event) => if tx.send(event).await.is_err() {},
                Err(error) => {
                    if tx
                        .send(ConfigVerificationOutcome::Failure(error.to_string()))
                        .await
                        .is_err()
                    {}
                }
            }

            if tx
                .send(ConfigVerificationOutcome::CreatingOrganization)
                .await
                .is_err()
            {
                return;
            }
            match ConfigVerificationOutcome::create_organization(org_info).await {
                Ok(event) => if tx.send(event).await.is_err() {},
                Err(error) => {
                    if tx
                        .send(ConfigVerificationOutcome::Failure(error.to_string()))
                        .await
                        .is_err()
                    {}
                }
            }
        },
    ))
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ConfigVerificationOutcome {
    Failure(String),
    TestingSmtp,
    TestingSmtpSuccess,
    TestingSmtpFailure(String),
    TestingApiKey,
    TestingApiKeySuccess,
    TestingApiKeyFailure(String),
    CreatingOrganization,
    OrganizationCreated,
    OrganizationCreationFailed(String),
}

#[cfg(feature = "server")]
impl ConfigVerificationOutcome {
    pub async fn test_smtp_config(
        smtps_uri: &str,
        org_name: String,
        support_mail: String,
        hello_name: String,
    ) -> KrillResult<ConfigVerificationOutcome> {
        let mut mailer = KrillSmtpsBuilder::new();
        mailer
            .set_from(&format!("{} <{}>", org_name, support_mail))
            .set_hello_name(&hello_name)
            .set_reply_to(&format!("{} <{}>", org_name, support_mail));
        let mailer = mailer.build(smtps_uri).await?;

        let outcome = mailer.test_connection().await;

        match outcome {
            Ok(is_success) => {
                if is_success {
                    if let Err(error) = SERVER_MAIL_CONNECTION
                        .set(mailer)
                        .or(Err(KrillError::SmtpsStaticAlreadyInitialized))
                    {
                        Ok(ConfigVerificationOutcome::Failure(error.to_string()))
                    } else {
                        Ok(ConfigVerificationOutcome::TestingSmtpSuccess)
                    }
                } else {
                    Ok(ConfigVerificationOutcome::TestingSmtpFailure(
                        "Invalid SMTPs settings".to_string(),
                    ))
                }
            }
            Err(error) => Ok(ConfigVerificationOutcome::TestingSmtpFailure(format!(
                "Unable to test SMTP Settings. {}",
                error
            ))),
        }
    }

    async fn test_api_key(api_key: &str) -> KrillResult<ConfigVerificationOutcome> {
        let mut client = SolanaTxParserUtils::new(JsonRpcCluster::Custom(api_key.to_string()));

        let parsed = client.get_version().await?;

        if parsed.result.is_some() {
            Ok(ConfigVerificationOutcome::TestingApiKeySuccess)
        } else {
            Ok(ConfigVerificationOutcome::TestingApiKeyFailure(
                parsed
                    .error
                    .map(|error| error.message)
                    .unwrap_or(String::from(
                        "Encountered error when parsing the `getVersion` method",
                    )),
            ))
        }
    }

    async fn create_organization(
        org_info: OrganizationInfo,
    ) -> KrillResult<ConfigVerificationOutcome> {
        let storage = store()?;

        if let Err(error) = storage.update_org_info(org_info).await {
            Ok(ConfigVerificationOutcome::Failure(error.to_string()))
        } else {
            ADMIN_SECRET
                .get()
                .ok_or(KrillError::Transmit(
                    "Unable to get `ADMIN_SECRET`!".to_string(),
                ))?
                .write()
                .await
                .clear();

            if let Err(error) = storage.set_app_state_login_init().await {
                return Ok(ConfigVerificationOutcome::Failure(error.to_string()));
            }

            let server_app_state = SERVER_APP_STATE.get().ok_or(KrillError::Transmit(
                "Unable to get `SERVER_APP_STATE`!".to_string(),
            ))?;

            *server_app_state.write().await = ServerConfigurationState::LoginInitialization;

            Ok(ConfigVerificationOutcome::OrganizationCreated)
        }
    }
}

#[cfg(feature = "server")]
fn validate_details<'a>(
    smtp_info: Option<&String>,
    api_key: Option<&String>,
    passcode: Option<&String>,
) -> Result<(), &'a str> {
    if smtp_info.is_none() {
        return Err("SMTP(s) configuration is missing");
    }

    if api_key.is_none() {
        return Err("Api Key is missing");
    }

    if passcode.is_none() {
        return Err("Passcode is missing");
    }

    Ok(())
}

#[cfg(feature = "server")]
fn validate_org_details<'a>(details: CacheOrgInfo) -> Result<OrganizationInfo, &'a str> {
    let mut org_info = OrganizationInfo::default();

    if let Some(name) = details.name.clone() {
        org_info.name = name.trim().to_string();
    } else {
        return Err("Organization info is missing");
    }

    if let Some((favicon_bytes, mime)) = details.favicon.clone() {
        org_info.favicon = (favicon_bytes, mime.trim().to_string());
    } else {
        return Err("Organization favicon is missing");
    }

    if let Some((logo_bytes, mime)) = details.logo.clone() {
        org_info.logo = (logo_bytes, mime.trim().to_string());
    } else {
        return Err("Organization logo is missing");
    }

    if let Some(support_mail) = details.support_mail.clone() {
        org_info.support_mail = support_mail.trim().to_string();

        Ok(org_info)
    } else {
        Err("Organization support email is missing")
    }
}
