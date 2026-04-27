use crate::error::OsmprjError;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

const INDEX_URL: &str = "https://download.geofabrik.de/index-v1.json";

#[derive(Debug, Deserialize)]
pub struct GeofabrikUrls {
    pub pbf: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GeofabrikProperties {
    pub id: String,
    pub urls: Option<GeofabrikUrls>,
}

#[derive(Debug, Deserialize)]
pub struct GeofabrikFeature {
    pub properties: GeofabrikProperties,
}

#[derive(Debug, Deserialize)]
struct GeofabrikIndex {
    features: Vec<GeofabrikFeature>,
}

fn cache_path() -> Result<PathBuf, OsmprjError> {
    dirs::cache_dir()
        .map(|p| p.join("osmprj").join("geofabrik-index-v1.json"))
        .ok_or(OsmprjError::NoCacheDir)
}

pub async fn load_index() -> Result<Vec<GeofabrikFeature>, OsmprjError> {
    let path = cache_path()?;

    if path.exists() {
        let content = fs::read_to_string(&path)?;
        let index: GeofabrikIndex = serde_json::from_str(&content)
            .map_err(|e| OsmprjError::GeofabrikFetchFailed { message: e.to_string() })?;
        return Ok(index.features);
    }

    eprintln!("Fetching Geofabrik index...");
    let response = reqwest::get(INDEX_URL).await
        .map_err(|e| OsmprjError::GeofabrikFetchFailed { message: e.to_string() })?;

    if !response.status().is_success() {
        return Err(OsmprjError::GeofabrikFetchFailed {
            message: format!("HTTP {}", response.status()),
        });
    }

    let body = response
        .text()
        .await
        .map_err(|e| OsmprjError::GeofabrikFetchFailed { message: e.to_string() })?;

    if let Some(parent) = path.parent() {
        if !parent.is_dir() {
            let _ = fs::remove_file(parent);
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(&path, &body)?;

    let index: GeofabrikIndex = serde_json::from_str(&body)
        .map_err(|e| OsmprjError::GeofabrikFetchFailed { message: e.to_string() })?;
    Ok(index.features)
}

pub fn lookup<'a>(id: &str, features: &'a [GeofabrikFeature]) -> Option<&'a GeofabrikFeature> {
    features.iter().find(|f| f.properties.id == id)
}
