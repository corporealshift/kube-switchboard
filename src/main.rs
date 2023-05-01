use eframe::egui;
use eframe::CreationContext;
use std::process::Command;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };

    //    let raw_namespaces = Command::new("kubectl get ns --sort-by=-.metadata.creationTimestamp -o jsonpath='{.items[*].metadata.name}'")
    let raw_namespaces = Command::new("kubectl")
        .args([
            "get",
            "ns",
            "--sort-by=.metadata.creationTimestamp",
            "-o",
            "jsonpath='{.items[*].metadata.name}'",
        ])
        .output()
        .expect("Failed to get namespaces");

    let ns_string = String::from_utf8(raw_namespaces.stdout)
        .expect("Could not parse namespaces")
        .to_string();
    println!("Res: {:?}", ns_string);
    let namespaces = ns_string
        .split_whitespace()
        .into_iter()
        .map(|ns| ns.to_owned())
        .collect::<Vec<_>>();

    eframe::run_native(
        "Dev Switchboard",
        options,
        Box::new(|cc| Box::new(DevSwitchboard::new(cc, namespaces))),
    )
}

struct DevSwitchboard {
    selected_namespace: String,
    namespaces: Vec<String>,
}

impl DevSwitchboard {
    fn new(_cc: &CreationContext<'_>, namespaces: Vec<String>) -> Self {
        Self {
            selected_namespace: "".to_owned(),
            namespaces,
        }
    }
}

impl eframe::App for DevSwitchboard {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
