//! Keeps the screen on while the calling component is mounted, where the browser
//! supports the Screen Wake Lock API. Browsers drop the lock when the tab is hidden,
//! so it is requested again when the tab becomes visible.

use dioxus::prelude::*;

const ACQUIRE: &str = r#"
try {
  if ('wakeLock' in navigator) {
    const take = async () => {
      try { window.__mrWake = await navigator.wakeLock.request('screen'); } catch (e) {}
    };
    window.__mrWakeVisible = () => {
      if (document.visibilityState === 'visible') take();
    };
    document.addEventListener('visibilitychange', window.__mrWakeVisible);
    take();
  }
} catch (e) {}
"#;

const RELEASE: &str = r#"
try {
  if (window.__mrWakeVisible) {
    document.removeEventListener('visibilitychange', window.__mrWakeVisible);
    window.__mrWakeVisible = null;
  }
  if (window.__mrWake) {
    window.__mrWake.release();
    window.__mrWake = null;
  }
} catch (e) {}
"#;

pub fn use_wake_lock() {
    use_effect(|| {
        let _ = document::eval(ACQUIRE);
    });
    use_drop(|| {
        let _ = document::eval(RELEASE);
    });
}
