use warp::Filter;
use warp::{ Rejection};
use clap::Parser;

mod state;
mod ws_handler;


// Import std::path for handling file paths
use std::path::Path;

/// Command line arguments for the WebSocket bridge
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about)]
pub struct WSBridgeArgs {
    /// WebSocket server address
    #[clap(short, long, default_value = "127.0.0.1")]
    pub address: String,

    /// WebSocket server port
    #[clap(short, long, default_value_t = 3031)]
    pub port: u16,

    /// Data send rate in Hz
    #[clap(short, long, default_value_t = 1000.0)]
    pub send_rate_hz: f64,
}

#[tokio::main]
async fn main() {
    let args = WSBridgeArgs::parse();
    pretty_env_logger::init();
    
    // Initialize state with command line arguments
    let mut state = state::WSBridgeState::new();
    let state = state.as_handle();
    
    // WebSocket route
    let ws_route = warp::path("ws")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(warp::ws())
        .and(with_state(state)) 
        .and_then(ws_handler::ws_handler);


    println!("Server started at http://0.0.0.0:{}", args.port);

    warp::serve(ws_route).run(([0, 0, 0, 0], args.port)).await;
}

fn with_state(state: state::StateHandle) -> impl Filter<Extract = (state::StateHandle,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}
