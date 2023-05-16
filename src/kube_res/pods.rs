use crate::{KubeMessage, KubeResource, KubeStatus};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams},
    {Client, Error},
};
use std::sync::mpsc::Sender;

pub fn check_pods(namespace: String, tx: Sender<KubeMessage>) {
    tokio::spawn(async move {
        match Client::try_default().await {
            Ok(client) => {
                let pods: Api<Pod> = Api::namespaced(client, namespace.as_str());
                let any_bad = pods.list(&ListParams::default()).await.map(|list| {
                    println!("got pods list!");
                    list.iter()
                        .map(|pod| {
                            pod.status
                                .clone()
                                .map(|s| s.phase.unwrap_or("unknown".to_owned()))
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
                            let _ = tx
                                .send(success(KubeStatus::Bad("One or more not ready".to_owned())));
                        } else {
                            let _ = tx.send(success(KubeStatus::Good));
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(error(err));
                    }
                }
            }
            Err(err) => {
                let _ = tx.send(error(err));
            }
        }
    });
}

fn success(status: KubeStatus) -> KubeMessage {
    KubeMessage::Resource(Ok(KubeResource {
        name: "pod".to_owned(),
        display: "Pods".to_owned(),
        status,
    }))
}

fn error(err: Error) -> KubeMessage {
    KubeMessage::Resource(Err(err))
}
