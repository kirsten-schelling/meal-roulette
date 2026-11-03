use dioxus::prelude::*;

mod components;
mod features;
mod hooks;
mod models;
mod pages;
mod router;
mod server;
mod showcase;

use router::Route;

fn main() {
    #[cfg(all(not(feature = "server"), not(target_arch = "wasm32")))]
    dioxus::fullstack::set_server_url(
        option_env!("MR_SERVER_URL").unwrap_or("http://10.0.2.2:8080"),
    );

    #[cfg(not(feature = "server"))]
    dioxus::launch(App);

    #[cfg(feature = "server")]
    {
        let _ = dotenvy::dotenv();
        server::serve(App);
    }
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/assets/primitives.css") }
        Router::<Route> {}
    }
}
