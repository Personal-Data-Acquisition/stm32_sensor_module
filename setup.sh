#!/bin/sh
#
#Author: Jake
#Date: 2023
#Description: File to setup stuff for working on the stm32 with rust.

#check if cargo, rust and rustup are availble

# Install the tool needed to flash firmware
cargo install probe-rs --features cli

# Optional, if you want ftdi support.
cargo install probe-rs --features cli,ftdi


