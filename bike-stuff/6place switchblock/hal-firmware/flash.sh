#!/bin/sh
if [ -z "$1" ]; then
	echo "usage: $0 <bin>"
	exit 1
fi

if [ -z "$PORT" ]; then
  echo 'set $PORT'
  exit 1
fi

FEATURES=""
if [ -n "$2" ]; then
  FEATURES="--features $2"
fi

set -xe

cargo build --bin "$1" $FEATURES
avrdude -patmega328p -carduino -P"$PORT" -b115200 -D -Uflash:w:target/avr-atmega328p/debug/"$1".elf:e
ravedude nano -P "$PORT" -c -b 57600
