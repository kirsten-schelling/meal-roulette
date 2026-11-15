//! Tells the app when the server runs a newer build than the one loaded in the browser.
//! `provide_app_update` runs once in the app shell; it checks on launch, whenever the app
//! becomes visible again and every few minutes. A background check that fails (offline, server
//! restarting) just waits for the next one.

use crate::server::api::version::{get_build_id, BUILD_ID};
use dioxus::prelude::*;

const WATCH: &str = r#"
const ping = () => dioxus.send(true);
document.addEventListener('visibilitychange', () => {
  if (document.visibilityState === 'visible') ping();
});
setInterval(ping, 5 * 60 * 1000);
ping();
"#;

#[derive(Clone, Copy)]
pub struct AppUpdate {
    update_ready: Signal<bool>,
}

pub fn provide_app_update() {
    let update = use_context_provider(|| AppUpdate {
        update_ready: Signal::new(false),
    });
    use_effect(move || {
        let mut watch = document::eval(WATCH);
        spawn(async move {
            while watch.recv::<bool>().await.is_ok() {
                let _ = update.check().await;
            }
        });
    });
}

pub fn use_app_update() -> AppUpdate {
    use_context::<AppUpdate>()
}

impl AppUpdate {
    pub fn is_ready(self) -> bool {
        (self.update_ready)()
    }

    /// Asks the server for its build id and raises the banner when it differs from this one.
    pub async fn check(self) -> Result<bool, ServerFnError> {
        let mut update_ready = self.update_ready;
        let newer = get_build_id().await? != BUILD_ID;
        if newer {
            update_ready.set(true);
        }
        Ok(newer)
    }

    /// The server marks index.html `no-cache` and content-hashes the bundle, so a plain reload
    /// fetches the new wasm.
    pub fn reload(self) {
        let _ = document::eval("location.reload();");
    }
}
