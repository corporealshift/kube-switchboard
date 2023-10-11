use crate::kube_res::actions::run_action;
use crate::KubeMessage;
use eframe::egui;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::mpsc::Sender;

#[derive(Deserialize, Clone)]
pub struct Link {
    name: String,
    url: String,
}

#[derive(Deserialize, Clone)]
pub struct Action {
    pub name: String,
    pub resource: String,
    pub action: String,
}

enum ActionStatus {
    Available,
    Running,
    Success,
    Failed,
}

struct ActionState {
    status: ActionStatus,
    result: String,
}

pub struct Board {
    pub namespace: String,
    sender: Sender<KubeMessage>,
    action_results: HashMap<String, ActionState>,
}

impl Board {
    pub fn new(sender: Sender<KubeMessage>) -> Board {
        Board {
            namespace: "".to_owned(),
            sender,
            action_results: HashMap::new(),
        }
    }
    pub fn board(&mut self, ui: &mut egui::Ui, links: Vec<Link>, actions: Vec<Action>) {
        if links.len() < 1 && actions.len() < 1 {
            ui.label("Welcome to the Dev Switchboard! Pick a board from the buttons above");
        } else {
            ui.heading("Links");
            links.iter().for_each(|link| {
                let label = format!("{}", link.name);
                let url = link.url.replace("{namespace}", self.namespace.as_str());
                ui.hyperlink_to(label, url.clone());
            });
            ui.separator();
            ui.heading("Actions");
            actions.iter().for_each(|action| {
                let label = format!("{}", action.name);
                if ui.button(label).clicked() {
                    println!(
                        "Taking an action! {}, for res: {}",
                        action.action, action.resource
                    );
                    run_action(self.namespace.clone(), action.clone(), self.sender.clone());
                    self.action_results.insert(
                        action.name.clone(),
                        ActionState {
                            status: ActionStatus::Running,
                            result: "".to_owned(),
                        },
                    );
                }
                let res = self.action_results.get(&action.name);
                match res {
                    Some(r) => match r.status {
                        ActionStatus::Success => {
                            egui::TextEdit::multiline(&mut r.result.clone().as_str())
                                .font(egui::TextStyle::Monospace)
                                .show(ui);
                        }
                        ActionStatus::Running => {
                            ui.add(egui::widgets::Spinner::new());
                        }
                        _ => {
                            ui.label(format!(
                                "Action {} failed! Result: {}",
                                action.name, r.result
                            ));
                        }
                    },
                    _ => {}
                }
            });
        }
    }

    pub fn receive_action_result(&mut self, action_name: String, result: String) {
        self.action_results.insert(
            action_name,
            ActionState {
                status: ActionStatus::Success,
                result,
            },
        );
    }
}
