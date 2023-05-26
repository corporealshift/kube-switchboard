use crate::{KubeMessage, KubeResource, KubeStatus};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams, ObjectList},
    {Client, Error},
};
use std::sync::mpsc::Sender;

pub fn check_pods(namespace: String, tx: Sender<KubeMessage>) {
    tokio::spawn(async move {
        match Client::try_default().await {
            Ok(client) => {
                let pods: Api<Pod> = Api::namespaced(client, namespace.as_str());
                let all_pods = pods.list(&ListParams::default()).await;
                let bad_pods = all_pods.map(|list| {
                    let all_pods_vec: Vec<Pod> = list.iter().map(|p| p.clone()).collect();
                    println!("got pods list! ns: {}", namespace);
                    (
                        all_pods_vec,
                        only_bad_pods(&list).collect::<Vec<Option<Pod>>>(),
                    )
                });
                match bad_pods {
                    Ok((all_pods, bad_pods)) => {
                        if bad_pods.iter().count() > 0 {
                            let _ = tx
                                .send(success(KubeStatus::Bad("One or more not ready".to_owned())));
                        } else if all_pods.iter().count() < 1 {
                            let _ = tx
                                .send(success(KubeStatus::Suspicious("No pods found".to_owned())));
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

fn only_bad_pods(list: &ObjectList<Pod>) -> impl Iterator<Item = Option<Pod>> + '_ {
    list.iter()
        .map(|pod| {
            pod.status
                .clone()
                .map(|s| (pod, s.phase.unwrap_or("unknown".to_owned())))
        })
        .filter(|opt| {
            if let Some((_pod, phase)) = opt {
                phase != &"Running".to_owned() && phase != &"Succeeded".to_owned()
            } else {
                false
            }
        })
        .map(|opt| {
            if let Some((pod, _phase)) = opt {
                Some(pod.clone())
            } else {
                None
            }
        })
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

#[cfg(test)]
mod tests {
    #[test]
    fn good_pods_are_ignored() {}
}
