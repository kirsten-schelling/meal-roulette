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
        document::Script { {hooks::use_theme::EARLY_SCRIPT} }
        Router::<Route> {}
    }
}
