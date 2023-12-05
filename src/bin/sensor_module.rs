/*
 * Author: Jake, <PUT NAME HERE>
 * Date: 2023
 * Filename: sensor_module.rs
 */

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

//use embassy_time::Duration;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut led = Output::new(p.PC13, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(300).await;

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;
    }
}

//
//#[embassy_executor::task]
//async fn blinker(mut led: Output<'static, P0_13>, interval: Duration) {
//    loop {
//        led.set_high();
//        Timer::after(interval).await;
//        led.set_low();
//        Timer::after(interval).await;
//    }
//}
//
