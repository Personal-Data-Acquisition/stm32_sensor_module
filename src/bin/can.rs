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

    let mut can=init_can(p.CAN,p.PA11,p.PA12);
    let interned = defmt::intern!("long string literal taking up little space");
    loop {
        send_can_message(&mut can, 0x40, b"Hello world this is a test of the canbus transmission system.").await;
    }
}
