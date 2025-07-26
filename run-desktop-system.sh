#!/bin/bash

echo "🚀 Local Network Chat - Desktop Version (System Libraries)"
echo "=================================================="
echo ""

# Check if binary exists
BINARY="./target/dx/local-net-chat/release/linux/app/local-net-chat"
if [ ! -f "$BINARY" ]; then
    echo "❌ Desktop app binary not found at: $BINARY"
    echo "🔧 Building first..."
    dx build --platform desktop --release
    if [ $? -ne 0 ]; then
        echo "❌ Build failed. Exiting..."
        exit 1
    fi
fi

echo "ℹ️  System Information:"
echo "   Glibc version: $(ldd --version | head -1)"
echo "   Architecture: $(uname -m)"
echo "   Kernel: $(uname -r)"
echo ""

# Detect snap environment
SNAP_ENV=false
if [ -n "$SNAP" ] || [ -d "/snap/core20" ] || [ -n "$SNAP_INSTANCE_NAME" ]; then
    SNAP_ENV=true
    echo "⚠️  SNAP ENVIRONMENT DETECTED"
    echo "   This may cause glibc symbol conflicts."
    echo "   Recommended: Use web version instead (./run-web.sh)"
    echo ""
fi

# Library analysis
echo "🔍 Library Analysis:"
echo "   System libc: $(readlink -f /lib/x86_64-linux-gnu/libc.so.6)"
if [ "$SNAP_ENV" = true ]; then
    echo "   Snap libc: $(ls /snap/core*/current/lib/x86_64-linux-gnu/libc.so.* 2>/dev/null | head -1)"
fi
echo ""

# Try different execution strategies
echo "🚀 Attempting to launch desktop application..."
echo ""

# Strategy 1: Force system library path with LD_PRELOAD
echo "📋 Strategy 1: Force system libraries with LD_PRELOAD"
export LD_LIBRARY_PATH="/usr/lib/x86_64-linux-gnu:/lib/x86_64-linux-gnu"
export LD_PRELOAD="/usr/lib/x86_64-linux-gnu/libpthread.so.0"
export LIBRARY_PATH="/lib/x86_64-linux-gnu:/usr/lib/x86_64-linux-gnu"

# Clear snap variables
unset SNAP SNAP_DATA SNAP_COMMON SNAP_USER_DATA SNAP_USER_COMMON SNAP_INSTANCE_NAME

echo "   Libraries: $LD_LIBRARY_PATH"
echo "   Preload: $LD_PRELOAD"
echo "   Executing: $BINARY"
echo ""

# Execute the binary with the preload fix
"$BINARY" "$@"
EXIT_CODE=$?

# If that fails, try without preload
if [ $EXIT_CODE -ne 0 ]; then
    echo ""
    echo "📋 Strategy 2: System libraries without preload"
    unset LD_PRELOAD
    export LD_LIBRARY_PATH="/lib/x86_64-linux-gnu:/usr/lib/x86_64-linux-gnu:/lib64"
    echo "   Libraries: $LD_LIBRARY_PATH"
    echo "   Executing: $BINARY"
    echo ""
    
    "$BINARY" "$@"
    EXIT_CODE=$?
fi

if [ $EXIT_CODE -ne 0 ]; then
    echo ""
    echo "❌ Desktop app failed to start (exit code: $EXIT_CODE)"
    echo ""
    echo "🔧 TROUBLESHOOTING:"
    echo "   1. Snap environment conflicts detected"
    echo "   2. Use web version instead: ./run-web.sh"
    echo "   3. Or try building on a non-snap system"
    echo ""
    echo "🌐 Starting web version as fallback..."
    exec ./run-web.sh
else
    echo ""
    echo "✅ Desktop app started successfully!"
fi