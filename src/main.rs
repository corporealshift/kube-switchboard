use eframe::egui;
use eframe::CreationContext;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use k8s_openapi::api::core::v1::Namespace;
use kube::{
    api::{Api, ListParams},
    Client, Error,
};

use tokio::runtime::Runtime;

enum MessageType {
    Namespaces,
}

struct KubeResponse {
    name: MessageType,
    result: Result<Vec<String>, Error>,
}

impl KubeResponse {
    fn namespaces(res: Result<Vec<String>, Error>) -> KubeResponse {
        KubeResponse {
            name: MessageType::Namespaces,
            result: res,
        }
    }
}

fn get_namespaces(tx: Sender<KubeResponse>, ctx: egui::Context) {
    tokio::spawn(async move {
        let client = Client::try_default().await.expect("Failed to get client");

        let namespaces: Api<Namespace> = Api::all(client);
        let all = namespaces.list(&ListParams::default()).await.map(|list| {
            list.iter()
                .map(|ns| ns.metadata.name.clone().unwrap_or("".to_owned()))
                .collect::<Vec<String>>()
        });

        let _ = tx.send(KubeResponse::namespaces(all));
        ctx.request_repaint();
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
    sender: Sender<KubeResponse>,
    receiver: Receiver<KubeResponse>,
    selected_namespace: String,
    namespaces: Vec<String>,
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
        }
    }
}

impl eframe::App for DevSwitchboard {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(ns_res) = self.receiver.try_recv() {
            match ns_res.result {
                Ok(namespaces) => {
                    self.namespaces = namespaces;
                    self.selected_namespace = "".to_owned();
                }
                _ => self.selected_namespace = "Failed to load namespaces".to_owned(),
            };
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Fulcrum Dev Switchboard");
            ui.horizontal(|ui| {
                let ns_label = ui.label("Namespace: ");
                ui.text_edit_singleline(&mut self.selected_namespace)
                    .labelled_by(ns_label.id)
            });
            ui.horizontal(|ui| {
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
            })
        });
    }
}
