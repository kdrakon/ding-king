use std::collections::HashMap;

use regex::{Captures, Regex};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use serde_json::json;
use serde_json::{Map, Value};

#[derive(Debug)]
pub enum BigFiveError<'a> {
    SimpleError(&'a str),
    InputError(&'a str),
}

pub struct BigFiveResults<'a> {
    name: &'a str,
    results: Vec<BigFiveResult<'a>>,
}

#[derive(Debug)]
pub struct BigFiveResult<'a> {
    name: &'a str,
    overall_score: u8,
    facets: HashMap<&'a str, u8>,
}

const RESULT_HEADER_REGEX: &str = r"([A-Z\- ]+?\.+\d+\n)";
const SCORE_REGEX: &str = r"([\w\- ]+)\.+(\d+)";

pub fn new<'a>(name: &'a str, input: &'a str) -> Result<BigFiveResults<'a>, BigFiveError<'a>> {
    let header_regex = Regex::new(RESULT_HEADER_REGEX).map_err(|_| BigFiveError::SimpleError("Could not compile Regex"))?;

    let headers = header_regex.captures_iter(input).map(|capture| capture.get(1)).flatten().map(|m| m.as_str()).collect::<Vec<_>>();
    let mut headers_iter = headers.into_iter();

    let results = header_regex.split(input).filter(|s| !s.is_empty()).collect::<Vec<_>>();

    if headers_iter.len() != results.len() {
        Err(BigFiveError::InputError("Headers did not match number of results"))
    } else {
        let results = results
            .into_iter()
            .map(|result| parse_result(headers_iter.next().unwrap(), result))
            .collect::<Result<Vec<BigFiveResult>, BigFiveError>>()?;
        Ok(BigFiveResults { name, results })
    }
}

fn parse_result<'a>(header: &'a str, input: &'a str) -> Result<BigFiveResult<'a>, BigFiveError<'a>> {
    let facets = input.lines().collect::<Vec<_>>();
    let score_regex = Regex::new(SCORE_REGEX).map_err(|_| BigFiveError::SimpleError("Could not compile Regex"))?;

    fn extract_field_score(captures: Captures) -> Result<(&str, u8), BigFiveError> {
        let capture_vec = captures.iter().collect::<Vec<_>>();
        match capture_vec.as_slice() {
            [Some(_), Some(facet), Some(score)] => score
                .as_str()
                .parse::<u8>()
                .map_err(|_| BigFiveError::InputError("Could not parse field score"))
                .and_then(|score| Ok((facet.as_str(), score))),
            _ => Err(BigFiveError::InputError("Could not parse field")),
        }
    }

    let (name, overall_score) =
        score_regex.captures(header).ok_or(BigFiveError::InputError("Could not parse header")).and_then(extract_field_score)?;

    let facets = facets
        .into_iter()
        .map(|facet| score_regex.captures(facet).ok_or(BigFiveError::InputError("Could not parse field score")).map(extract_field_score))
        .collect::<Result<Vec<_>, BigFiveError>>()
        .map(|vec| vec.into_iter().flatten().collect::<HashMap<&str, u8>>())?;

    Ok(BigFiveResult { name, overall_score, facets })
}

impl serde::ser::Serialize for BigFiveResult<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut facet_map: Map<String, Value> = Map::new();
        for (facet, score) in &self.facets {
            facet_map.insert(facet.to_string(), json!(score));
        }

        let mut body_map: Map<String, Value> = Map::new();
        body_map.insert("Overall Score".to_string(), json!(self.overall_score));
        body_map.insert("Facets".to_string(), json!(facet_map));

        let mut root_map: Map<String, Value> = Map::new();
        root_map.insert(self.name.to_string(), json!(body_map));
        root_map.serialize(serializer)
    }
}

impl serde::ser::Serialize for BigFiveResults<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut map: Map<String, Value> = Map::new();
        map.insert("NAME".to_string(), json!(self.name));
        for result in &self.results {
            if let Ok(Value::Object(result_map)) = serde_json::to_value(result) {
                for (key, value) in result_map {
                    map.insert(key, value);
                }
            }
        }
        map.serialize(serializer)
    }
}

pub trait BigFiveResultsTextToHash {
    fn to_h(&self) -> Result<serde_json::Value, serde_json::Error>;
}

impl BigFiveResultsTextToHash for BigFiveResult<'_> {
    fn to_h(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

impl BigFiveResultsTextToHash for BigFiveResults<'_> {
    fn to_h(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::big_five_results_text_serializer;
    use crate::big_five_results_text_serializer::*;

    const TEST_INPUT_1: &str = "EXTRAVERSION.........54
Friendliness.........94
Gregariousness.......72
Assertiveness........44
Activity Level.......18
Excitement-Seeking...17
Cheerfulness.........57
";

    const TEST_INPUT_2: &str = "AGREEABLENESS...96
Trust...........70
Morality........89
Altruism........80
Cooperation.....93
Modesty.........95
Sympathy........73
";

    #[test]
    fn test_new() {
        let results = big_five_results_text_serializer::new("Sean", TEST_INPUT_1).unwrap();
        assert_eq!(results.results.len(), 1);
        let result = results.results.get(0).unwrap();
        assert_eq!(result.name, "EXTRAVERSION");
        assert_eq!(result.overall_score, 54);
        let expected_facets = [
            ("Friendliness", 94),
            ("Gregariousness", 72),
            ("Assertiveness", 44),
            ("Activity Level", 18),
            ("Excitement-Seeking", 17),
            ("Cheerfulness", 57),
        ]
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
        assert!(expected_facets.difference(&result.facets.clone().into_iter().collect::<HashSet<_>>()).collect::<Vec<_>>().is_empty());
    }

    #[test]
    fn test_multiple_results() {
        let concat_input = format!("{}{}", TEST_INPUT_1, TEST_INPUT_2);
        let results = big_five_results_text_serializer::new("Sean", &concat_input).unwrap();
        assert_eq!(results.results.len(), 2);
        let result = results.results.get(1).unwrap();
        assert_eq!(result.name, "AGREEABLENESS");
        assert_eq!(result.overall_score, 96);
        let expected_facets = [("Trust", 70), ("Morality", 89), ("Altruism", 80), ("Cooperation", 93), ("Modesty", 95), ("Sympathy", 73)]
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        assert!(expected_facets.difference(&result.facets.clone().into_iter().collect::<HashSet<_>>()).collect::<Vec<_>>().is_empty());
    }

    #[test]
    fn test_to_h() {
        let result = big_five_results_text_serializer::new("Sean", TEST_INPUT_1).unwrap();
        let json = result.to_h().and_then(|r| serde_json::to_string(&r)).unwrap();
        assert_eq!(json, "{\"EXTRAVERSION\":{\"Facets\":{\"Activity Level\":18,\"Assertiveness\":44,\"Cheerfulness\":57,\"Excitement-Seeking\":17,\"Friendliness\":94,\"Gregariousness\":72},\"Overall Score\":54},\"NAME\":\"Sean\"}");

        let concat_input = format!("{}{}", TEST_INPUT_1, TEST_INPUT_2);
        let result = big_five_results_text_serializer::new("Sean", &concat_input).unwrap();
        let json = result.to_h().and_then(|r| serde_json::to_string(&r)).unwrap();
        assert_eq!(json, "{\"AGREEABLENESS\":{\"Facets\":{\"Altruism\":80,\"Cooperation\":93,\"Modesty\":95,\"Morality\":89,\"Sympathy\":73,\"Trust\":70},\"Overall Score\":96},\"EXTRAVERSION\":{\"Facets\":{\"Activity Level\":18,\"Assertiveness\":44,\"Cheerfulness\":57,\"Excitement-Seeking\":17,\"Friendliness\":94,\"Gregariousness\":72},\"Overall Score\":54},\"NAME\":\"Sean\"}");
    }
}
