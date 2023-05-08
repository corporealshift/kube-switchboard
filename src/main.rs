use eframe::egui;
use eframe::egui::Color32;
use eframe::CreationContext;
use std::fmt;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use k8s_openapi::api::core::v1::Namespace;
use kube::{
    api::{Api, ListParams},
    Client, Error,
};

use tokio::runtime::Runtime;

enum KubeMessage {
    Namespaces(Result<Vec<String>, Error>),
    Resource(Result<KubeResource, Error>),
}

#[derive(PartialEq)]
enum Board {
    Welcome,
    Skaffold,
}

#[derive(PartialEq, Clone)]
enum KubeStatus {
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
            KubeStatus::Bad(msg) => write!(f, "Error: {}", msg),
            KubeStatus::Suspicious(msg) => write!(f, "May have a problem: {}", msg),
        }
    }
}

#[derive(PartialEq, Clone)]
struct KubeResource {
    status: KubeStatus,
    name: String,
    display: String,
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
}

impl fmt::Display for KubeResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.display, self.status)
    }
}

fn get_namespaces(tx: Sender<KubeMessage>, ctx: egui::Context) {
    tokio::spawn(async move {
        let client = Client::try_default().await;

        match client {
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
                tx.send(KubeMessage::Namespaces(Err(err)));
            }
        }
        ctx.request_repaint();
    });
}

fn check_status(tx: Sender<KubeMessage>, ctx: egui::Context) {}

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
    fn new(cc: &CreationContext<'_>, namespaces: Vec<String>) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        get_namespaces(sender.clone(), cc.egui_ctx.clone());
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
                            .filter(|r| r.name == new_resource.name)
                            .map(|_| new_resource.clone())
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
                }
                for resource in self.resources.clone() {
                    ui.horizontal(|ui| {
                        if !resource.is_ready() {
                            ui.add(egui::widgets::Spinner::new());
                        }
                        ui.label(format!("{}", resource));
                    });
                }
            }
        });
    }
}
