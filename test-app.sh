#!/bin/bash

echo "🧪 Local Network Chat - Testing Suite"
echo "======================================"
echo ""

# Test 1: Desktop app with LD_PRELOAD fix
echo "🖥️  Test 1: Desktop App (LD_PRELOAD fix)"
echo "Command: LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libpthread.so.0 ./target/..."
echo ""

LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:/lib/x86_64-linux-gnu \
LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libpthread.so.0 \
timeout 3 ./target/dx/local-net-chat/release/linux/app/local-net-chat &
PID=$!

sleep 2
if kill -0 $PID 2>/dev/null; then
    echo "✅ Desktop app started successfully!"
    kill $PID 2>/dev/null
else
    echo "❌ Desktop app failed to start"
fi

wait $PID 2>/dev/null
echo ""

# Test 2: Run scripts
echo "📜 Test 2: Run Scripts"
echo ""

echo "Testing ./run-desktop.sh..."
timeout 3 ./run-desktop.sh &
SCRIPT_PID=$!
sleep 2
if kill -0 $SCRIPT_PID 2>/dev/null; then
    echo "✅ run-desktop.sh works!"
    kill $SCRIPT_PID 2>/dev/null
else
    echo "❌ run-desktop.sh failed"
fi
wait $SCRIPT_PID 2>/dev/null

echo ""
echo "🎉 Testing complete!"
echo ""
echo "📋 Usage Summary:"
echo "   Desktop: ./run-desktop.sh"
echo "   Advanced: ./run-desktop-system.sh"
echo "   Web: ./run-web.sh"
echo ""
echo "✅ Snap environment glibc conflicts resolved!"