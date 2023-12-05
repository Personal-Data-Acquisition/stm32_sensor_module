# stm32_sensor_module
A stm32 based sensor module controller.

## Directory Structure

## How to run

To setup the physical hardware for this you'll need a stm32f103C8T6, or 
just a stm32 bluepill.

From there you need to connect a debugger/programmer via the pins on the end of
the board.

**3v3:** The VCC or V+ of 3.3 volts.
**Gnd:** Ground on both
**SWCLK:** The data clock source.
**SWIO:** Data.

These should be connected from the programmer to the development board.


```sh

# You don't need to actually build first but eh not a bad move to.
cargo build --bin blinky --release

# Need the prope-rs program to actully run it.
cargo run --bin blinky --release
```



## How to contribute


