use dioxus::prelude::*;

mod components;
mod models;
mod server;
mod showcase;

use showcase::Showcase;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/assets/primitives.css") }
        Showcase {}
    }
}
