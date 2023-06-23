use eframe::egui;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Link {
    name: String,
    url: String,
}

#[derive(Deserialize, Clone)]
pub struct Action {
    name: String,
    resource: String,
    action: String,
}

pub fn board(ui: &mut egui::Ui, namespace: String, links: Vec<Link>, actions: Vec<Action>) {
    if links.len() < 1 && actions.len() < 1 {
        ui.label("Welcome to the Dev Switchboard! Pick a board from the buttons above");
    } else {
        ui.heading("Links");
        links.iter().for_each(|link| {
            let label = format!("{}", link.name);
            let url = link.url.replace("{namespace}", namespace.as_str());
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
            }
        });
    }
}
