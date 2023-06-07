use crate::{KubeMessage, KubeResource, KubeStatus};
use kube::{
    api::{Api, ListParams, ObjectList},
    {Client, Error},
};
use std::sync::mpsc::Sender;

pub fn check_services(expected_services: Vec<String>, tx: Sender<KubeMessage>) {
    tokio::spawn(async move {
        let msg = match Client::try_default().await {
            Ok(client) => success(KubeStatus::Good),
            Err(err) => error(err),
        };

        match tx.send(msg) {
            Ok(_) => {}
            Err(e) => println!("Failed sending message about services: {}", e),
        }
    });
}

fn success(status: KubeStatus) -> KubeMessage {
    KubeMessage::Resource(Ok(KubeResource {
        name: "service".to_owned(),
        display: "Services".to_owned(),
        status,
    }))
}

fn error(err: Error) -> KubeMessage {
    KubeMessage::Resource(Err(err))
}
