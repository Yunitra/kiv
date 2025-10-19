//! # Kiv Language Server Protocol
//!
//! This crate provides LSP support for the Kiv programming language,
//! enabling IDE features like syntax highlighting, code completion, and error checking.

mod backend;

use backend::Backend;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    // Set up logging
    env_logger::init();

    // Create LSP service
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);

    // Run the server
    Server::new(stdin, stdout, socket).serve(service).await;
}
