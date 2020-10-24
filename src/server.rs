use jsonrpc_tcp_server::jsonrpc_core::IoHandler;
use jsonrpc_tcp_server::jsonrpc_core::Params;
use log::*;
use std::default::Default;
use tokio::net::TcpListener;

#[allow(unused_imports)]
use std::net::SocketAddr;

pub struct PlantUmlLanguageServer {
    address: String,
}
fn serialized_response<T: serde::Serialize>(response: T) -> std::result::Result<serde_json::Value, jsonrpc_tcp_server::jsonrpc_core::Error > 
{
    serde_json::to_value(response).map_err(|e| { 
        error!("Serialization error: {}", e);
                jsonrpc_tcp_server::jsonrpc_core::Error::new(
                    jsonrpc_tcp_server::jsonrpc_core::ErrorCode::InternalError,
                )
    })
}

impl PlantUmlLanguageServer {
    pub fn new(address: &str) -> Self {
        PlantUmlLanguageServer {
            address: address.to_owned(),
        }
    }

    pub async fn start(&mut self) -> super::Result<()> {
        let mut io = IoHandler::<()>::default();
        info!("Registering rpc methods");
        io.add_method("say_hello", |_params| {
            Ok(jsonrpc_tcp_server::jsonrpc_core::Value::String(
                "hello".to_owned(),
            ))
        });

        io.add_notification("textDocument/didOpen", |params: Params| {
            let p = params.parse::<lsp_types::DidOpenTextDocumentParams>();
            debug!("textDocument/didOpen {:?}", p);
        });

        io.add_method("textDocument/completion", |params: Params| {
            debug!("Initialize called");
            let p = params.parse::<lsp_types::CompletionParams>();
            debug!("initialize {:?}", p);

            let item1 = lsp_types::CompletionItem{
                kind: Some(lsp_types::CompletionItemKind::Struct),
                label: "Sup???".into(),
                detail: Some("This is detailed sup".into()),
                insert_text: Some("INSERT ME".into()),
                ..Default::default()
            };
            let response = lsp_types::CompletionList{
                is_incomplete: false,
                items: vec![item1],
            };
            let response = lsp_types::CompletionResponse::List(response);
            serialized_response(response)
        });

        io.add_method("initialize", |params: Params| {
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
                        trigger_characters: Some(vec![":".into(), "\"".into()]),
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

        let handler = std::sync::Arc::new(io);
        let listener = TcpListener::bind(&self.address).await?;
        println!("listening on {}", &self.address);
        loop {
            let h = std::sync::Arc::clone(&handler);
            let (socket, _) = listener.accept().await?;
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
