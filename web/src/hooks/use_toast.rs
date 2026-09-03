use dioxus::prelude::*;
use uuid::Uuid;

use crate::components::ui::Tone;

#[derive(Clone)]
pub struct ToastItem {
    pub id: Uuid,
    pub title: String,
    pub message: String,
    pub tone: Tone,
}

#[derive(Clone)]
pub struct ToastController {
    pub toasts: Signal<Vec<ToastItem>>,
}

pub fn use_toast() -> ToastController {
    use_context::<ToastController>()
}

impl ToastController {
    pub fn show(&self, title: Option<String>, message: String, tone: Tone) {
        let mut toasts = self.toasts;
        let id = Uuid::now_v7();

        toasts.write().push(ToastItem {
            id,
            title: title.unwrap_or_default(),
            message: message.into(),
            tone,
        });

        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(3_000).await;
            toasts.write().retain(|t| t.id != id);
        });
    }
}
