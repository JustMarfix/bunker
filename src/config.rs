use serde::{Serialize, Deserialize};
use std::fs;
use aho_corasick::AhoCorasick;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum Modes {
    UrlSubstr,
    BodySubstr,
    HeaderExists,
    HeaderSubstr
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub services: Vec<Service>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Service {
    pub listen: String,
    pub upstream: String,
    pub proto: String,
    pub filters: Vec<Filter>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Filter {
    pub mode: Modes,
    pub regex: bool,
    pub substr: String,
    #[serde(skip_serializing, skip_deserializing, default = "dummy_corasick")]
    pub aho_corasick: AhoCorasick
}

fn dummy_corasick() -> AhoCorasick {
    AhoCorasick::new(["dummy"]).unwrap()
}

pub fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let config_raw = fs::read_to_string(path)?;
    let config: Config = serde_yml::from_str(&config_raw)?;
    Ok(config)
}
