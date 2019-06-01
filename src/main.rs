use std::io;
use std::io::BufRead;

use clap::{App, Arg};
use reqwest::header::CONTENT_TYPE;
use reqwest::{Response, StatusCode};
use serde_json::json;

use crate::big_five_results_text_serializer::BigFiveResults;
use crate::big_five_results_text_serializer::BigFiveResultsTextToHash;

mod big_five_results_text_serializer;

struct BigFiveResultsPoster<'a> {
    email: &'a str,
    big_five_results_hash: serde_json::Value,
}

#[derive(Debug)]
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

fn main() {
    let arg_matches = App::new("ding-king")
        .arg(Arg::with_name("name").long("name").value_name("NAME").required(true))
        .arg(Arg::with_name("email").long("email").value_name("EMAIL").required(true))
        .arg(Arg::with_name("post_url").long("url").value_name("POST_URL").required(true))
        .get_matches();

    let stdin = io::stdin();
    let lines = stdin
        .lock()
        .lines()
        .map(|line| {
            let line = line.unwrap();
            String::from(line.trim())
        })
        .collect::<Vec<_>>();

    if !lines.is_empty() {
        let name = arg_matches.value_of("name").unwrap();
        let email = arg_matches.value_of("email").unwrap();
        let post_url = arg_matches.value_of("post_url").unwrap();
        let file = lines.join("\n");

        match big_five_results_text_serializer::new(name, &file) {
            Err(err) => eprintln!("{:?}", err),
            Ok(results) => match results.to_h() {
                Err(err) => eprintln!("{:?}", err),
                Ok(big_five_results_hash) => {
                    let request = BigFiveResultsPoster { email, big_five_results_hash };
                    match request.post(post_url) {
                        Err(err) => eprintln!("{:?}", err),
                        Ok(post_result) => println!("{:?}", post_result),
                    }
                }
            },
        }
    }
}
