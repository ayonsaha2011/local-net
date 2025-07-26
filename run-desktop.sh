#!/bin/bash

echo "🚀 Starting Local Network Chat (Desktop Version)"
echo ""

# Build check
if [ ! -f "./target/dx/local-net-chat/release/linux/app/local-net-chat" ]; then
    echo "❌ Desktop app not found. Building first..."
    dx build --platform desktop --release
    if [ $? -ne 0 ]; then
        echo "❌ Build failed. Exiting..."
        exit 1
    fi
fi

# Check if we're in a snap environment
if [ -n "$SNAP" ] || [ -d "/snap/core20" ]; then
    echo "⚠️  Snap environment detected. Applying glibc conflict fix..."
    echo ""
    
    # Clear snap-related environment variables
    unset SNAP
    unset SNAP_DATA
    unset SNAP_COMMON
    unset SNAP_USER_DATA
    unset SNAP_USER_COMMON
    
    # Apply the LD_PRELOAD fix for snap environments
    export LD_LIBRARY_PATH="/usr/lib/x86_64-linux-gnu:/lib/x86_64-linux-gnu"
    export LD_PRELOAD="/usr/lib/x86_64-linux-gnu/libpthread.so.0"
    export LIBRARY_PATH="/lib/x86_64-linux-gnu:/usr/lib/x86_64-linux-gnu"
    export PATH="/usr/local/bin:/usr/bin:/bin"
    
    echo "🔧 Using LD_PRELOAD fix for snap environment"
    echo "   Libraries: $LD_LIBRARY_PATH"
    echo "   Preload: $LD_PRELOAD"
    echo ""
fi

# Run the application
echo "Running desktop application..."
exec ./target/dx/local-net-chat/release/linux/app/local-net-chat