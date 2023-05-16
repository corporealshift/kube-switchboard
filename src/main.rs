mod kube_res;

use self::kube_res::{KubeMessage, KubeResource, KubeStatus};

use eframe::egui;
use eframe::egui::Color32;
use eframe::CreationContext;
use std::fmt;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use k8s_openapi::api::core::v1::{Namespace, Pod};
use kube::{
    api::{Api, ListParams},
    Client, Error,
};

use tokio::runtime::Runtime;

#[derive(PartialEq)]
enum Board {
    Welcome,
    Skaffold,
}

fn get_namespaces(tx: Sender<KubeMessage>) {
    tokio::spawn(async move {
        match Client::try_default().await {
            Ok(client) => {
                let namespaces: Api<Namespace> = Api::all(client);
                let all = namespaces.list(&ListParams::default()).await.map(|list| {
                    list.iter()
                        .map(|ns| ns.metadata.name.clone().unwrap_or("".to_owned()))
                        .collect::<Vec<String>>()
                });

                let _ = tx.send(KubeMessage::Namespaces(all));
            }
            Err(err) => {
                let _ = tx.send(KubeMessage::Namespaces(Err(err)));
            }
        }
    });
}

fn check_pods(namespace: String, tx: Sender<KubeMessage>) {
    tokio::spawn(async move {
        match Client::try_default().await {
            Ok(client) => {
                let pods: Api<Pod> = Api::namespaced(client, namespace.as_str());

                let any_bad = pods.list(&ListParams::default()).await.map(|list| {
                    println!("got pods list!");
                    list.iter()
                        .map(|pod| {
                            let phase = pod
                                .status
                                .clone()
                                .map(|s| s.phase.unwrap_or("unknown".to_owned()));
                            //println!("Pod Status: {:#?}", phase.clone().unwrap_or_default());
                            phase
                        })
                        .filter(|phase| {
                            phase != &Some("Running".to_owned())
                                && phase != &Some("Succeeded".to_owned())
                        })
                        .count()
                });
                match any_bad {
                    Ok(count) => {
                        if count > 0 {
                            let _ = tx.send(KubeMessage::Resource(Ok(KubeResource {
                                name: "pod".to_owned(),
                                display: "Pods".to_owned(),
                                status: KubeStatus::Bad("One or more not ready".to_owned()),
                            })));
                        } else {
                            let _ = tx.send(KubeMessage::Resource(Ok(KubeResource {
                                name: "pod".to_owned(),
                                display: "Pods".to_owned(),
                                status: KubeStatus::Good,
                            })));
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(KubeMessage::Resource(Err(err)));
                    }
                }
            }
            Err(err) => {
                let _ = tx.send(KubeMessage::Resource(Err(err)));
            }
        }
    });
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };

    let rt = Runtime::new().expect("Unable to create tokio runtime");
    let _enter = rt.enter();
    // Keep the runtime alive
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        })
    });

    eframe::run_native(
        "Dev Switchboard",
        options,
        Box::new(|cc| {
            Box::new(DevSwitchboard::new(
                cc,
                vec!["Loading namespaces...".to_owned()],
            ))
        }),
    )
}

struct DevSwitchboard {
    sender: Sender<KubeMessage>,
    receiver: Receiver<KubeMessage>,
    selected_namespace: String,
    namespaces: Vec<String>,
    resources: Vec<KubeResource>,
    board: Board,
    ready: bool,
}

impl DevSwitchboard {
    fn new(_cc: &CreationContext<'_>, namespaces: Vec<String>) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        get_namespaces(sender.clone());
        Self {
            sender,
            receiver,
            selected_namespace: "Loading namespaces...".to_owned(),
            namespaces,
            resources: vec![],
            board: Board::Welcome,
            ready: false,
        }
    }
}

impl eframe::App for DevSwitchboard {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.receiver.try_recv() {
            Ok(message) => match message {
                KubeMessage::Namespaces(res) => match res {
                    Ok(namespaces) => {
                        self.namespaces = namespaces;
                        self.selected_namespace = "".to_owned();
                        self.ready = true;
                    }
                    _ => self.selected_namespace = "Failed - Check login!".to_owned(),
                },
                KubeMessage::Resource(res) => match res {
                    Ok(new_resource) => {
                        self.resources = self
                            .resources
                            .iter()
                            .map(|r| {
                                if r.name == new_resource.name {
                                    new_resource.clone()
                                } else {
                                    r.clone()
                                }
                            })
                            .collect();
                    }
                    _ => {}
                },
            },
            _ => {} // don't care if message does not receive
        }
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.heading("Fulcrum Dev Switchboard");
            ui.horizontal(|ui| {
                let ns_label = ui.label("Namespace: ");
                ui.text_edit_singleline(&mut self.selected_namespace)
                    .labelled_by(ns_label.id)
            });
            ui.horizontal(|ui| {
                if !self.ready {
                    ui.add(egui::widgets::Spinner::new());
                }
                ui.label("Namespaces:");
                eframe::egui::ComboBox::new("namespaces", "")
                    .width(200.0)
                    .selected_text(self.selected_namespace.clone())
                    .show_ui(ui, |ui| {
                        for ns in self.namespaces.clone().into_iter() {
                            let ns_label = ns.clone();
                            ui.selectable_value(&mut self.selected_namespace, ns, ns_label);
                        }
                    })
            });
            ui.horizontal(|ui| {
                ui.label("Select a board:");
                ui.selectable_value(&mut self.board, Board::Welcome, "Dashboard");
                ui.selectable_value(&mut self.board, Board::Skaffold, "Skaffold");
            })
        });
        egui::CentralPanel::default().show(ctx, |ui| match self.board {
            Board::Welcome => {
                ui.label("Welcome to the Dev switchboard! Pick a board from the buttons above");
            }
            Board::Skaffold => {
                ui.heading("Skaffold helper tools");
                if self.ready && ui.button("Check Status").clicked() {
                    self.resources = vec![
                        KubeResource::new("service".to_owned(), "Services".to_owned()),
                        KubeResource::new("deployment".to_owned(), "Deploys".to_owned()),
                        KubeResource::new("pod".to_owned(), "Pods".to_owned()),
                    ];
                    check_pods(self.selected_namespace.clone(), self.sender.clone());
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
        });
    }
}
