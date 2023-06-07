use figment::{
    providers::{Env, Format, Toml},
    Error, Figment,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Expected {
    services: Vec<String>,
    deployments: Vec<String>,
}

#[derive(Deserialize)]
struct Kubernetes {
    expected: Expected,
}

#[derive(Deserialize)]
pub struct Config {
    kubernetes: Kubernetes,
}

impl Config {
    pub fn kube_services(&self) -> Vec<String> {
        self.kubernetes.expected.services.clone()
    }

    pub fn kube_deployments(&self) -> Vec<String> {
        self.kubernetes.expected.deployments.clone()
    }
}

pub fn load() -> Result<Config, Error> {
    Figment::new()
        .merge(Toml::file("Config.toml"))
        .merge(Env::prefixed("DEVSWB_"))
        .extract()
}
