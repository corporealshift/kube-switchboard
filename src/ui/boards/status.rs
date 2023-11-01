use crate::kube_res::{pods::check_pods, services::check_services};
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
    fn check(&mut self, expected_services: Vec<String>, expected_deploys: Vec<String>) {
        self.resources = vec![
            KubeResource::new("service".to_owned(), "Services".to_owned()),
            KubeResource::new("deployment".to_owned(), "Deploys".to_owned()),
            KubeResource::new("pod".to_owned(), "Pods".to_owned()),
        ];
        check_pods(self.namespace.clone(), self.sender.clone());
        check_services(
            self.namespace.clone(),
            expected_services,
            self.sender.clone(),
        );
    }
    pub fn board(&mut self, ui: &mut egui::Ui, services: Vec<String>, deployments: Vec<String>) {
        ui.heading("Status of various resources");
        if ui.button("Check Status").clicked() {
            self.check(services, deployments);
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
