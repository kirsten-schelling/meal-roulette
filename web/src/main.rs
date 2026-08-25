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
    #[cfg(feature = "server")]
    let _ = dotenvy::dotenv();

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/assets/primitives.css") }
        Router::<Route> {}
    }
}
