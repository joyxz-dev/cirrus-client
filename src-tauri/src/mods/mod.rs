pub mod installer;

use serde::{Deserialize, Serialize};

use crate::error::CirrusError;

const MODRINTH_BASE: &str = "https://api.modrinth.com/v2";
const USER_AGENT: &str = "Cirrus/0.1.0 (cirrusclient.gg)";

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModSearchQuery {
    pub query: String,
    pub loader: Option<String>,
    pub mc_version: Option<String>,
    pub category: Option<String>,
    pub offset: u32,
    pub limit: u32,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModResult {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub categories: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModSearchResult {
    pub hits: Vec<ModResult>,
    pub total_hits: u64,
    pub offset: u32,
    pub limit: u32,
}

#[derive(Deserialize)]
struct ModrinthSearchResponse {
    hits: Vec<ModrinthHit>,
    total_hits: u64,
    offset: u32,
    limit: u32,
}

#[derive(Deserialize)]
struct ModrinthHit {
    project_id: String,
    slug: String,
    title: String,
    description: String,
    icon_url: Option<String>,
    downloads: u64,
    categories: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModVersion {
    pub id: String,
    pub name: String,
    pub version_number: String,
    pub files: Vec<ModVersionFile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModVersionFile {
    pub url: String,
    pub filename: String,
    pub hashes: ModVersionHashes,
    pub primary: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModVersionHashes {
    pub sha512: String,
    pub sha1: Option<String>,
}

pub fn build_client() -> Result<reqwest::Client, CirrusError> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(Into::into)
}

pub async fn search_mods(
    client: &reqwest::Client,
    query: &ModSearchQuery,
) -> Result<ModSearchResult, CirrusError> {
    let mut facets: Vec<Vec<String>> = vec![vec!["project_type:mod".to_string()]];

    if let Some(loader) = &query.loader {
        facets.push(vec![format!("categories:{loader}")]);
    }
    if let Some(version) = &query.mc_version {
        facets.push(vec![format!("versions:{version}")]);
    }
    if let Some(cat) = &query.category {
        facets.push(vec![format!("categories:{cat}")]);
    }

    let facets_json = serde_json::to_string(&facets)?;

    let resp: ModrinthSearchResponse = client
        .get(format!("{MODRINTH_BASE}/search"))
        .query(&[
            ("query", query.query.as_str()),
            ("facets", &facets_json),
            ("index", "relevance"),
            ("offset", &query.offset.to_string()),
            ("limit", &query.limit.to_string()),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(ModSearchResult {
        hits: resp
            .hits
            .into_iter()
            .map(|h| ModResult {
                id: h.project_id,
                slug: h.slug,
                title: h.title,
                description: h.description,
                icon_url: h.icon_url,
                downloads: h.downloads,
                categories: h.categories,
            })
            .collect(),
        total_hits: resp.total_hits,
        offset: resp.offset,
        limit: resp.limit,
    })
}

pub async fn get_mod_versions(
    client: &reqwest::Client,
    project_id: &str,
    loader: Option<&str>,
    mc_version: Option<&str>,
) -> Result<Vec<ModVersion>, CirrusError> {
    let mut req = client.get(format!("{MODRINTH_BASE}/project/{project_id}/version"));

    let mut params = vec![];
    let loader_json;
    let version_json;

    if let Some(l) = loader {
        loader_json = serde_json::to_string(&[l])?;
        params.push(("loaders", loader_json.as_str()));
    }
    if let Some(v) = mc_version {
        version_json = serde_json::to_string(&[v])?;
        params.push(("game_versions", version_json.as_str()));
    }

    req = req.query(&params);

    req.send()
        .await?
        .error_for_status()?
        .json::<Vec<ModVersion>>()
        .await
        .map_err(Into::into)
}
