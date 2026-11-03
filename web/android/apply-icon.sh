#!/usr/bin/env bash
# dx 0.7 always writes its stock launcher icons into the generated Gradle project and has no
# Android icon setting, so swap them for the brand icon and rebuild the APK with Gradle.
# Run after `dx build --android ...` with the same CARGO_TARGET_DIR.
set -euo pipefail
target="${CARGO_TARGET_DIR:-target}"
profile="${1:-debug}"
proj="$target/dx/web/$profile/android/app"
res="$proj/app/src/main/res"
rm -f "$res"/mipmap-*/ic_launcher.webp
rm -rf "$res/mipmap-anydpi-v26"
mkdir -p "$res/mipmap-xxxhdpi"
cp "$(dirname "$0")/../assets/pwa/icon-192.png" "$res/mipmap-xxxhdpi/ic_launcher.png"
cd "$proj"
task=assembleDebug; [ "$profile" = release ] && task=assembleRelease
./gradlew.bat "$task" -q
