use crate::Board;
use eframe::egui;
use eframe::egui::InnerResponse;

pub fn topbar(
    ui: &mut egui::Ui,
    namespace: &mut String,
    ready: bool,
    namespaces: Vec<String>,
    board: &mut Board,
) -> InnerResponse<()> {
    ui.heading("Fulcrum Dev Switchboard");
    ui.horizontal(|ui| {
        let ns_label = ui.label("Namespace: ");
        ui.text_edit_singleline(namespace).labelled_by(ns_label.id)
    });
    ui.horizontal(|ui| {
        if !ready {
            ui.add(egui::widgets::Spinner::new());
        }
        ui.label("Namespaces:");
        eframe::egui::ComboBox::new("namespaces", "")
            .width(200.0)
            .selected_text(namespace.clone())
            .show_ui(ui, |ui| {
                for ns in namespaces.into_iter() {
                    let ns_label = ns.clone();
                    ui.selectable_value(namespace, ns, ns_label);
                }
            })
    });
    ui.horizontal(|ui| {
        ui.label("Select a board:");
        ui.selectable_value(board, Board::Welcome, "Dashboard");
        ui.selectable_value(board, Board::Skaffold, "Kubernetes");
    })
}
