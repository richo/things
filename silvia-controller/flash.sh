#!/bin/sh
if [ -z "$1" ]; then
	echo "usage: $0 <bin>"
	exit 1
fi

cargo build --bin "$1" && avrdude -patmega328p -carduino -P/dev/tty.usbserial-110 -b115200 -D -Uflash:w:target/avr-atmega328p/debug/"$1".elf:e && ravedude nano -P /dev/tty.usbserial-110 -c -b 57600
