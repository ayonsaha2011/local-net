#!/bin/bash
export LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:/lib/x86_64-linux-gnu

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

fi
echo "🚀 Starting Local Network Chat (Desktop Version)"
dx serve
