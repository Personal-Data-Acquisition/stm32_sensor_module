#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(array_chunks)]

use defmt::*;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};
mod canlib;
use canlib::*;


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    let p = embassy_stm32::init(Default::default());

    embassy_stm32::pac::AFIO.mapr().modify(|w| w.set_can1_remap(2));
    let mut can=init_can(p.CAN,p.PB8,p.PB9);

    loop {
        send_can_message(&mut can, 0x40, b"Hello world this is a test of the canbus transmission system.").await;
    }
}