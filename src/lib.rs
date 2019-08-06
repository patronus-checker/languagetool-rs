extern crate reqwest;
#[macro_use]
extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Deserializer};
use serde::de::Error as DeError;
use serde_json::Value;
use std::fmt;
use std::error::Error as StdError;

pub struct LanguageTool {
    instance_url: String,
    http_client: reqwest::Client,
}

#[derive(Debug)]
pub enum Error {
    ReqwestError(reqwest::Error),
    BadStatusError(reqwest::StatusCode),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ReqwestError(ref e) => fmt::Display::fmt(e, f),
            Error::BadStatusError(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ReqwestError(ref e) => e.description(),
            Error::BadStatusError(ref e) => {
                e.canonical_reason().unwrap_or("Unregistered status code")
            }
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::ReqwestError(ref e) => Some(e),
            Error::BadStatusError(_) => None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Language {
    pub name: String,
    pub code: String,
    pub long_code: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub text: String,
    pub language: String,
    pub mother_tongue: Option<String>,
    pub preferred_variants: Option<String>,
    pub enabled_rules: Option<String>,
    pub disabled_rules: Option<String>,
    pub enabled_categories: Option<String>,
    pub disabled_categories: Option<String>,
    pub enabled_only: Option<bool>,
}

impl Request {
    pub fn new(text: String, language: String) -> Self {
        Request {
            text: text,
            language: language,
            mother_tongue: None,
            preferred_variants: None,
            enabled_rules: None,
            disabled_rules: None,
            enabled_categories: None,
            disabled_categories: None,
            enabled_only: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub software: Option<Software>,
    pub language: Option<ResponseLanguage>,
    pub matches: Option<Vec<Match>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Software {
    pub name: String,
    pub version: String,
    pub build_date: String,
    // In older versions the version is a String:
    // https://github.com/languagetool-org/languagetool/issues/712
    #[serde(deserialize_with = "number_or_numeric_string")]
    pub api_version: i64,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResponseLanguage {
    pub name: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Match {
    pub message: String,
    pub short_message: Option<String>,
    pub offset: i64,
    pub length: i64,
    pub replacements: Vec<Replacement>,
    pub context: Context,
    pub rule: Option<Rule>,
}

#[derive(Debug, Deserialize)]
pub struct Replacement {
    pub value: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Context {
    pub text: String,
    pub offset: i64,
    pub length: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub sub_id: Option<String>,
    pub description: String,
    pub urls: Option<Vec<Url>>,
    pub issue_type: Option<String>,
    pub category: Category,
}

#[derive(Debug, Deserialize)]
pub struct Url {
    pub value: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Category {
    pub id: Option<String>,
    pub name: Option<String>,
}

impl LanguageTool {
    pub fn new(instance_url: &str) -> Result<Self, Error> {
        let instance_url = String::from(instance_url.trim_end_matches('/'));
        let http_client = reqwest::Client::builder()
            .build()
            .map_err(Error::ReqwestError)?;

        Ok(LanguageTool {
            instance_url: instance_url,
            http_client: http_client,
        })
    }

    pub fn list_languages(&self) -> Result<Vec<Language>, Error> {
        let mut res = self.http_client
            .get(&(self.instance_url.clone() + "/v2/languages"))
            .send()
            .map_err(Error::ReqwestError)?;

        if res.status().is_success() {
            res.json().map_err(Error::ReqwestError)
        } else {
            Err(Error::BadStatusError(res.status()))
        }
    }

    pub fn check(&self, req: Request) -> Result<Response, Error> {
        let mut res = self.http_client
            .post(&(self.instance_url.clone() + "/v2/check"))
            .form(&req)
            .send()
            .map_err(Error::ReqwestError)?;

        if res.status().is_success() {
            res.json().map_err(Error::ReqwestError)
        } else {
            Err(Error::BadStatusError(res.status()))
        }
    }
}

fn number_or_numeric_string<'de, D>(de: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let helper: Value = Deserialize::deserialize(de)?;

    match helper {
        Value::Number(n) => n.as_i64().ok_or_else(|| DeError::custom("Not an integer")),
        Value::String(s) => s.parse().map_err(DeError::custom),
        _ => Err(DeError::custom("Neither number nor a numeric string")),
    }
}
