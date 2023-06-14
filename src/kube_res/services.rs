use crate::{KubeMessage, KubeResource, KubeStatus};
use k8s_openapi::api::core::v1::Service;
use kube::{
    api::{Api, ListParams},
    {Client, Error},
};
use std::sync::mpsc::Sender;

pub fn check_services(namespace: String, expected_services: Vec<String>, tx: Sender<KubeMessage>) {
    tokio::spawn(async move {
        let msg = match Client::try_default().await {
            Ok(client) => {
                let services_request: Api<Service> = Api::namespaced(client, namespace.as_str());
                let service_list = services_request.list(&ListParams::default()).await;
                let missing_services: Result<Vec<String>, Error> = service_list.map(|list| {
                    let svc_names: Vec<Option<String>> =
                        list.iter().map(|s| s.metadata.name.clone()).collect();
                    missing_services(expected_services, svc_names)
                });
                println!("services: {:?}", missing_services);

                match missing_services {
                    Ok(services) => {
                        if services.iter().count() > 0 {
                            success(KubeStatus::Bad(format!(
                                "Services not in k8s: {}",
                                services.join(", ")
                            )))
                        } else {
                            success(KubeStatus::Good)
                        }
                    }
                    Err(err) => error(err),
                }
            }
            Err(err) => error(err),
        };

        match tx.send(msg) {
            Ok(_) => {}
            Err(e) => println!("Failed sending message about services: {}", e),
        }
    });
}

fn missing_services(expected: Vec<String>, actual: Vec<Option<String>>) -> Vec<String> {
    expected
        .into_iter()
        .filter(|service| !actual.contains(&Some(service.clone())))
        .collect()
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

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(test)]
    mod missing_services {
        use super::*;

        #[test]
        pub fn returns_missing_services() {
            let expected = vec!["rails-fulcrum".to_owned(), "query".to_owned()];
            let actual = vec![Some("query".to_owned())];
            assert_eq!(
                vec!["rails-fulcrum".to_owned()],
                missing_services(expected, actual)
            );
        }

        #[test]
        pub fn ignores_extra_services() {
            let expected = vec!["rails-fulcrum".to_owned(), "query".to_owned()];
            let actual = vec![
                Some("query".to_owned()),
                Some("attachments".to_owned()),
                None,
            ];
            assert_eq!(
                vec!["rails-fulcrum".to_owned()],
                missing_services(expected, actual),
            );
        }

        #[test]
        pub fn returns_nothing_when_full_match() {
            let expected = vec!["rails-fulcrum".to_owned(), "query".to_owned()];
            let actual = vec![Some("rails-fulcrum".to_owned()), Some("query".to_owned())];
            let empty: Vec<String> = vec![];
            assert_eq!(empty, missing_services(expected, actual),);
        }
    }
}
