#!/bin/bash

DIR="$(dirname "$0")"
ENV_FILE="$DIR/.env"
LOG="$DIR/log.txt"

exec > >(tee -a "$LOG") 2>&1

echo "Launching Kiiin Frame at $(date)"

# shellcheck disable=SC1090
if [ -f "$ENV_FILE" ]; then
    . "$ENV_FILE"
else
    echo "No env file found at $ENV_FILE"
fi

# Default to port 3000 if not set
PORT="${PORT:-3000}"
echo "Using port $PORT"

echo "Adding iptables rule for port $PORT"
iptables -I INPUT -p tcp --dport "$PORT" -j ACCEPT

echo "Kill GUI"
stop lab126_gui
stop webreader

echo "Disabling screensaver"
lipc-set-prop com.lab126.powerd preventScreenSaver 1

echo "Starting kiiin_frame"
"$DIR/kiiin_frame"
