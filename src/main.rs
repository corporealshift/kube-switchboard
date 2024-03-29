mod kube_res;
mod ui;

mod config;

use self::config::Config;
use self::kube_res::{namespaces::get_namespaces, KubeMessage, KubeResource, KubeStatus};

use self::ui::boards::{status, welcome};
use self::ui::topbar::Topbar;

use eframe::egui;
use eframe::CreationContext;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use tokio::runtime::Runtime;

#[derive(PartialEq)]
pub enum Board {
    Welcome,
    Status,
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

    let conf: Config = config::load().expect("Unable to load config file");

    eframe::run_native(
        "Kube Switchboard",
        options,
        Box::new(|cc| {
            Box::new(DevSwitchboard::new(
                cc,
                vec!["Loading namespaces...".into()],
                conf,
            ))
        }),
    )
}

struct DevSwitchboard {
    conf: Config,
    receiver: Receiver<KubeMessage>,
    selected_namespace: String,
    topbar: Topbar,
    namespaces: Vec<String>,
    status_board: status::Board,
    welcome_board: welcome::Board,
    board: Board,
}

impl DevSwitchboard {
    fn new(_cc: &CreationContext<'_>, namespaces: Vec<String>, conf: Config) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        get_namespaces(sender.clone());
        Self {
            conf,
            receiver,
            selected_namespace: "".to_owned(),
            topbar: Topbar::new(namespaces.clone(), sender.clone()),
            namespaces,
            status_board: status::Board::new(sender.clone()),
            welcome_board: welcome::Board::new(sender.clone()),
            board: Board::Welcome,
        }
    }
}

impl eframe::App for DevSwitchboard {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.receiver.try_recv() {
            Ok(message) => match message {
                KubeMessage::Namespaces(res) => match res {
                    Ok(namespaces) => {
                        self.namespaces = namespaces.clone();
                        self.topbar.receive_namespaces(namespaces);
                        self.selected_namespace = "".to_owned();
                    }
                    _ => self.selected_namespace = "Failed - Check login!".to_owned(),
                },
                KubeMessage::Resource(res) => match res {
                    Ok(new_resource) => {
                        self.status_board.receive_resource(new_resource);
                    }
                    _ => {}
                },
                KubeMessage::Action(res) => match res {
                    Ok(action_res) => {
                        self.welcome_board
                            .receive_action_result(action_res.name, action_res.results);
                    }
                    _ => {}
                },
            },
            _ => {} // don't care if message does not receive
        }

        self.status_board.namespace = self.selected_namespace.clone();
        self.welcome_board.namespace = self.selected_namespace.clone();

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            self.topbar
                .display(ui, &mut self.selected_namespace, &mut self.board)
        });
        egui::CentralPanel::default().show(ctx, |ui| match self.board {
            Board::Welcome => self
                .welcome_board
                .board(ui, self.conf.links(), self.conf.actions()),
            Board::Status => {
                self.status_board
                    .board(ui, self.conf.kube_services(), self.conf.kube_deployments())
            }
        });
    }
}
