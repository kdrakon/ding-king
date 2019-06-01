use crate::big_five_results_text_serializer::BigFiveResults;
use crate::big_five_results_text_serializer::BigFiveResultsTextToHash;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Response, StatusCode};
use serde_json::json;

mod big_five_results_text_serializer;

struct BigFiveResultsPoster<'a> {
    email: &'a str,
    big_five_results_hash: serde_json::Value,
}

enum BigFiveResultsPosterError<'a> {
    PostError(&'a str),
    PostResponseError(u16, String),
    DataError(&'a str),
}

impl BigFiveResultsPoster<'_> {
    pub fn post<'a>(&'a self, post_url: &'a str) -> Result<(u16, String), BigFiveResultsPosterError<'a>> {
        let mut big_five_results_hash = self.big_five_results_hash.clone();
        match big_five_results_hash.as_object_mut() {
            Some(json_map) => {
                json_map.insert(String::from("EMAIL"), json!(self.email));
                let json_body = serde_json::to_string(json_map).map_err(|_| BigFiveResultsPosterError::DataError("Error serializing JSON"))?;

                let client = reqwest::Client::new();
                let mut response = client
                    .post(post_url)
                    .header(CONTENT_TYPE, "application/json")
                    .body(json_body)
                    .send()
                    .map_err(|err| BigFiveResultsPosterError::PostError("Failed to perform POST"))?;

                match response.status() {
                    StatusCode::CREATED => Ok((201, response.text().unwrap_or(String::new()))),
                    other_code => Err(BigFiveResultsPosterError::PostResponseError(other_code.as_u16(), response.text().unwrap_or(String::new()))),
                }
            }
            None => Err(BigFiveResultsPosterError::DataError("Failed to parse result hash as JSON object")),
        }
    }
}

fn main() {}
