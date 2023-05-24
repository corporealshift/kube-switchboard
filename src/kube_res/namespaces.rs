use crate::KubeMessage;
use k8s_openapi::api::core::v1::Namespace;
use kube::{
    api::{Api, ListParams},
    Client,
};
use std::sync::mpsc::Sender;

pub fn get_namespaces(tx: Sender<KubeMessage>) {
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
