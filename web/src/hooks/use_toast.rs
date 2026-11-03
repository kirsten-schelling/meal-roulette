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

#[derive(Clone, Copy)]
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
            message,
            tone,
        });

        spawn(async move {
            sleep_ms(3_000).await;
            toasts.write().retain(|t| t.id != id);
        });
    }
}

#[cfg(target_arch = "wasm32")]
async fn sleep_ms(ms: u32) {
    gloo_timers::future::TimeoutFuture::new(ms).await;
}

#[cfg(not(target_arch = "wasm32"))]
async fn sleep_ms(ms: u32) {
    tokio::time::sleep(std::time::Duration::from_millis(u64::from(ms))).await;
}
