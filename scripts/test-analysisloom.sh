#!/bin/bash
# Test AnalysisLoom with DRI3 fix and capture ALL output
export DISPLAY=:99
export MESA_LOADER_DRIVER_OVERRIDE=llvmpipe
export LIBGL_ALWAYS_SOFTWARE=1
export EGL_PLATFORM=x11
export GDK_BACKEND=x11
export ANALYSISLOOM_SCREENSHOT=1

echo "=== ENV TEST ==="
echo "DISPLAY=$DISPLAY"
echo "MESA_LOADER_DRIVER_OVERRIDE=$MESA_LOADER_DRIVER_OVERRIDE"
echo "LIBGL_ALWAYS_SOFTWARE=$LIBGL_ALWAYS_SOFTWARE"
echo "EGL_PLATFORM=$EGL_PLATFORM"
echo "GDK_BACKEND=$GDK_BACKEND"

echo "=== GLX TEST ==="
glxinfo -B 2>&1 | grep -E "renderer|vendor|version"
echo ""
echo "=== EGL TEST ==="
eglinfo -p x11 2>&1 | grep -v "EGL" | head -20
echo ""
echo "=== STARTING APP ==="
BINARY="/home/kali/ysf-forensic-suite/packages/analysisloom/src-tauri/target/debug/analysisloom"
echo "Binary: $BINARY"
ls -la "$BINARY"
"$BINARY" 2>&1 &
APP_PID=$!
echo "PID: $APP_PID"

sleep 10

WID=$(xdotool search --name "AnalysisLoom" 2>/dev/null | head -1)
echo "Window: $WID"

if [ -n "$WID" ]; then
  xdotool windowmap "$WID"
  xdotool windowsize "$WID" 1200 780
  xdotool windowmove "$WID" 40 20
  sleep 3
  import -window "$WID" /tmp/analysisloom_test2.png
  echo "Screenshot: $(identify /tmp/analysisloom_test2.png 2>/dev/null)"
fi

sleep 2
kill "$APP_PID" 2>/dev/null
wait "$APP_PID" 2>/dev/null
echo "=== DONE ==="