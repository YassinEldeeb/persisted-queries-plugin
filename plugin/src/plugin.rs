use std::sync::mpsc;
use std::thread;

use apollo_router::plugin::{Plugin, PluginInit};
use apollo_router::register_plugin;
use apollo_router::services::router::{self, Body};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower::{BoxError, ServiceBuilder, ServiceExt};

#[derive(Debug, Default, Deserialize, JsonSchema, Clone)]
pub struct Conf {
    enabled: bool,
}

pub struct PersistedQueryPlugin {
    configuration: Conf,
}

#[derive(Deserialize, Serialize)]
struct RequestBody {
    query: Option<String>,
    variables: Option<serde_json::Value>,
}

#[async_trait]
impl Plugin for PersistedQueryPlugin {
    type Config = Conf;

    async fn new(init: PluginInit<Self::Config>) -> Result<Self, BoxError> {
        Ok(PersistedQueryPlugin {
            configuration: init.config,
        })
    }

    fn router_service(&self, service: router::BoxService) -> router::BoxService {
        ServiceBuilder::new()
            .map_request(move |mut req: router::Request| {
                let (tx, rx) = mpsc::channel();

                // Spawn an async task to handle the body processing
                tokio::spawn(async move {
                    let mut req_body = req.router_request.body_mut();
                    let mut data = Vec::new();

                    while let Some(chunk) = req_body.data().await {
                        data.extend_from_slice(&chunk.expect("Failed to read chunk"));
                    }

                    let body_str =
                        String::from_utf8(data).expect("Failed to convert body to string");

                    // Deserialize the string into a RequestBody
                    let request_body: RequestBody =
                        serde_json::from_str(&body_str).expect("Failed to deserialize body");

                    // Extract document_id from the request body variables
                    let document_id = request_body
                        .variables
                        .as_ref()
                        .and_then(|vars| vars.get("documentId"))
                        .and_then(|value| value.as_str())
                        .map(|s| s.to_string());

                    if let Some(document_id) = document_id {
                        println!("Document ID found: {}", document_id);

                        let client = reqwest::blocking::Client::builder()
                            .danger_accept_invalid_certs(true)
                            .build()
                            .expect("Failed to build client");

                        let url = format!(
                            "http://localhost:3000/client-name/client-version/{}",
                            document_id
                        );

                        let response = client.get(&url).send();

                        match response {
                            Ok(resp) => match resp.text() {
                                Ok(query_document) => {
                                    println!("Fetched query document: {}", query_document);
                                    tx.send(Some(query_document))
                                        .expect("Failed to send query document");
                                }
                                Err(e) => {
                                    println!("Failed to read query document: {}", e);
                                    tx.send(None).expect("Failed to send error");
                                }
                            },
                            Err(e) => {
                                println!("Failed to fetch persisted query document: {}", e);
                                tx.send(None).expect("Failed to send error");
                            }
                        }
                    } else {
                        println!("Document ID not found in the request");
                        tx.send(None).expect("Failed to send error");
                    }
                });

                // Wait for the async task to complete and update the request body
                if let Ok(Some(query_document)) = rx.recv() {
                    let updated_request_body = RequestBody {
                        query: Some(query_document),
                        variables: None,
                    };

                    let updated_request_body_bytes = serde_json::to_vec(&updated_request_body)
                        .expect("Failed to serialize body");

                    *req.router_request.body_mut() = Body::from(updated_request_body_bytes);
                } else {
                    println!("Failed to receive query document");
                }

                req
            })
            .service(service)
            .boxed()
    }
}

register_plugin!("example", "persisted_query_plugin", PersistedQueryPlugin);
