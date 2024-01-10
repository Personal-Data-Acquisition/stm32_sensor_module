# stm32_sensor_module
A stm32 based sensor module controller.

## Directory Structure


## Using the sub-modules

The repo uses git sub-modules that can be selectively pulled into the project.

Each of the sub-modules is a library crate essentially and has it's own unit 
and integration tests.

For example if you want to pull the sub-modules current code:

```sh
git submodule --init <NAMEHERE>
```

### Running the tests in the sub-modules

```sh
cd ./sensor_lib_sn65hvd230
cargo test --target x86_64-unknown-linux-gnu

Finished test ...
Running unittests src/lib.rs ...
//SNIP
```

### Adding a sub-module to the dependencies

To add a git sub-module to the sensor modules dependencies add it to the
array in the `Cargo.toml` file.

```toml
[workspace]
members = ["sensor_lib_sn65hvd230", <PUT NEW ONE HERE>]

```


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

# Need the prope-rs program to actually run it.
cargo run --bin blinky --release
```





## How to contribute


