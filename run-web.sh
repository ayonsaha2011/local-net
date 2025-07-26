#!/bin/bash

echo "🚀 Starting Local Network Chat (Web Version)"
echo "Due to snap environment limitations, using web version instead of desktop"
echo ""
echo "The app will be available at: http://localhost:8080"
echo "Press Ctrl+C to stop the server"
echo ""

# Kill any existing dx serve processes
pkill -f "dx serve" 2>/dev/null

# Start the web server
dx serve --platform web --port 8080 --open