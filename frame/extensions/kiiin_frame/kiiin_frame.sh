#!/bin/sh

DIR="$(dirname "$0")"
ENV_FILE="$DIR/.env"

# shellcheck disable=SC1090
if [ -f "$ENV_FILE" ]; then
    . "$ENV_FILE"
else
    echo "No env file found at $ENV_FILE"
fi

# Default to port 3000 if not set
PORT="${PORT:-3000}"

iptables -I INPUT -p tcp --dport "$PORT" -j ACCEPT

if [ "$ISKINDLE4NT" = true ]; then
    /etc/init.d/framework stop #kindle NT4 code
else
    stop framework
    stop lab126_gui #code for kindle paperwhite3
fi

initctl stop webreader
lipc-set-prop com.lab126.powerd preventScreenSaver 1
"$DIR/bin/kiiin_frame"
