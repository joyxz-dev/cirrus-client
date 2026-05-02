use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use zeroize::Zeroizing;

use crate::error::CirrusError;

const TENANT: &str = "consumers";
const SCOPE: &str = "XboxLive.signin offline_access";
const POLL_INTERVAL_SECS: u64 = 5;

#[derive(Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: Option<u64>,
    message: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DeviceCodeChallenge {
    pub user_code: String,
    pub verification_uri: String,
    pub message: String,
}

pub struct DeviceCodeSession {
    pub challenge: DeviceCodeChallenge,
    pub device_code: Zeroizing<String>,
    pub client_id: String,
    pub poll_interval: u64,
    pub expires_in: u64,
}

pub struct MsaTokens {
    pub access_token: Zeroizing<String>,
    pub refresh_token: Zeroizing<String>,
    #[allow(dead_code)]
    pub expires_in: u64,
}

pub async fn begin_device_code_flow(
    client: &reqwest::Client,
    client_id: &str,
) -> Result<DeviceCodeSession, CirrusError> {
    let device_resp = client
        .post(format!(
            "https://login.microsoftonline.com/{TENANT}/oauth2/v2.0/devicecode"
        ))
        .form(&[("client_id", client_id), ("scope", SCOPE)])
        .send()
        .await?
        .error_for_status()?
        .json::<DeviceCodeResponse>()
        .await?;

    Ok(DeviceCodeSession {
        challenge: DeviceCodeChallenge {
            user_code: device_resp.user_code,
            verification_uri: device_resp.verification_uri,
            message: device_resp.message,
        },
        device_code: Zeroizing::new(device_resp.device_code),
        client_id: client_id.to_string(),
        poll_interval: device_resp.interval.unwrap_or(POLL_INTERVAL_SECS),
        expires_in: device_resp.expires_in,
    })
}

pub async fn complete_device_code_flow(
    client: &reqwest::Client,
    session: DeviceCodeSession,
) -> Result<MsaTokens, CirrusError> {
    poll_for_token(
        client,
        &session.client_id,
        &session.device_code,
        session.poll_interval,
        session.expires_in,
    )
    .await
}

async fn poll_for_token(
    client: &reqwest::Client,
    client_id: &str,
    device_code: &str,
    interval: u64,
    max_wait: u64,
) -> Result<MsaTokens, CirrusError> {
    let deadline = std::time::Instant::now() + Duration::from_secs(max_wait);

    loop {
        sleep(Duration::from_secs(interval)).await;

        if std::time::Instant::now() > deadline {
            return Err(CirrusError::Auth("Device code expired".into()));
        }

        let resp = client
            .post(format!(
                "https://login.microsoftonline.com/{TENANT}/oauth2/v2.0/token"
            ))
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", client_id),
                ("device_code", device_code),
            ])
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;

        match resp.error.as_deref() {
            None => {
                let access = resp
                    .access_token
                    .ok_or_else(|| CirrusError::Auth("Missing access_token".into()))?;
                let refresh = resp
                    .refresh_token
                    .ok_or_else(|| CirrusError::Auth("Missing refresh_token".into()))?;
                return Ok(MsaTokens {
                    access_token: Zeroizing::new(access),
                    refresh_token: Zeroizing::new(refresh),
                    expires_in: resp.expires_in.unwrap_or(3600),
                });
            }
            Some("authorization_pending") | Some("slow_down") => continue,
            Some(e) => return Err(CirrusError::Auth(format!("Token error: {e}"))),
        }
    }
}

pub async fn refresh_msa_token(
    client: &reqwest::Client,
    client_id: &str,
    refresh_token: &str,
) -> Result<MsaTokens, CirrusError> {
    let resp = client
        .post(format!(
            "https://login.microsoftonline.com/{TENANT}/oauth2/v2.0/token"
        ))
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", refresh_token),
            ("scope", SCOPE),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<TokenResponse>()
        .await?;

    if let Some(e) = resp.error {
        return Err(CirrusError::Auth(format!("Refresh failed: {e}")));
    }

    let access = resp
        .access_token
        .ok_or_else(|| CirrusError::Auth("Missing access_token on refresh".into()))?;
    let refresh = resp
        .refresh_token
        .unwrap_or_else(|| refresh_token.to_string());

    Ok(MsaTokens {
        access_token: Zeroizing::new(access),
        refresh_token: Zeroizing::new(refresh),
        expires_in: resp.expires_in.unwrap_or(3600),
    })
}
