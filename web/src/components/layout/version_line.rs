//! The running build id with a "Check for updates" button, at the foot of the sidebar and drawer.

use crate::components::ui::{Button, ButtonVariant, Size, Tone};
use crate::hooks::use_app_update::use_app_update;
use crate::hooks::use_toast::use_toast;
use crate::server::api::version::BUILD_ID;
use dioxus::prelude::*;

#[component]
pub fn VersionLine() -> Element {
    let update = use_app_update();
    let toast = use_toast();
    let mut checking = use_signal(|| false);

    rsx! {
        div { class: "mr-version",
            span { "Version {BUILD_ID}" }
            Button {
                variant: ButtonVariant::Ghost,
                size: Size::Sm,
                loading: checking(),
                onclick: move |_| {
                    spawn(async move {
                        checking.set(true);
                        match update.check().await {
                            Ok(true) => {}
                            Ok(false) => toast.show(None, "You have the latest version.".to_string(), Tone::Green),
                            Err(_) => toast.show(
                                Some("Could not check for updates".to_string()),
                                "Check your connection and try again.".to_string(),
                                Tone::Red,
                            ),
                        }
                        checking.set(false);
                    });
                },
                "Check for updates"
            }
        }
    }
}
