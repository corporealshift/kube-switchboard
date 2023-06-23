pub mod namespaces;
pub mod pods;
pub mod services;
use eframe::egui::Color32;
use kube::Error;
use std::fmt;

#[derive(PartialEq, Clone, Debug)]
pub enum KubeStatus {
    Loading,
    Good,
    Bad(String),
    Suspicious(String),
}

impl fmt::Display for KubeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KubeStatus::Loading => write!(f, "Loading..."),
            KubeStatus::Good => write!(f, "All good here"),
            KubeStatus::Bad(msg) => write!(f, "Bad: {}", msg),
            KubeStatus::Suspicious(msg) => write!(f, "May have a problem: {}", msg),
        }
    }
}

pub struct ActionResult {
    pub name: String,
    pub results: String,
}

#[derive(PartialEq, Clone)]
pub struct KubeResource {
    pub status: KubeStatus,
    pub name: String,
    pub display: String,
}

impl KubeResource {
    pub fn new(name: String, display: String) -> Self {
        Self {
            status: KubeStatus::Loading,
            name,
            display,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.status != KubeStatus::Loading
    }

    pub fn color(&self) -> Color32 {
        match self.status {
            KubeStatus::Loading => Color32::DARK_GRAY,
            KubeStatus::Good => Color32::GREEN,
            KubeStatus::Bad(_) => Color32::RED,
            KubeStatus::Suspicious(_) => Color32::YELLOW,
        }
    }
}

impl fmt::Display for KubeResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.display, self.status)
    }
}

pub enum KubeMessage {
    Namespaces(Result<Vec<String>, Error>),
    Resource(Result<KubeResource, Error>),
    Action(Result<ActionResult, Error>),
}
