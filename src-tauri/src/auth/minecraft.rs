use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::error::CirrusError;

#[derive(Deserialize)]
struct XblResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: XblDisplayClaims,
}

#[derive(Deserialize)]
struct XblDisplayClaims {
    xui: Vec<XblXui>,
}

#[derive(Deserialize)]
struct XblXui {
    uhs: String,
}

#[derive(Deserialize)]
struct XstsResponse {
    #[serde(rename = "Token")]
    token: String,
}

#[derive(Deserialize)]
struct McAuthResponse {
    access_token: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
}

pub struct MinecraftSession {
    #[allow(dead_code)]
    pub access_token: Zeroizing<String>,
    pub profile: MinecraftProfile,
    pub expires_at: u64,
}

pub async fn authenticate(
    client: &reqwest::Client,
    msa_access_token: &str,
) -> Result<MinecraftSession, CirrusError> {
    let xbl = get_xbl_token(client, msa_access_token).await?;
    let user_hash = xbl
        .display_claims
        .xui
        .first()
        .map(|x| x.uhs.clone())
        .ok_or_else(|| CirrusError::Auth("No user hash in XBL response".into()))?;

    let xbl_token = Zeroizing::new(xbl.token);
    let xsts_token = Zeroizing::new(get_xsts_token(client, &xbl_token).await?);
    let mc_token = Zeroizing::new(get_mc_token(client, &xsts_token, &user_hash).await?);

    let profile = get_profile(client, &mc_token).await?;
    let expires_at = chrono::Utc::now().timestamp() as u64 + 86400;

    Ok(MinecraftSession {
        access_token: mc_token,
        profile,
        expires_at,
    })
}

async fn get_xbl_token(
    client: &reqwest::Client,
    msa_token: &str,
) -> Result<XblResponse, CirrusError> {
    let body = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={msa_token}")
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });

    client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<XblResponse>()
        .await
        .map_err(Into::into)
}

async fn get_xsts_token(
    client: &reqwest::Client,
    xbl_token: &str,
) -> Result<String, CirrusError> {
    let body = serde_json::json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbl_token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });

    let resp = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await?;

    if resp.status() == 401 {
        return Err(CirrusError::Auth(
            "Xbox account not eligible or 2FA required".into(),
        ));
    }

    let xsts: XstsResponse = resp.error_for_status()?.json().await?;
    Ok(xsts.token)
}

async fn get_mc_token(
    client: &reqwest::Client,
    xsts_token: &str,
    user_hash: &str,
) -> Result<String, CirrusError> {
    let body = serde_json::json!({
        "identityToken": format!("XBL3.0 x={user_hash};{xsts_token}")
    });

    let resp: McAuthResponse = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(resp.access_token)
}

async fn get_profile(
    client: &reqwest::Client,
    mc_token: &str,
) -> Result<MinecraftProfile, CirrusError> {
    client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(mc_token)
        .send()
        .await?
        .error_for_status()?
        .json::<MinecraftProfile>()
        .await
        .map_err(Into::into)
}
