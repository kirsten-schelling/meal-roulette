use dioxus::prelude::*;

use crate::{
    components::ui::Toast,
    hooks::use_toast::{ToastController, ToastItem},
};

#[component]
pub fn ToastMaster(children: Element) -> Element {
    let mut toasts = use_signal(Vec::<ToastItem>::new);
    use_context_provider(|| ToastController { toasts });

    rsx! {
        div {
            class: "mr-toast-stack",
            role: "region",
            aria_label: "Notifications",
            aria_live: "polite",
            for toast in toasts() {
                Toast {
                    key: "{toast.id}",
                    tone: toast.tone,
                    on_dismiss: move |_| {
                        toasts.write().retain(|t| t.id != toast.id);
                    },
                    title: toast.title,
                    "{toast.message}"
                }
            }
        }
        {children}
    }
}
