use std::fmt::{Debug};
use serde_json::{json};
use crate::error::GoogleApiError;


#[derive(Default, Debug)]
pub(crate) struct HttpClient {}


impl HttpClient {
    pub async fn get<T>(token: &str, url: &str) -> Result<T, GoogleApiError>
        where
            T: for<'de> serde::Deserialize<'de>,
    {
        let mut response = reqwest::Client::new()
            .get(format!("{}", url))
            .header("Authorization", format!("Bearer {}", token));

        response = response.header("Accept", "application/json");

        let response = response.send().await;


        if response.is_err() {
            return Err(GoogleApiError::Connection(response.err().unwrap().to_string()));
        }
        let response = response.unwrap();
        let status = response.status();
        let value = response.text().await;
        if !(200 <= status.as_u16() && status.as_u16() < 300) {
            return Err(GoogleApiError::JsonParse(value.unwrap().to_string()));
        }
        if value.is_err() {
            return Err(GoogleApiError::JsonParse(value.unwrap().to_string()));
        }
        let value = value.unwrap();
        let parse = serde_json::from_str(value.as_str());
        if parse.is_err() {
            return Err(GoogleApiError::JsonParse(value));
        }

        Ok(parse.unwrap())
    }
    pub async fn post<T, U>(token: &str, url: &str, params: U) -> Result<T, GoogleApiError>
        where
            T: for<'de> serde::Deserialize<'de>,
            U: serde::Serialize + std::fmt::Debug
    {
        let mut response = reqwest::Client::new()
            .post(format!("{}", url));
        if !token.is_empty() {
            response = response.header("Authorization", format!("Bearer {}", token))
        }
        let response = response
            .json(&json!(params))
            .send()
            .await;


        if response.is_err() {
            return Err(GoogleApiError::Connection(response.err().unwrap().to_string()));
        }
        let response = response.unwrap();
        let status = response.status();
        let value = response.text().await;
        if status != 200 {
            return Err(GoogleApiError::JsonParse(value.unwrap().to_string()));
        }
        if value.is_err() {
            return Err(GoogleApiError::JsonParse(value.unwrap().to_string()));
        }
        let value = value.unwrap();
        let parse = serde_json::from_str(value.as_str());
        if parse.is_err() {
            return Err(GoogleApiError::JsonParse(value));
        }

        Ok(parse.unwrap())
    }
}