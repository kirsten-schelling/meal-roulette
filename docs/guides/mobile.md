# Mobile (Android)

The `web` crate also builds as a native client: the same components and CSS render inside the
platform WebView, and `#[server]` functions are called over HTTP on the server you point it at.
Only Android is covered here. iOS needs a Mac and is not set up.

## Installing on the phone

The installed web app is the supported phone client. Open the server's tailnet HTTPS URL in
Chrome on the phone and choose "Install app". It loads from the server, so every deploy reaches
it with no rebuild and no cable. When a new version is out, a "New version is ready" bar offers a
reload; the menu also shows the running version and a "Check for updates" button.

The native APK below is for development only: it bundles its own code, so each update means a
rebuild and a USB install, and it does not show the update bar.

## What the native build does

- Features: `mobile` (Android) and `desktop` (a native window on the dev machine, a faster loop
  than the emulator for the same code path). Both replace the default `web` feature, so pass
  `--no-default-features`; the justfile recipes do.
- On native client builds the app calls `set_server_url` before launch. The URL comes from the
  `MR_SERVER_URL` env var at compile time and defaults to `http://10.0.2.2:8080`, which is how the
  Android emulator reaches the host machine's `localhost`.
- The server keeps its normal binding. It is not exposed on the LAN; a real phone reaches it over
  the tailnet (see below).

## Toolchain on Windows

1. Install Android Studio. In SDK Manager install an SDK Platform (API 34 or newer), SDK
   Command-line Tools, NDK (Side by side), CMake and Platform-Tools. In Device Manager create an
   x86_64 emulator.
2. Add the Rust targets:
   `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android`
3. Set user environment variables (PowerShell), using the versions you installed, then restart
   the terminal:
   ```powershell
   [Environment]::SetEnvironmentVariable("JAVA_HOME", "C:\Program Files\Android\Android Studio\jbr", "User")
   [Environment]::SetEnvironmentVariable("ANDROID_HOME", "$env:LOCALAPPDATA\Android\Sdk", "User")
   [Environment]::SetEnvironmentVariable("NDK_HOME", "$env:LOCALAPPDATA\Android\Sdk\ndk\<version>", "User")
   ```
   Add `%ANDROID_HOME%\platform-tools` and `%ANDROID_HOME%\emulator` to `PATH`. Check with
   `adb version` and `emulator -list-avds`.
4. Install `dioxus-cli` at the same 0.7.x as the `dioxus` dependency in `Cargo.lock`
   (`cargo install dioxus-cli --version <x.y.z>`). A mismatched `dx` is a common failure.

## Running

Start the database and the server first (`just db`, `just web`), then in another terminal:

| Command        | Runs on                                                                                          |
| -------------- | ------------------------------------------------------------------------------------------------ |
| `just desktop` | a native window on this machine, talking to `http://10.0.2.2:8080` unless `MR_SERVER_URL` is set |
| `just android` | the booted emulator (`emulator -avd <name>`) or a USB device                                     |

On the desktop, `10.0.2.2` does not resolve, so set `MR_SERVER_URL=http://localhost:8080`
before `just desktop`.

## A real phone

`10.0.2.2` only works in the emulator. Build with `MR_SERVER_URL` set to the URL the phone can
reach, which is the tailnet HTTPS URL that `tailscale serve` publishes for the server (the
tailscale access setup describes how to turn it on). The phone must be signed in to the same
tailnet.

```powershell
$env:MR_SERVER_URL = "https://<machine>.<tailnet>.ts.net"
just android
```

`MR_SERVER_URL` is read at compile time; changing it means rebuilding.

## App identity

`web/Dioxus.toml` sets the package id (`dev.kirsten.mealroulette`) and publisher. The launcher
label comes from `web/android/AndroidManifest.xml`, because dx otherwise derives it from the
crate name. dx has no Android icon setting and always writes its stock icons, so after
`dx build --android` run `bash web/android/apply-icon.sh` (same `CARGO_TARGET_DIR`) to swap in
`web/assets/pwa/icon-192.png` and rebuild the APK. `dx serve` / `just android` install with the
stock icon. The package id differs from the old `com.example.Web`, so it installs as a separate
app; the old "Web" app can be uninstalled.
