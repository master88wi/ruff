use ruff::RUFF_PKG_VERSION;
use std::future::Future;
use std::pin::Pin;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentFormattingParams, InitializeParams, InitializeResult, InitializedParams, MessageType,
    PositionEncodingKind, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextEdit,
};
use tower_lsp::{Client, LanguageServer, LspService};

/// Creates a LSP server that reads from stdin and writes the output to stdout.
pub fn stdio() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let (service, socket) = LspService::new(|client| Server { client });
        tower_lsp::Server::new(stdin, stdout, socket)
            .serve(service)
            .await;
    });
}

struct Server {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    #[tracing::instrument(level="debug", skip_all, err, fields(client=?params.client_info))]
    async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        let init = InitializeResult {
            capabilities: ServerCapabilities {
                // TODO
                position_encoding: None,
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: String::from(env!("CARGO_PKG_NAME")),
                version: Some(RUFF_PKG_VERSION.to_string()),
            }),
        };

        Ok(init)
    }

    #[tracing::instrument(skip_all)]
    async fn initialized(&self, params: InitializedParams) {}

    #[tracing::instrument(skip_all, fields(file=%params.text_document.uri))]
    async fn did_open(&self, params: DidOpenTextDocumentParams) {}

    #[tracing::instrument(skip_all, fields(file=%params.text_document.uri))]
    async fn did_change(&self, params: DidChangeTextDocumentParams) {}

    #[tracing::instrument(skip_all, fields(file=%params.text_document.uri))]
    async fn did_close(&self, params: DidCloseTextDocumentParams) {}

    #[tracing::instrument(skip_all, fields(file=%params.text_document.uri))]
    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> LspResult<Option<Vec<TextEdit>>> {
        Ok(Some(vec![]))
    }

    #[tracing::instrument(skip_all, err)]
    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }
}
