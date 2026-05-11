use dioxus::{
    fullstack::{response::Response, ServerEvents, SetCookie},
    prelude::*,
};

#[cfg(feature = "server")]
use krill_common::{FAVICON_DEFAULT, LOGO_DEFAULT};
use serde::{Deserialize, Serialize};

use crate::ProgressStateToUiRecord;

#[cfg(feature = "server")]
use {
    dioxus::fullstack::headers::Header,
    krill_common::{AuthTokenDetails, Holder, ServerConfigurationState},
    krill_mail::{EmailEnvelopeDetails, KrillSmtpsBuilder},
    solana_tx_parser::{JsonRpcCluster, SolanaTxParserUtils},
};

use krill_common::{KrillError, KrillResult, OrganizationInfo, UserRole, VerifyMailDetailsToUi};

#[cfg(feature = "server")]
use crate::{
    backend::{
        store, ServerUtils, ADMIN_SECRET, SERVER_API_KEY, SERVER_APP_STATE, SERVER_DOMAIN_NAME,
        SERVER_MAIL_CONNECTION, SERVER_ORG_INFO,
    },
    CacheOrgInfo,
};

#[cfg(feature = "server")]
type SseTxInner = dioxus_fullstack::SseTx<ConfigVerificationOutcome>;

#[server]
pub async fn verification_stream(
    data: ProgressStateToUiRecord,
) -> dioxus::Result<ServerEvents<ConfigVerificationOutcome>> {
    Ok(ServerEvents::new(
        |mut tx: dioxus_fullstack::SseTx<ConfigVerificationOutcome>| async move {
            if !validate_details(
                &mut tx,
                data.smtp_info.as_ref(),
                data.api_key.as_ref(),
                data.passcode.as_ref(),
            )
            .await
            {
                tx.close_channel();

                return;
            }

            if !validate_passcode(&mut tx, data.passcode.unwrap().trim()).await {
                tx.close_channel();

                return;
            }

            let org_info = data.org_info;
            let (success_parsing, org_info_parsed) = validate_org_details(&mut tx, org_info).await;
            if !success_parsing {
                tx.close_channel();

                return;
            }

            let org_name = org_info_parsed.name.clone();
            let support_mail = org_info_parsed.support_mail.clone();
            let domain_name = data.domain_name.as_ref().unwrap().trim().to_string();
            let smtps_uri = data.smtp_info.as_ref().unwrap().trim();
            let api_key = data.api_key.as_ref().unwrap().trim();

            if org_name.is_empty() || org_name.len() < 2 {
                tx.send(ConfigVerificationOutcome::Failure(
                    "Invalid organization name".to_string(),
                ))
                .await
                .err();

                return;
            }

            if !ConfigVerificationOutcome::test_smtp_config(
                &mut tx,
                smtps_uri,
                org_name.clone(),
                support_mail,
                domain_name.to_string(),
            )
            .await
            {
                tx.close_channel();

                return;
            }

            if !ConfigVerificationOutcome::test_api_key(&mut tx, api_key).await {
                tx.close_channel();

                return;
            }

            if !ConfigVerificationOutcome::create_organization(
                &mut tx,
                smtps_uri,
                &domain_name,
                api_key,
                org_info_parsed,
            )
            .await
            {
                tx.close_channel();
            }
        },
    ))
}

#[get("/verification-support-mail-link/{token}")]
pub async fn verify_support_mail(token: String) -> ServerFnResult<Response> {
    let mut res = Response::new(axum::body::Body::empty());

    let storage = store().map_err(|error| ServerFnError::ServerError {
        message: error.to_string(),
        code: 500,
        details: None,
    })?;

    let parsed_token = ServerUtils::parse_token(&token)?;

    let login_init_details =
        storage
            .get_superuser_auth_token()
            .await
            .map_err(|error| ServerFnError::ServerError {
                message: error.to_string(),
                code: 500,
                details: None,
            })?;

    let auth_details = if let Some(auth_details) = login_init_details {
        auth_details
    } else {
        redirect_error_header(
            &mut res,
            "Auth details not found! Possibly already verified",
        )?;

        return Ok(res);
    };

    if !AuthTokenDetails::const_cmp(auth_details.token, &parsed_token) {
        redirect_error_header(&mut res, "Unauthorized user")?;

        return Ok(res);
    }

    if auth_details.details.is_expired() {
        redirect_error_header(&mut res, "The link is already expired!")?;

        return Ok(res);
    }

    if auth_details.details.holder().role() != UserRole::Superuser {
        redirect_error_header(&mut res, "Unauthorized user")?;

        return Ok(res);
    }

    let state = SERVER_APP_STATE.get().ok_or(ServerFnError::ServerError {
        message: "Unable to transition the state of the server `SERVER_APP_STATE`".to_string(),
        code: 400,
        details: None,
    })?;

    let auth_token = AuthTokenDetails::generate_token();

    let store_key = storage
        .set_auth_token(auth_token, auth_details.details.clone())
        .await
        .map_err(|error| ServerFnError::ServerError {
            message: error.to_string(),
            code: 500,
            details: None,
        })?;

    *state.write().await = ServerConfigurationState::Initialized;

    redirect_success_header(&mut res)?;
    build_cookie(
        &mut res,
        &auth_details
            .details
            .auth_token_as_cookie_raw(store_key, "Lax"),
    )?;

    Ok(res)
}

#[server]
pub async fn send_superuser_login_auth_link() -> ServerFnResult<Vec<u8>> {
    let org_info = crate::ServerUtils::request_get_org()?;

    let holder = Holder::new_with_tld(&org_info.support_mail)
        .map_err(|error| {
            let error_message = "Error-HolderNew";

            tracing::error!("{error_message}. Error: `{error:?}`");

            ServerFnError::ServerError {
                message: error_message.to_string() + ": Internal server error",
                code: 500,
                details: None,
            }
        })?
        .set_superuser();

    send_auth_email_processor(
        holder,
        "Verify that you control this email address",
        "Click or Tap the link below to verify that you control the support email address",
    )
    .await
    .map(|value| bitcode::encode(&value))
    .map_err(|error| {
        let error_message = "Error-Sending Mail";

        tracing::error!("{error_message}. Error: `{error:?}`");

        ServerFnError::ServerError {
            message: error_message.to_string() + ": Internal server error",
            code: 500,
            details: None,
        }
    })
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
    OrganizationCreated(Vec<u8>), //Returns the new org info
    OrganizationCreationFailed(String),
}

#[cfg(feature = "server")]
impl ConfigVerificationOutcome {
    async fn handler_with_error<T: core::fmt::Debug>(
        tx: &mut SseTxInner,
        value: KrillResult<T>,
    ) -> Option<T> {
        match value {
            Ok(value) => Some(value),
            Err(error) => {
                tx.send(ConfigVerificationOutcome::Failure(error.to_string()))
                    .await
                    .err();

                tx.close_channel();

                None
            }
        }
    }

    async fn tx_handler(tx: &mut SseTxInner, outcome: ConfigVerificationOutcome) -> bool {
        if tx.send(outcome).await.is_err() {
            tx.close_channel();

            false
        } else {
            true
        }
    }

    async fn tx_error_handler(tx: &mut SseTxInner, outcome: ConfigVerificationOutcome) -> bool {
        let _ = tx.send(outcome).await.is_err();

        tx.close_channel();

        false
    }

    async fn tx_org_failure_handler(tx: &mut SseTxInner, error: &str) -> bool {
        if tx
            .send(ConfigVerificationOutcome::OrganizationCreationFailed(
                error.to_string(),
            ))
            .await
            .is_err()
        {
            tx.close_channel();
        }

        false
    }

    pub async fn test_smtp_config(
        tx: &mut SseTxInner,
        smtps_uri: &str,
        org_name: String,
        support_mail: String,
        hello_name: String,
    ) -> bool {
        if tx
            .send(ConfigVerificationOutcome::TestingSmtp)
            .await
            .is_err()
        {
            tx.close_channel();
            return false;
        }

        let mut mailer = KrillSmtpsBuilder::new();
        mailer
            .set_from(&format!("{} <{}>", org_name, support_mail))
            .set_hello_name(&hello_name)
            .set_reply_to(&format!("{} <{}>", org_name, support_mail));

        let mailer = if let Some(value) = Self::handler_with_error(tx, mailer.build(smtps_uri))
            .await
            .take()
        {
            value
        } else {
            tx.close_channel();

            return false;
        };

        let outcome = mailer.test_connection().await;

        match outcome {
            Ok(is_success) => {
                if is_success {
                    return Self::tx_handler(tx, ConfigVerificationOutcome::TestingSmtpSuccess)
                        .await;
                } else {
                    return Self::tx_error_handler(
                        tx,
                        ConfigVerificationOutcome::TestingSmtpFailure(
                            "Invalid SMTPs settings".to_string(),
                        ),
                    )
                    .await;
                }
            }
            Err(error) => {
                return Self::tx_error_handler(
                    tx,
                    ConfigVerificationOutcome::TestingSmtpFailure(format!(
                        "Unable to test SMTP Settings. {}",
                        error
                    )),
                )
                .await
            }
        }
    }

    async fn test_api_key(tx: &mut SseTxInner, api_key: &str) -> bool {
        if tx
            .send(ConfigVerificationOutcome::TestingApiKey)
            .await
            .is_err()
        {
            tx.close_channel();
            return false;
        }

        let mut client = SolanaTxParserUtils::new(JsonRpcCluster::Custom(api_key.to_string()));

        let parsed = if let Some(value) = Self::handler_with_error(tx, client.get_version().await)
            .await
            .take()
        {
            value
        } else {
            tx.close_channel();

            return false;
        };

        if parsed.result.is_some() {
            Self::tx_handler(tx, ConfigVerificationOutcome::TestingApiKeySuccess).await
        } else {
            Self::tx_handler(
                tx,
                ConfigVerificationOutcome::TestingApiKeyFailure(
                    parsed
                        .error
                        .map(|error| error.message)
                        .unwrap_or(String::from(
                            "Encountered error when parsing the `getVersion` method",
                        )),
                ),
            )
            .await
        }
    }

    async fn create_organization(
        tx: &mut SseTxInner,
        smtps_uri: &str,
        fqdn: &str,
        api_key: &str,
        org_info: OrganizationInfo,
    ) -> bool {
        let media_checker = |bytes: &[u8]| -> bool {
            wasm_toolkit::WasmToolkitCommon::to_file_format_kind(bytes) == file_format::Kind::Image
        };

        if !media_checker(&org_info.logo) {
            return Self::tx_org_failure_handler(tx, "Invalid Logo. Only images are accepted")
                .await;
        }

        if !media_checker(&org_info.favicon) {
            return Self::tx_org_failure_handler(tx, "Invalid Logo. Only images are accepted")
                .await;
        }

        if SERVER_MAIL_CONNECTION.get().is_some() {
            return Self::tx_org_failure_handler(
                tx,
                "`SERVER_MAIL_CONNECTION` already initialized!",
            )
            .await;
        }

        if SERVER_DOMAIN_NAME.get().is_some() {
            return Self::tx_org_failure_handler(tx, "`SERVER_DOMAIN_NAME` already initialized!")
                .await;
        }

        if SERVER_API_KEY.get().is_some() {
            return Self::tx_org_failure_handler(tx, "`SERVER_API_KEY` already initialized!").await;
        }

        if ADMIN_SECRET.get().is_none() {
            return Self::tx_org_failure_handler(tx, "`SERVER_DOMAIN_NAME` already initialized!")
                .await;
        }

        if !Self::tx_handler(tx, ConfigVerificationOutcome::CreatingOrganization).await {
            tx.close_channel();

            return false;
        }

        if let Some(server_app_state) = SERVER_APP_STATE.get() {
            *server_app_state.write().await = ServerConfigurationState::LoginInitialization;
        } else {
            return Self::tx_org_failure_handler(tx, "`SERVER_APP_STATE` already initialized!")
                .await;
        }

        let storage = match store() {
            Ok(value) => value,
            Err(error) => {
                return Self::tx_org_failure_handler(tx, error.to_string().as_str()).await;
            }
        };

        // Ensure this also fails before attempting to commit
        if let Some(inner) = ADMIN_SECRET.get() {
            inner.write().await.clear();
        } else {
            return Self::tx_org_failure_handler(
                tx,
                "`ADMIN_SECRET` is not initialized! This is not supposed to happen!",
            )
            .await;
        }

        if let Err(error) = storage
            .set_all_org_details(org_info.clone(), smtps_uri, api_key, fqdn)
            .await
        {
            return Self::tx_org_failure_handler(tx, error.to_string().as_str()).await;
        }
        if let Err(error) = storage.set_app_state_login_init().await {
            return Self::tx_org_failure_handler(tx, error.to_string().as_str()).await;
        }

        let mut mailer = KrillSmtpsBuilder::new();
        mailer
            .set_from(&format!("{} <{}>", org_info.name, org_info.support_mail))
            .set_hello_name(fqdn)
            .set_reply_to(&format!("{} <{}>", org_info.name, org_info.support_mail));
        let mailer = match mailer.build(smtps_uri) {
            Ok(value) => value,
            Err(error) => {
                return Self::tx_org_failure_handler(tx, error.to_string().as_str()).await;
            }
        };

        SERVER_DOMAIN_NAME.get_or_init(|| fqdn.to_string());
        SERVER_API_KEY.get_or_init(|| api_key.to_string());
        SERVER_ORG_INFO.get_or_init(|| org_info);
        SERVER_MAIL_CONNECTION.get_or_init(|| mailer);

        if let Some(info) = SERVER_ORG_INFO.get().cloned() {
            Self::tx_handler(
                tx,
                ConfigVerificationOutcome::OrganizationCreated(bitcode::encode(&info)),
            )
            .await
        } else {
            return Self::tx_org_failure_handler(tx, "organization was created successfully but server was not able to send the registered org info").await;
        }
    }
}

#[cfg(feature = "server")]
async fn validate_details(
    tx: &mut SseTxInner,
    smtp_info: Option<&String>,
    api_key: Option<&String>,
    passcode: Option<&String>,
) -> bool {
    if smtp_info.is_none() {
        tx.send(ConfigVerificationOutcome::Failure(
            "SMTP(s) configuration is missing".to_string(),
        ))
        .await
        .err();

        return false;
    }

    if api_key.is_none() {
        tx.send(ConfigVerificationOutcome::Failure(
            "SMTP(s) configuration is missing".to_string(),
        ))
        .await
        .err();

        return false;
    }

    if passcode.is_none() {
        tx.send(ConfigVerificationOutcome::Failure(
            "Passcode is missing".to_string(),
        ))
        .await
        .err();

        return false;
    }

    true
}

#[cfg(feature = "server")]
async fn validate_passcode(tx: &mut SseTxInner, passcode: &str) -> bool {
    if let Some(admin_config) = ADMIN_SECRET.get() {
        let stored_passcode = admin_config.read().await.secret_to_string();

        if admin_config.read().await.is_expired_after_60() {
            tx.send(ConfigVerificationOutcome::Failure(
                "Passcode expired".to_string(),
            ))
            .await
            .err();

            return false;
        }

        if stored_passcode != passcode {
            tx.send(ConfigVerificationOutcome::Failure(
                "Passcode is invalid".to_string(),
            ))
            .await
            .err();

            return false;
        }

        true
    } else {
        tx.send(ConfigVerificationOutcome::Failure(
            "`ADMIN_SECRET` static not initialized! Server error!".to_string(),
        ))
        .await
        .err();

        false
    }
}

#[cfg(feature = "server")]
async fn validate_org_details(
    tx: &mut SseTxInner,
    details: CacheOrgInfo,
) -> (bool, OrganizationInfo) {
    let mut org_info = OrganizationInfo::default();

    if let Some(name) = details.name.clone() {
        org_info.name = name.trim().to_string();
    } else {
        tx.send(ConfigVerificationOutcome::Failure(
            "Organization name is missing".to_string(),
        ))
        .await
        .err();

        return (false, org_info);
    }

    if details.favicon.is_empty() {
        if details.logo.is_empty() {
            org_info.favicon = FAVICON_DEFAULT.to_vec();
        } else {
            org_info.favicon = details.logo.clone();
        }
    } else {
        org_info.favicon = details.favicon.clone();
    }

    if details.logo.is_empty() {
        org_info.logo = LOGO_DEFAULT.to_vec();
    } else {
        org_info.logo = details.logo.clone();
    }

    if let Some(support_mail) = details.support_mail.clone() {
        org_info.support_mail = support_mail.trim().to_string();
    } else {
        tx.send(ConfigVerificationOutcome::Failure(
            "Organization support email is missing".to_string(),
        ))
        .await
        .err();

        return (false, org_info);
    }

    (true, org_info)
}

#[cfg(feature = "server")]
fn build_cookie(res: &mut Response, auth_token_as_cookie: &str) -> ServerFnResult<()> {
    res.headers_mut().append(
        SetCookie::name(),
        auth_token_as_cookie
            .parse::<dioxus_fullstack::HeaderValue>()
            .map_err(|error| {
                tracing::error!(
                    "Unable to set cookie header. Error: `{}`",
                    error.to_string()
                );

                ServerFnError::ServerError {
                    message: "Unable to set cookie header".to_string(),
                    code: 500,
                    details: None,
                }
            })?,
    );

    Ok(())
}

#[cfg(feature = "server")]
fn redirect_error_header(res: &mut Response, error: &str) -> ServerFnResult<()> {
    *res.status_mut() = StatusCode::SEE_OTHER;

    use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

    /// https://url.spec.whatwg.org/#fragment-percent-encode-set
    const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
    let encoded = utf8_percent_encode(error, FRAGMENT).to_string();

    let route = (crate::RouteUtils::ERRORS.to_string() + "/" + encoded.as_str())
        .parse::<dioxus_fullstack::HeaderValue>()
        .map_err(|inner_error| {
            tracing::error!(
                "Unable to set redirect error header. Error: `{}`",
                inner_error.to_string()
            );

            ServerFnError::ServerError {
                message: error.to_string(),
                code: 500,
                details: None,
            }
        })?;

    res.headers_mut().insert("Location", route);

    Ok(())
}

#[cfg(feature = "server")]
fn redirect_success_header(res: &mut Response) -> ServerFnResult<()> {
    *res.status_mut() = StatusCode::TEMPORARY_REDIRECT;

    let route = crate::RouteUtils::DASHBOARD
        .parse::<dioxus_fullstack::HeaderValue>()
        .map_err(|error| {
            tracing::error!(
                "Unable to set redirect error header. Error: `{}`",
                error.to_string()
            );

            ServerFnError::ServerError {
                message: "Unable to set redirect error header route".to_string(),
                code: 500,
                details: None,
            }
        })?;

    res.headers_mut().insert("Location", route);

    Ok(())
}

#[cfg(feature = "server")]
async fn send_auth_email_processor(
    holder: Holder,
    subject: &str,
    body: &str,
) -> KrillResult<VerifyMailDetailsToUi> {
    let domain = SERVER_DOMAIN_NAME
        .get()
        .ok_or(KrillError::Statics("`SERVER_DOMAIN_NAME` not set"))?;

    let storage = store()?;

    let new_issue: bool;
    let mut auth_token = [0u8; AuthTokenDetails::AUTH_TOKEN_LEN];

    let auth_details = if let Some(auth_details) = storage.get_superuser_auth_token().await? {
        new_issue = false;

        auth_token = auth_details.token;
        auth_details.details
    } else {
        new_issue = true;

        AuthTokenDetails::new(holder.clone())
    };

    if new_issue || auth_details.can_resend() {
        let superuser_auth_token = storage.set_superuser_token(holder.clone()).await?;
        auth_token = superuser_auth_token.token;

        let mailer = SERVER_MAIL_CONNECTION
            .get()
            .ok_or(KrillError::Statics("`SERVER_MAIL_CONNECTION` not set"))?;

        let message = EmailEnvelopeDetails::new()
            .set_to(&holder.email_envelope_details())
            .set_subject(subject)
            .set_body(&html_code_template(
                domain,
                &AuthTokenDetails::store_key_bytes_to_hex(auth_token),
                body,
                &auth_details.expiry_formatted(),
            ));

        mailer.send(&message).await?;

        Ok((auth_token, superuser_auth_token.details).into())
    } else {
        Ok((auth_token, auth_details).into())
    }
}

fn html_code_template(domain: &str, code: &str, body: &str, expiry: &str) -> String {
    let query = "/verification-support-mail-link/";
    let uri = if domain.starts_with("localhost") || domain.starts_with("127.0.0.1") {
        String::from("http://") + domain + ":8080" + query + code
    } else {
        String::from("https://") + domain + query + code
    };

    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>Verification Link</title>
</head>
<body style="margin:0; padding:0; background-color:#f4f4f4; font-family: Arial, sans-serif;">

  <table width="100%" cellpadding="0" cellspacing="0" border="0" style="background-color:#f4f4f4; padding:20px 0;">
    <tr>
      <td align="center">

        <!-- Container -->
        <table width="400" cellpadding="0" cellspacing="0" border="0" style="background:#ffffff; border-radius:8px; padding:20px; text-align:center;">
          
          <!-- Title -->
          <tr>
            <td style="font-size:20px; font-weight:bold; color:#333;">
              Verification Link
            </td>
          </tr>

          <!-- Spacer -->
          <tr><td style="height:10px;"></td></tr>

          <!-- Message -->
          <tr>
            <td style="font-size:14px; color:#555;">
              {body}. This link will expire in {expiry}.
            </td>
          </tr>

          <!-- Spacer -->
          <tr><td style="height:20px;"></td></tr>

          <!-- Code -->
          <td style="font-size:12px; color:#000;">
              <a href="{uri}">{uri}</a>
        </td>

          <!-- Spacer -->
          <tr><td style="height:20px;"></td></tr>

          <!-- Footer -->
          <tr>
            <td style="font-size:12px; color:#888;">
              If you did not request this code, you can safely ignore this email.
            </td>
          </tr>

        </table>

      </td>
    </tr>
  </table>

</body>
</html>
"#
    )
}
