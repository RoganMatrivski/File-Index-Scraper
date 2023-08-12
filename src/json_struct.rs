use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonStruct {
    pub version: String,
    pub full_url: String,
    pub filepath: PathBuf,
    pub encoded_filepath: PathBuf,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonList {
    pub version: String,
    pub base_url: String,
    pub url_query: String,
    pub files: Vec<JsonFile>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonFile {
    pub filepath: PathBuf,
    pub encoded_filepath: PathBuf,
}
