use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use failure::Error;

// mirror manifests
#[derive(Serialize, Deserialize)]
pub struct Mirror {
    name: String,
    #[serde(rename = "name-tr")]
    name_tr: String,
    loc: String,
    #[serde(rename = "loc-tr")]
    loc_tr: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Tarball {
    arch: String,
    #[serde(rename = "downloadSize")]
    download_size: i64,
    #[serde(rename = "instSize")]
    inst_size: i64,
    path: String,
    sha256sum: String,
}

#[derive(Serialize, Deserialize)]
pub struct Variant {
    name: String,
    retro: bool,
    description: String,
    #[serde(rename = "description-tr")]
    description_tr: String,
    tarballs: Vec<Tarball>,
}

#[derive(Serialize, Deserialize)]
pub struct Bulletin {
    #[serde(rename = "type")]
    type_: String,
    title: String,
    #[serde(rename = "title-tr")]
    title_tr: String,
    body: String,
    #[serde(rename = "body-tr")]
    body_tr: String,
}

#[derive(Serialize, Deserialize)]
pub struct Recipe {
    version: usize,
    bulletin: Bulletin,
    variants: Vec<Variant>,
    mirrors: Vec<Mirror>,
}

// config manifest
#[derive(Serialize, Deserialize)]
pub struct UserBasicConfig {
    path: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserMirrorConfig {
    name: String,
    loc: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserVariantConfig {
    name: String,
    description: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserDistroConfig {
    mainline: HashMap<String, UserVariantConfig>,
    retro: HashMap<String, UserVariantConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct UserConfig {
    config: UserBasicConfig,
    mirrors: Vec<UserMirrorConfig>,
    distro: UserDistroConfig,
}

#[inline]
pub fn parse_config(data: &[u8]) -> Result<UserConfig, Error> {
    Ok(toml::from_slice(data)?)
}

pub fn get_root_path(config: &UserConfig) -> String {
    config.config.path.clone()
}

pub fn generate_manifest(manifest: &Recipe) -> Result<String, Error> {
    Ok(serde_json::to_string(manifest)?)
}
