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
struct XstsErrorResponse {
    #[serde(rename = "XErr")]
    xerr: Option<u64>,
    #[serde(rename = "Message")]
    message: Option<String>,
}

#[derive(Deserialize)]
struct McAuthResponse {
    access_token: String,
    expires_in: Option<u64>,
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
    let (mc_token_raw, expires_in) = get_mc_token(client, &xsts_token, &user_hash).await?;
    let mc_token = Zeroizing::new(mc_token_raw);

    let profile = get_profile(client, &mc_token).await?;
    let expires_at = chrono::Utc::now().timestamp() as u64 + expires_in;

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
        let err_body: XstsErrorResponse = resp.json().await.unwrap_or(XstsErrorResponse {
            xerr: None,
            message: None,
        });
        let msg = match err_body.xerr {
            Some(2148916233) => "This Microsoft account has no Xbox account. Please create one at xbox.com.".into(),
            Some(2148916235) => "Xbox Live is not available in your region.".into(),
            Some(2148916236) | Some(2148916237) => "Adult verification required. Please complete it at xbox.com.".into(),
            Some(2148916238) => "Child accounts must be added to a family by an adult.".into(),
            Some(code) => format!("Xbox auth error (XErr={code})"),
            None => err_body.message.unwrap_or_else(|| "Xbox account not eligible".into()),
        };
        return Err(CirrusError::Auth(msg));
    }

    let xsts: XstsResponse = resp.error_for_status()?.json().await?;
    Ok(xsts.token)
}

async fn get_mc_token(
    client: &reqwest::Client,
    xsts_token: &str,
    user_hash: &str,
) -> Result<(String, u64), CirrusError> {
    let body = serde_json::json!({
        "identityToken": format!("XBL3.0 x={user_hash};{xsts_token}")
    });

    let resp = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        return Err(CirrusError::Auth(format!(
            "Minecraft login failed ({status}): {body_text}"
        )));
    }

    let parsed: McAuthResponse = resp.json().await?;
    Ok((parsed.access_token, parsed.expires_in.unwrap_or(86400)))
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
