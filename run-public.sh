#!/bin/bash

# Exit on error
set -e

# Default values
TUNNEL_TYPE="ngrok"
CONTAINER_ENGINE="docker"

# Check for podman
if command -v podman-compose &> /dev/null; then
    CONTAINER_ENGINE="podman"
elif command -v docker-compose &> /dev/null; then
    CONTAINER_ENGINE="docker"
else
    echo "❌ No container engine (docker-compose or podman-compose) found!"
    exit 1
fi

# Parse arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --bore) TUNNEL_TYPE="bore"; shift ;;
        --ngrok) TUNNEL_TYPE="ngrok"; shift ;;
        *) echo "Unknown parameter passed: $1"; exit 1 ;;
    esac
done

echo "🚀 Starting AntiGravity Hands-on System using $CONTAINER_ENGINE..."

# Start containers in background
if [ "$CONTAINER_ENGINE" == "podman" ]; then
    podman-compose up -d
else
    docker-compose up -d
fi

echo "✅ Containers are up!"

if [ "$TUNNEL_TYPE" == "ngrok" ]; then
    echo "🌐 Starting ngrok tunnel on port 5173..."
    ngrok http 5173 --log=stdout &
    sleep 5
    PUBLIC_URL=$(curl -s http://localhost:4040/api/tunnels | grep -o 'https://[^"]*.ngrok-free.app' | head -n 1)
else
    echo "🌐 Starting bore tunnel on port 5173..."
    # Note: bore requires a server. Defaulting to bore.pub if not specified.
    # Instruction: cargo install bore-cli
    bore local 5173 --to bore.pub &
    sleep 5
    # Bore doesn't have a simple local API like ngrok to get the URL easily if it's dynamic.
    # However, bore.pub usually gives you a dedicated port if you are lucky or you can specify --port.
    # For now, we'll assume the user sees the output.
    echo "⚠️  Please check the bore output above for your public URL."
    PUBLIC_URL="[Check Bore Output]"
fi

if [ -z "$PUBLIC_URL" ] || [ "$PUBLIC_URL" == "[Check Bore Output]" ]; then
    if [ "$TUNNEL_TYPE" == "ngrok" ]; then
        echo "❌ Failed to get ngrok URL. Is ngrok running?"
    fi
else
    echo "------------------------------------------------"
    echo "🎉 Your Codelab is now PUBLIC!"
    echo "Admin Dashboard: $PUBLIC_URL/admin"
    echo "Attendee Entry:  $PUBLIC_URL"
    echo "------------------------------------------------"
fi

# Keep script running to maintain tunnel process
wait
