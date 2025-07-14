#!/bin/sh

DIR="$(dirname "$0")"
ENV_FILE="$DIR/.env"
LOG="$DIR/log.txt"

echo "Launching at $(date)" >"$LOG"
echo "DIR=$DIR" >>"$LOG"

echo "Stopping Kindle framework" >>"$LOG"
/etc/init.d/framework stop

# shellcheck disable=SC1090
if [ -f "$ENV_FILE" ]; then
    echo "Sourcing $ENV_FILE" >>"$LOG"
    . "$ENV_FILE"
else
    echo "No env file found at $ENV_FILE" >>"$LOG"
fi

# Default to port 3000 if not set
PORT="${PORT:-3000}"
echo "Using port $PORT" >>"$LOG"

# Add iptables rule (safely)
iptables -C INPUT -p tcp --dport "$PORT" -j ACCEPT 2>/dev/null || {
    echo "Adding iptables rule for port $PORT" >>"$LOG"
    iptables -I INPUT -p tcp --dport "$PORT" -j ACCEPT >>"$LOG" 2>&1
}

# Run the binary
echo "Starting $DIR/bin/kiiin_frame" >>"$LOG"
"$DIR/bin/kiiin_frame" >>"$LOG" 2>&1

echo "Exited at $(date)" >>"$LOG"
