#!/bin/bash
# ZipLoom Screenshot — satu tab per run (ZIPLOOM_TAB)
set -e

APP_DIR="/home/kali/ysf-forensic-suite/packages/ziploom"
OUT_DIR="$APP_DIR/docs/screenshots"
mkdir -p "$OUT_DIR"
rm -f "$OUT_DIR"/*.png

export DISPLAY=:99
export MESA_LOADER_DRIVER_OVERRIDE=llvmpipe
export LIBGL_ALWAYS_SOFTWARE=1
export EGL_PLATFORM=x11
export GDK_BACKEND=x11
export ZIPLOOM_SCREENSHOT=1

BINARY="$APP_DIR/src-tauri/target/debug/ziploom"

take_screenshot() {
  local tab_idx=$1
  local filename=$2
  local label=$3

  echo "📸 [$label] Starting app (tab $tab_idx)..."
  
  # Run app with specific tab
  ZIPLOOM_TAB=$tab_idx "$BINARY" &
  APP_PID=$!
  
  # Wait for window
  sleep 5
  WID=$(xdotool search --name "ZipLoom" 2>/dev/null | head -1)
  if [ -z "$WID" ]; then sleep 4; WID=$(xdotool search --name "ZipLoom" 2>/dev/null | head -1); fi

  if [ -n "$WID" ]; then
    echo "  Window found — resizing to 1200x680"
    xdotool windowmap "$WID"
    xdotool windowsize "$WID" 1200 680
    xdotool windowmove "$WID" 40 20
    sleep 2
  fi

  # Wait for screenshot mode to prepare tab
  sleep 3

  # Capture just the window (not whole root)
  import -window "$WID" "$OUT_DIR/${filename}.png" 2>/dev/null || \
    import -window root "$OUT_DIR/${filename}.png"
  
  local size=$(du -h "$OUT_DIR/${filename}.png" | cut -f1)
  echo "  ✅ $filename.png ($size)"

  kill "$APP_PID" 2>/dev/null || true
  wait "$APP_PID" 2>/dev/null || true
  sleep 2
}

# Take each tab separately
take_screenshot 0 "ziploom_compress"   "Compress"
take_screenshot 1 "ziploom_extract"    "Extract"
take_screenshot 2 "ziploom_inspect"    "Inspect"
take_screenshot 3 "ziploom_about"      "About"

echo ""
echo "✅ All ZipLoom screenshots done!"
ls -lh "$OUT_DIR"/*.png
