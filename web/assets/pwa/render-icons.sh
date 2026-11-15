#!/usr/bin/env bash
# Renders the PNG icons from the two SVG sources. Needs resvg (github.com/linebender/resvg).
set -euo pipefail
cd "$(dirname "$0")"
resvg -w 192 -h 192 icon.svg icon-192.png
resvg -w 512 -h 512 icon.svg icon-512.png
resvg -w 512 -h 512 icon-maskable.svg icon-maskable-512.png
resvg -w 180 -h 180 icon-maskable.svg apple-touch-icon.png
