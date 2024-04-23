#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(array_chunks)]
#![feature(ascii_char)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::can::bxcan::StandardId;
use {defmt_rtt as _, panic_probe as _};
mod canlib;
use canlib::*;


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    let p = embassy_stm32::init(Default::default());

    embassy_stm32::pac::AFIO.mapr().modify(|w| w.set_can1_remap(2));
    let mut can=init_can(p.CAN,p.PB8,p.PB9);
    //let x =can.read().await;
    //println!("{}",x.unwrap().frame.data());

    let rng = init_rng(p.ADC1, p.ADC2, p.PA0, p.PA1).await;

    let ID = init_sensor_module_can(&mut can,"TEST","TEST", &rng).await;

    loop {
        sensor_check_inbox(&mut can).await;
        send_can_message(&mut can, ID, b"Hello world this is a test of the canbus transmission system.").await;
    }
}