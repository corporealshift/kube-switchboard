use crate::welcome::{Action, Link};
use figment::{
    providers::{Env, Format, Toml},
    Error, Figment,
};
use serde::Deserialize;
use std::env;

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
struct Switchboard {
    links: Vec<Link>,
    actions: Vec<Action>,
}

#[derive(Deserialize)]
pub struct Config {
    kubernetes: Kubernetes,
    switchboard: Switchboard,
}

impl Config {
    pub fn kube_services(&self) -> Vec<String> {
        self.kubernetes.expected.services.clone()
    }

    pub fn kube_deployments(&self) -> Vec<String> {
        self.kubernetes.expected.deployments.clone()
    }

    pub fn links(&self) -> Vec<Link> {
        self.switchboard.links.clone()
    }

    pub fn actions(&self) -> Vec<Action> {
        self.switchboard.actions.clone()
    }
}

pub fn load() -> Result<Config, Error> {
    match env::home_dir() {
        Some(path) => {
            let mut config_path = path.clone();
            config_path.push(".kube_swb/Config.toml");
            let file = Toml::file(config_path.as_path());
            Figment::new()
                .merge(file)
                .merge(Env::prefixed("KUBESWB_"))
                .extract()
        }
        None => Figment::new().merge(Env::prefixed("KUBESWB_")).extract(),
    }
}
