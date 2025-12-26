#!/bin/bash

# Exit on error
set -e

echo "🚀 Starting AntiGravity Hands-on System..."

# Start containers in background
docker-compose up -d

echo "✅ Docker containers are up!"
echo "🌐 Starting ngrok tunnel on port 5173..."

# Start ngrok
# Note: This requires ngrok to be installed and authenticated
ngrok http 5173 --log=stdout &

sleep 5

# Get the public URL from ngrok API
NGROK_URL=$(curl -s http://localhost:4040/api/tunnels | grep -o 'https://[^"]*.ngrok-free.app' | head -n 1)

if [ -z "$NGROK_URL" ]; then
    echo "❌ Failed to get ngrok URL. Is ngrok running?"
else
    echo "------------------------------------------------"
    echo "🎉 Your Codelab is now PUBLIC!"
    echo "Admin Dashboard: $NGROK_URL/admin"
    echo "Attendee Entry:  $NGROK_URL"
    echo "------------------------------------------------"
fi

# Keep script running to maintain ngrok process
wait
