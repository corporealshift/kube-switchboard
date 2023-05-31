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
                let pods_request: Api<Pod> = Api::namespaced(client, namespace.as_str());
                let all_pods = pods_request.list(&ListParams::default()).await;
                let pods = all_pods.map(|list| {
                    let all_pods_vec: Vec<Pod> = list.iter().map(|p| p.clone()).collect();
                    println!("got pods list! ns: {}", namespace);
                    (
                        all_pods_vec,
                        only_bad_pods(&list).collect::<Vec<Option<Pod>>>(),
                    )
                });
                match pods {
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
    use super::*;
    use k8s_openapi::api::core::v1::{Pod, PodStatus};
    use kube::api::ObjectList;

    #[test]
    pub fn good_pods_are_ignored() {
        let pods = ObjectList {
            metadata: Default::default(),
            items: vec![
                pod(Some("Running".to_owned())),
                pod(Some("Succeeded".to_owned())),
                pod(Some("Pending".to_owned())),
                pod(Some("Failed".to_owned())),
            ],
        };

        assert_eq!(only_bad_pods(&pods).count(), 2);
    }

    #[test]
    fn pods_with_no_status_are_bad() {
        let pods = ObjectList {
            metadata: Default::default(),
            items: vec![pod(Some("Running".to_owned())), pod(None)],
        };
        assert_eq!(only_bad_pods(&pods).count(), 1);
    }

    fn pod(phase: Option<String>) -> Pod {
        Pod {
            metadata: Default::default(),
            spec: None,
            status: Some(PodStatus {
                conditions: None,
                container_statuses: None,
                ephemeral_container_statuses: None,
                host_ip: None,
                init_container_statuses: None,
                message: None,
                nominated_node_name: None,
                phase,
                pod_ip: None,
                pod_ips: None,
                qos_class: None,
                reason: None,
                start_time: None,
            }),
        }
    }
}
