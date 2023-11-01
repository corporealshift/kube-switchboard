use crate::Board;
use eframe::egui;
use eframe::egui::InnerResponse;

use crate::kube_res::{namespaces::get_namespaces, KubeMessage};
use std::sync::mpsc::Sender;
pub struct Topbar {
    namespaces: Vec<String>,
    namespaces_loaded: bool,
    sender: Sender<KubeMessage>,
}

impl Topbar {
    pub fn new(namespaces: Vec<String>, sender: Sender<KubeMessage>) -> Topbar {
        Topbar {
            namespaces,
            namespaces_loaded: false,
            sender,
        }
    }

    pub fn receive_namespaces(&mut self, namespaces: Vec<String>) {
        self.namespaces_loaded = true;
        self.namespaces = namespaces;
    }

    pub fn display(
        &mut self,
        ui: &mut egui::Ui,
        selected_namespace: &mut String,
        board: &mut Board,
    ) -> InnerResponse<()> {
        ui.heading("Kubernetes Switchboard");
        ui.horizontal(|ui| {
            let ns_label = ui.label("Namespace: ");
            ui.text_edit_singleline(selected_namespace)
                .labelled_by(ns_label.id)
        });
        ui.horizontal(|ui| {
            if !self.namespaces_loaded {
                ui.add(egui::widgets::Spinner::new());
            }
            ui.label("Namespaces:");
            eframe::egui::ComboBox::new("namespaces", "")
                .width(200.0)
                .selected_text(selected_namespace.clone())
                .show_ui(ui, |ui| {
                    for ns in self.namespaces.clone().into_iter() {
                        let ns_label = ns.clone();
                        ui.selectable_value(selected_namespace, ns, ns_label);
                    }
                });
            if ui.button("⟲").clicked() {
                self.namespaces_loaded = false;
                get_namespaces(self.sender.clone());
                selected_namespace.clear();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Select a board:");
            ui.selectable_value(board, Board::Welcome, "Dashboard");
            ui.selectable_value(board, Board::Status, "Status");
        })
    }
}
