mod app;
mod client_manager;
mod local_executor;
mod net;
mod repl;
mod ui;

use std::sync::Arc;

#[tokio::main]
async fn main() {
    let state = app::new_shared_state();

    // Start the TCP server in the background
    let server_state = Arc::clone(&state);
    tokio::spawn(async move {
        net::start_server(server_state).await;
    });

    // Run the TUI (this blocks until user exits)
    if let Err(e) = ui::run_ui(state).await {
        eprintln!("UI error: {}", e);
    }

    println!("Fluffy Terminal exited. Goodbye!");
}
