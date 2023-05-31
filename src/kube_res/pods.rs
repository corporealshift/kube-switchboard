use crate::{KubeMessage, KubeResource, KubeStatus};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams, ObjectList},
    {Client, Error},
};
use std::sync::mpsc::Sender;

pub fn check_pods(namespace: String, tx: Sender<KubeMessage>) {
    tokio::spawn(async move {
        let msg = match Client::try_default().await {
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
                pods_message(pods)
            }
            Err(err) => error(err),
        };
        match tx.send(msg) {
            Ok(_) => {}
            Err(e) => println!("Failed sending message about pods: {}", e),
        };
    });
}

fn pods_message(pods: Result<(Vec<Pod>, Vec<Option<Pod>>), Error>) -> KubeMessage {
    match pods {
        Ok((all_pods, bad_pods)) => {
            if bad_pods.iter().count() > 0 {
                success(KubeStatus::Bad("One or more not ready".to_owned()))
            } else if all_pods.iter().count() < 1 {
                success(KubeStatus::Suspicious("No pods found".to_owned()))
            } else {
                success(KubeStatus::Good)
            }
        }
        Err(err) => error(err),
    }
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
mod test {
    use super::*;

    use k8s_openapi::api::core::v1::{Pod, PodStatus};
    use kube::api::ObjectList;
    use std::io::Error as IOError;
    use std::io::ErrorKind;

    #[cfg(test)]
    mod pods_message {
        use super::*;
        #[test]
        pub fn is_bad_when_any_bad_pods() {
            let pods = Ok((
                vec![pod(Some("Running".to_owned()))],
                vec![Some(pod(Some("Bad".to_owned())))],
            ));
            let msg = pods_message(pods);

            match msg {
                KubeMessage::Resource(Ok(res)) => {
                    assert_eq!(
                        res.status,
                        KubeStatus::Bad("One or more not ready".to_owned())
                    );
                }
                _ => panic!("bad pods should result in a Bad message"),
            }
        }

        #[test]
        pub fn is_sus_when_no_pods() {
            let pods = Ok((vec![], vec![]));
            let msg = pods_message(pods);

            match msg {
                KubeMessage::Resource(Ok(res)) => {
                    assert_eq!(
                        res.status,
                        KubeStatus::Suspicious("No pods found".to_owned())
                    );
                }
                _ => panic!("No pods should result in Suspicious message"),
            }
        }

        #[test]
        pub fn is_good_when_no_bad_pods() {
            let pods = Ok((vec![pod(Some("Running".to_owned()))], vec![]));
            let msg = pods_message(pods);

            match msg {
                KubeMessage::Resource(Ok(res)) => {
                    assert_eq!(res.status, KubeStatus::Good);
                }
                _ => panic!("Only good pods should result in Good message"),
            }
        }

        #[test]
        pub fn is_error_when_error() {
            let io_error = IOError::new(ErrorKind::NotFound, "borked");
            let pods = Err(Error::ReadEvents(io_error));
            let msg = pods_message(pods);

            match msg {
                KubeMessage::Resource(Err(err)) => {
                    assert_eq!(
                        err.to_string(),
                        "Error reading events stream: borked".to_owned()
                    );
                }
                _ => panic!("Error should report an error message"),
            }
        }
    }

    #[cfg(test)]
    mod only_bad_pods {
        use super::*;
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
