use crate::grammar::Parsed;
use jsonrpc_tcp_server::jsonrpc_core::{IoHandler, Params, RpcMethodSimple, RpcNotificationSimple};
use log::*;
use std::default::Default;
use std::sync::{Arc, Mutex, RwLock};
use tokio::net::TcpListener;
use tokio::sync::watch;

#[allow(unused_imports)]
use std::net::SocketAddr;

type SerializedResponse =
    std::result::Result<serde_json::Value, jsonrpc_tcp_server::jsonrpc_core::Error>;
fn serialized_response<T: serde::Serialize>(response: T) -> SerializedResponse {
    serde_json::to_value(response).map_err(|e| {
        error!("Serialization error: {}", e);
        jsonrpc_tcp_server::jsonrpc_core::Error::new(
            jsonrpc_tcp_server::jsonrpc_core::ErrorCode::InternalError,
        )
    })
}
trait Serializable {
    fn serialize(&self) -> SerializedResponse;
}

impl Serializable for serde_json::Value {
    fn serialize(&self) -> SerializedResponse {
        serialized_response(self)
    }
}

mod advice {
    fn disconnected_participants() {
        // find any participants that have no connectsion
    }
}

type ReadWriteGuarded<T> = Arc<RwLock<Option<T>>>;
fn empty<T>() -> ReadWriteGuarded<T> {
    Arc::new(RwLock::new(None))
}

mod events {
    #[derive(Default)]
    pub(crate) struct Events {}

    impl Events {
        fn fireDocumentUpdated(event: DocumentUpdated) {}
    }

    pub(crate) struct DocumentUpdated {
        uri: String,
        content: String,
    }

    #[derive(Debug, Clone)]
    pub enum Event {}
}

pub struct PlantUmlLanguageServer {
    address: String,
    parsed: Arc<RwLock<Option<Parsed>>>,
    handler: Arc<IoHandler<()>>,
    bus: (
        tokio::sync::broadcast::Sender<events::Event>,
        tokio::sync::broadcast::Receiver<events::Event>,
    ),
}

fn onHello(_params: Params) -> SerializedResponse {
    serde_json::Value::String("hello".to_string()).serialize()
}

impl PlantUmlLanguageServer {
    pub fn new(address: &str) -> Self {
        let parsed: ReadWriteGuarded<Parsed> = empty();
        let mut handler = IoHandler::<()>::default();
        let events = events::Events {};

        let plock = parsed.clone();
        info!("Registering rpc methods");
        handler.add_notification("textDocument/didChange", move |params: Params| {
            let p = params.parse::<lsp_types::DidChangeTextDocumentParams>();
            if let Ok(x) = p {
                let text: String = x.content_changes.iter().next().unwrap().text.to_string();
                let p = crate::parse(&text);
                let res = p
                    .root()
                    .participant_decls()
                    .filter_map(|it| it.participant_name())
                    .map(|it| it.identifier())
                    .collect::<Vec<_>>();
                trace!("Participant Names - {:?}", res);

                let mut x = plock.write().unwrap();
                x.replace(p);
            }
        });

        handler.add_method("say_hello", onHello);

        handler.add_notification("textDocument/didOpen", |params: Params| {
            let p = params.parse::<lsp_types::DidOpenTextDocumentParams>();
            debug!("textDocument/didOpen {:?}", p);
        });

        let plock = parsed.clone();
        handler.add_method("textDocument/completion", move |params: Params| {
            debug!("Initialize called");
            let p = params.parse::<lsp_types::CompletionParams>();
            debug!("initialize {:?}", p);

            let items = match plock.try_read() {
                Ok(locked) => locked
                    .as_ref()
                    .unwrap()
                    .root()
                    .participant_decls()
                    .filter_map(|it| it.participant_name())
                    .map(|it| lsp_types::CompletionItem {
                        kind: Some(lsp_types::CompletionItemKind::Struct),
                        label: it.identifier().to_string(),
                        //detail: Some("This is detailed sup".into()),
                        //insert_text: Some(it.identifier
                        ..Default::default()
                    })
                    .collect(),
                Err(_) => vec![],
            };
            let response = lsp_types::CompletionList {
                is_incomplete: false,
                items,
            };
            let response = lsp_types::CompletionResponse::List(response);
            serialized_response(response)
        });

        handler.add_method("initialize", |params: Params| {
            debug!("Initialize called");
            let p = params.parse::<lsp_types::InitializeParams>();
            debug!("initialize {:?}", p);
            let response = lsp_types::InitializeResult {
                server_info: Some(lsp_types::ServerInfo {
                    name: "pummls".into(),
                    version: Some("0.0.1".into()),
                }),
                capabilities: lsp_types::ServerCapabilities {
                    text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Kind(
                        lsp_types::TextDocumentSyncKind::Full,
                    )),
                    completion_provider: Some(lsp_types::CompletionOptions {
                        trigger_characters: Some(vec![" ".into()]),
                        resolve_provider: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            };
            let response = serde_json::to_value(response).map_err(|e| {
                error!("Serialization error: {}", e);
                jsonrpc_tcp_server::jsonrpc_core::Error::new(
                    jsonrpc_tcp_server::jsonrpc_core::ErrorCode::InternalError,
                )
            })?;
            Ok(response)
        });

        PlantUmlLanguageServer {
            address: address.to_owned(),
            parsed,
            handler: Arc::new(handler),
            bus: tokio::sync::broadcast::channel::<events::Event>(10),
        }
    }

    pub async fn start(&self) -> super::Result<()> {
        let listener = TcpListener::bind(&self.address).await?;
        println!("listening on {}", &self.address);
        loop {
            let (socket, _) = listener.accept().await?;
            let h = self.handler.clone();
            tokio::spawn(async move {
                debug!("Spawning lsp connection");
                let l = conn::LspConnection::new(h);
                l.run(socket).await;
                debug!("LSP Connection done");
            });
        }
    }
}

mod conn {
    use super::super::codec::*;
    use futures03::sink::SinkExt;
    use futures03::stream::StreamExt;
    use jsonrpc_tcp_server::jsonrpc_core::futures::Future;
    use jsonrpc_tcp_server::jsonrpc_core::IoHandler;
    use log::*;
    use tokio_util::codec::Framed;

    pub struct LspConnection {
        handler: std::sync::Arc<IoHandler>,
    }

    impl LspConnection {
        pub fn new(handler: std::sync::Arc<IoHandler>) -> Self {
            LspConnection { handler }
        }

        pub async fn run(&self, socket: tokio::net::TcpStream) {
            let codec = LspCodec::new();
            let (mut _sink, mut input) = Framed::new(socket, codec).split();
            while let Some(Ok(event)) = input.next().await {
                match event {
                    LspEvent::Message(value) => {
                        debug!(target: "server", "Received message: {}", value);
                        let f = futures03::compat::Compat01As03::new(
                            self.handler
                                .handle_request(&value)
                                .map(|x| Ok::<Option<String>, ()>(x)),
                        )
                        .await;

                        if let Ok(response) = f {
                            match response {
                                Err(e) => {
                                    warn!(target: "tcp", "Error while processing request: {:?}", e);
                                }
                                Ok(None) => {
                                    trace!(target: "tcp", "JSON RPC request produced no response");
                                }
                                Ok(Some(response_data)) => {
                                    trace!(target: "tcp", "Sending response: {}", &response_data);
                                    let response =
                                        _sink.send(LspEvent::Message(response_data)).await;
                                    match response {
                                        Ok(_) => debug!(target:"tcp", "Done."),
                                        Err(e) => warn!(target: "tcp", "Failed to send {}", e),
                                    }
                                }
                            }
                        }
                    }
                };
            }
            debug!("Exiting LSP Connection Loop");
        }
    }
}
