use super::ActionResult;
use crate::welcome::Action;
use crate::KubeMessage;
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::Api,
    {Client, Error},
};
use std::sync::mpsc::Sender;

pub fn run_action(namespace: String, action: Action, tx: Sender<KubeMessage>) {
    tokio::spawn(async move {
        match Client::try_default().await {
            Ok(client) => {
                run_valid_action(client, tx, namespace, action).await;
            }
            Err(err) => println!("Could not get client, error: {}", err),
        };
    });
}

async fn run_valid_action(
    client: Client,
    tx: Sender<KubeMessage>,
    namespace: String,
    action: Action,
) {
    let validated_action = match action.action.as_str() {
        "get-secret" => {
            let secrets_request: Api<Secret> = Api::namespaced(client, namespace.as_str());
            let secret = secrets_request.get(action.resource.as_str()).await;
            Some(match secret {
                Ok(s) => match s.data {
                    Some(data) => {
                        let mut responses: Vec<String> = vec![];
                        for (key, value) in data.iter() {
                            let parsed_value = match String::from_utf8(value.0.clone()) {
                                Ok(str_value) => str_value,
                                _ => "".to_owned(),
                            };
                            let line = key.to_owned() + ": " + parsed_value.as_str();
                            responses.push(line);
                        }
                        success(action.name, responses.join("\n"))
                    }
                    None => success(action.name, format!("Nothing found!")),
                },
                Err(err) => error(action.name, err),
            })
        }
        _ => None,
    };
    match validated_action {
        Some(msg) => {
            match tx.send(msg) {
                Ok(_) => {}
                Err(e) => println!("Failed running action: {}", e),
            };
        }
        None => {}
    };
}

// For errors we still return an `Ok` so that we can send messages
// per individual action
fn error(action_name: String, err: Error) -> KubeMessage {
    KubeMessage::Action(Ok(ActionResult {
        name: action_name,
        results: format!("Failed to run action: {}", err),
    }))
}

fn success(action_name: String, results: String) -> KubeMessage {
    KubeMessage::Action(Ok(ActionResult {
        name: action_name,
        results,
    }))
}
