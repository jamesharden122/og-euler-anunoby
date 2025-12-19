#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_logger::tracing;
#[cfg(feature = "server")]
use dioxus::server::{axum, DioxusRouterExt as _, FullstackState, ServeConfig};
use og_euler_anunoby::App;

fn main() {
    #[cfg(feature = "server")]
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(launch_server());
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[cfg(feature = "server")]
async fn launch_server() {
    // Connect to dioxus' logging infrastructure
    dioxus::logger::initialize_default();

    // Connect to the IP and PORT env vars passed by the Dioxus CLI (or your dockerfile)
    let socket_addr = dioxus_cli_config::fullstack_address_or_localhost();

    // Build a custom axum router
    let router = axum::Router::<FullstackState>::new()
        .serve_dioxus_application(ServeConfig::new(), App)
        .into_make_service();

    // And launch it!
    let listener = tokio::net::TcpListener::bind(socket_addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
