#!/bin/sh
# Name: kiiin_frame

DIR="$(dirname "$0")"

lipc-set-prop com.lab126.cmd wirelessEnable 1
iw dev wlan0 set power_save off
iw dev wlan0 get power_save
iptables -I INPUT -p tcp --dport 3000 -j ACCEPT

stop lab126_gui
stop webreader

lipc-set-prop com.lab126.powerd preventScreenSaver 1

date

exec "$DIR/kiiin_frame"
