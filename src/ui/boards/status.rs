use crate::kube_res::pods::check_pods;
use crate::{KubeMessage, KubeResource};
use eframe::egui;
use std::sync::mpsc::Sender;

pub struct Board {
    resources: Vec<KubeResource>,
    sender: Sender<KubeMessage>,
    pub namespace: String,
}

impl Board {
    pub fn new(sender: Sender<KubeMessage>) -> Board {
        Board {
            resources: vec![],
            sender,
            namespace: "".to_owned(),
        }
    }
    fn check(&mut self) {
        self.resources = vec![
            KubeResource::new("service".to_string(), "Services".to_string()),
            KubeResource::new("deployment".to_owned(), "Deploys".to_string()),
            KubeResource::new("pod".to_string(), "Pods".to_string()),
        ];
        check_pods(self.namespace.clone(), self.sender.clone());
    }
    pub fn board(&mut self, ui: &mut egui::Ui, ready: bool) {
        ui.heading("Status of various resources");
        if ready && ui.button("Check Status").clicked() {
            self.check();
        }
        for resource in self.resources.clone() {
            ui.horizontal(|ui| {
                if !resource.is_ready() {
                    ui.add(egui::widgets::Spinner::new());
                }
                ui.colored_label(resource.color(), format!("{}", resource));
            });
        }
    }
    pub fn receive_resource(&mut self, resource: KubeResource) {
        self.resources = self
            .resources
            .iter()
            .map(|r| {
                if r.name == resource.name {
                    resource.clone()
                } else {
                    r.clone()
                }
            })
            .collect();
    }
}
