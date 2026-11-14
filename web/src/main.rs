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

/// The icons and manifest keep their plain names (no content hash) because the manifest refers
/// to its icons by relative path, and so lands beside them.
const PLAIN: AssetOptions = AssetOptions::builder()
    .with_hash_suffix(false)
    .into_asset_options();

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/assets/primitives.css") }
        // The default viewport meta lacks viewport-fit=cover, which the safe-area padding needs.
        // resizes-content makes Android Chrome shrink the layout viewport (and so the dialog's
        // dvh cap) when the keyboard opens, instead of covering the focused field.
        document::Meta {
            name: "viewport",
            content: "width=device-width, initial-scale=1, viewport-fit=cover, interactive-widget=resizes-content",
        }
        document::Meta { name: "theme-color", content: "#fbf6ef" }
        document::Link {
            rel: "manifest",
            href: asset!("/assets/pwa/manifest.webmanifest", PLAIN),
        }
        document::Link {
            rel: "icon",
            r#type: "image/svg+xml",
            href: asset!("/assets/pwa/icon.svg", PLAIN),
        }
        document::Link {
            rel: "apple-touch-icon",
            href: asset!("/assets/pwa/apple-touch-icon.png", PLAIN),
        }
        document::Link {
            rel: "icon",
            r#type: "image/png",
            sizes: "192x192",
            href: asset!("/assets/pwa/icon-192.png", PLAIN),
        }
        document::Link {
            rel: "icon",
            r#type: "image/png",
            sizes: "512x512",
            href: asset!("/assets/pwa/icon-512.png", PLAIN),
        }
        // Named only by the manifest; listed here so the asset is bundled.
        document::Link { rel: "prefetch", href: asset!("/assets/pwa/icon-maskable-512.png", PLAIN) }
        document::Script { {hooks::use_theme::EARLY_SCRIPT} }
        Router::<Route> {}
    }
}
