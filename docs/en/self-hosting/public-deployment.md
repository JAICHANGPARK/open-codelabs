# Public deployment (ngrok/bore/cloudflare)

How to expose your local server for workshops or events.

## Overview

Open Codelabs provides a `run-public.sh` script to expose your local server easily:

- **ngrok**: the most popular tunneling service
- **bore**: a Rust-based open-source alternative
- **Cloudflare Tunnel**: provides a free quick tunnel (`cloudflared`)

## Use ngrok

### 1. Install ngrok

=== "macOS"
    ```bash
    brew install ngrok
    ```

=== "Linux"
    ```bash
    # Snap
    snap install ngrok

    # Or download directly
    wget https://bin.equinox.io/c/bNyj1mQVY4c/ngrok-v3-stable-linux-amd64.tgz
    tar xvzf ngrok-v3-stable-linux-amd64.tgz
    sudo mv ngrok /usr/local/bin
    ```

=== "Windows"
    Install from the [ngrok download page](https://ngrok.com/download).

### 2. ngrok auth (optional)

You can use the free plan, but creating an account unlocks more features:

```bash
# Sign up at ngrok.com and copy the token
ngrok config add-authtoken <your_token>
```

### 3. Run

```bash
chmod +x run-public.sh
./run-public.sh --ngrok
```

Example output:

```
Starting Open-Codelabs: Hands-on System using docker...
Containers are up!
Starting ngrok tunnel on port 5173...
------------------------------------------------
Your Codelab is now PUBLIC!
Admin Dashboard: https://abc123.ngrok-free.app/admin
Attendee Entry:  https://abc123.ngrok-free.app
------------------------------------------------
```

### 4. Generate a QR code

Create a QR code so participants can connect easily:

```bash
# Install qrencode
brew install qrencode  # macOS
apt-get install qrencode  # Linux

# Generate a QR code
echo "https://abc123.ngrok-free.app" | qrencode -t UTF8

# Or save as an image
echo "https://abc123.ngrok-free.app" | qrencode -o qr.png
```

The admin dashboard also shows a QR code automatically.

## Use bore

bore is an open-source alternative and can be self-hosted.

### 1. Install bore

```bash
cargo install bore-cli
```

### 2. Run

```bash
./run-public.sh --bore
```

bore uses `bore.pub` by default.

### 3. Custom bore server

Run your own server:

```bash
# Start the server
bore server --secret <your_secret>

# Connect the client
bore local 5173 --to your-server.com --port 80 --secret <your_secret>
```

## Use Cloudflare Tunnel

Cloudflare Tunnel provides a free quick tunnel and generates a `trycloudflare.com` URL automatically.

### 1. Install cloudflared

=== "macOS"
    ```bash
    brew install cloudflare/cloudflare/cloudflared
    ```

=== "Linux"
    Install from the [Cloudflare download guide](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/).

=== "Windows"
    Install from the [Cloudflare download guide](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/).

### 2. Run

```bash
./run-public.sh --cloudflare
```

The public URL is printed in the cloudflared logs.

## run-public.sh details

### Script content

```bash
#!/bin/bash

set -e

# Default values
TUNNEL_TYPE="ngrok"
CONTAINER_ENGINE="docker"
PORT="${PORT:-5173}"

# Check for podman
if command -v podman-compose &> /dev/null; then
    CONTAINER_ENGINE="podman"
elif command -v docker-compose &> /dev/null; then
    CONTAINER_ENGINE="docker"
else
    echo "No container engine found!"
    exit 1
fi

# Parse arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --bore) TUNNEL_TYPE="bore"; shift ;;
        --cloudflare) TUNNEL_TYPE="cloudflare"; shift ;;
        --ngrok) TUNNEL_TYPE="ngrok"; shift ;;
        *) echo "Unknown parameter: $1"; exit 1 ;;
    esac
done

echo "Starting Open-Codelabs Hands-on System using $CONTAINER_ENGINE..."

# Start containers in background
if [ "$CONTAINER_ENGINE" == "podman" ]; then
    podman-compose up -d
else
    docker-compose up -d
fi

echo "Containers are up!"

if [ "$TUNNEL_TYPE" == "ngrok" ]; then
    echo "Starting ngrok tunnel on port $PORT..."
    ngrok http "$PORT" --log=stdout &
    sleep 5
    PUBLIC_URL=$(curl -s http://localhost:4040/api/tunnels | grep -o 'https://[^\"]*.ngrok-free.app' | head -n 1)
elif [ "$TUNNEL_TYPE" == "bore" ]; then
    echo "Starting bore tunnel on port $PORT..."
    bore local "$PORT" --to bore.pub &
    sleep 5
    echo "Please check the bore output above for your public URL."
    PUBLIC_URL="[Check Bore Output]"
elif [ "$TUNNEL_TYPE" == "cloudflare" ]; then
    echo "Starting Cloudflare tunnel on port $PORT..."
    cloudflared tunnel --url "http://localhost:$PORT" --no-autoupdate &
    sleep 5
    echo "Please check the cloudflared output above for your public URL."
    PUBLIC_URL="[Check Cloudflared Output]"
fi

if [ -z "$PUBLIC_URL" ] || [ "$PUBLIC_URL" == "[Check Bore Output]" ] || [ "$PUBLIC_URL" == "[Check Cloudflared Output]" ]; then
    if [ "$TUNNEL_TYPE" == "ngrok" ]; then
        echo "Failed to get ngrok URL. Is ngrok running?"
    fi
else
    echo "------------------------------------------------"
    echo "Your Codelab is now PUBLIC!"
    echo "Admin Dashboard: $PUBLIC_URL/admin"
    echo "Attendee Entry:  $PUBLIC_URL"
    echo "------------------------------------------------"
fi

# Keep script running
wait
```

### Customization

Change the port:

```bash
# Edit the script
ngrok http 3000  # use 3000 instead of 5173

# Or use an env var
PORT=3000 ./run-public.sh --ngrok
```

## Workshop scenario

### Prep (day before)

1. **Local test**

```bash
# Test full stack with Docker
docker-compose up

# Check in browser
open http://localhost:5173
```

2. **Create and verify codelabs**
   - Finish all steps
   - Upload images
   - Export a backup

3. **Network test**

```bash
# Test ngrok tunnel
./run-public.sh --ngrok

# Test from another device
```

### Event day

#### 1 hour before

```bash
# Start the system
./run-public.sh --ngrok

# Check URL and prepare QR code
# Show QR code on projector
```

#### During the event

1. **Guide participants**
   - Scan the QR code or open the URL
   - Enter name and attendee code

2. **Live monitoring**
   - Monitor progress in the admin dashboard
   - Respond to help requests quickly

3. **Use chat**
   - Send announcements
   - Answer questions
   - Provide 1:1 support

#### After the event

```bash
# Feedback collection (automatic)
# Backup data
docker cp $(docker-compose ps -q backend):/app/data/sqlite.db ./backup_$(date +%Y%m%d).db

# Stop the system
docker-compose down
```

## Security considerations

### ngrok security settings

```bash
# Add Basic Auth
ngrok http 5173 --basic-auth="user:password"

# IP allowlist (paid plan)
ngrok http 5173 --cidr-allow=192.168.1.0/24
```

### Temporary URLs

Free ngrok plans generate a new URL per session:

- Pros: no access after the event
- Cons: you must re-share the URL each time

### HTTPS

ngrok provides HTTPS automatically:

```
https://abc123.ngrok-free.app  <- automatic SSL/TLS
```

## Alternatives

### localtunnel

Node.js-based alternative:

```bash
npm install -g localtunnel
lt --port 5173
```

### Tailscale

VPN-based access (more secure):

```bash
# Install and configure Tailscale
brew install tailscale
tailscale up

# Participants must also install Tailscale
```

## Performance optimization

### ngrok bandwidth

Free plan limits:

- Connections: 40/min
- Bandwidth: unlimited
- Tunnels: 1

Large events (100+ people):

- Use ngrok Pro
- Or use Cloudflare Tunnel

### Network stability

```bash
# ngrok reconnect settings
ngrok http 5173 --log=stdout --log-level=info
```

Run in the background:

```bash
# systemd service (Linux)
cat > /etc/systemd/system/codelabs-tunnel.service << EOF
[Unit]
Description=Codelabs ngrok Tunnel
After=network.target

[Service]
Type=simple
User=your_user
WorkingDirectory=/path/to/open-codelabs
ExecStart=/usr/local/bin/ngrok http 5173
Restart=always

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable codelabs-tunnel
sudo systemctl start codelabs-tunnel
```

## Monitoring

### ngrok dashboard

While ngrok is running, check [http://localhost:4040](http://localhost:4040):

- Live requests/responses
- Bandwidth usage
- Error logs

### System resources

```bash
# Real-time monitoring
docker stats

# Logs
docker-compose logs -f --tail=100
```

## Troubleshooting

### ngrok connection failure

```bash
# Check ngrok process
ps aux | grep ngrok

# Check ports
lsof -i :4040
lsof -i :5173

# Restart ngrok
killall ngrok
./run-public.sh --ngrok
```

### Cannot fetch URL

```bash
# Check ngrok API directly
curl http://localhost:4040/api/tunnels | jq .

# Open dashboard
open http://localhost:4040
```

### Slow connection

- Choose a nearby ngrok region:

```bash
ngrok http 5173 --region=jp  # Japan
ngrok http 5173 --region=ap  # Asia-Pacific
```

## Next steps

- [Firebase deployment](firebase.md) - use Firebase Hosting
- [Environment variables](environment.md) - detailed settings
- [Docker deployment](docker.md) - production deployment
- [FAQ](../faq.md) - common questions
