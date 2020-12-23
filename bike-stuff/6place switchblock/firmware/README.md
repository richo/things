# switchblock

Firmware for any small atmega core for my switchblock. I used an arduino nano clone.

## Prerequisites

  * A recent version of the nightly Rust compiler. Anything including or greater than `rustc 1.47.0-nightly (0820e54a8 2020-07-23)` can be used.
  * The rust-src rustup component - `$ rustup component add rust-src`
  * AVR-GCC on the system for linking
  * AVR-Libc on the system for support libraries

## Usage

Now to build, run:

```bash
# build
cargo build -Z build-std=core --target avr-atmega328p.json --release
# flash
avrdude -patmega328p -carduino -PCOM4 -b115200 -D -Uflash:w:target/avr-atmega328p/release/blink.elf:e

```

## Acknowledgements

This is derived from `blink` by Dylan McKay, the original source can be found [here](orig).

[orig]: https://github.com/avr-rust/blink

