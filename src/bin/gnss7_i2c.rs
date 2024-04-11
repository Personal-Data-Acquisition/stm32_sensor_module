#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::time::Duration;
use cortex_m::asm::delay;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::i2c::{Error, I2c};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, peripherals};
use embassy_stm32::adc::Adc;
use embassy_stm32::can::Can;
use embassy_stm32::peripherals::{CAN, I2C2};
use embassy_time::{Delay, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};
mod canlib;
use canlib::*;

const ADDRESS: u8 = 0x42;
const WHOAMI: u8 = 0x0F;

bind_interrupts!(struct Irqs {
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});

async fn check_gnss(ID:u8, i2c: &mut I2c<'_, I2C2>, can: &mut Can<'_, CAN>,) -> Result<(), Error> {
    // 0xFD (MSB) and 0xFE (LSB) are the registers that contain number of bytes available
    // 0xFF contains gps data stream.
    let mut avail=[0u8;2];
    i2c.blocking_write_read(0x42,&[0xfd],&mut avail).unwrap();
    let mut bytes_available = (avail[0] as usize) << 8 | avail[1] as usize;
    println!("Bytes available:{}",bytes_available);
    while bytes_available>0 {
        //read in 256 byte chunks
        let to_read = if bytes_available > 256 { 256 } else { bytes_available };
        bytes_available-=to_read;
        let mut arr=[0u8;256];
        //use slice to get around not having vectors
        let mut slice: &mut [u8] = &mut arr[0..to_read];
        //read
        i2c.blocking_read(0x42, &mut slice).unwrap();
        //print as string
        let txt = core::str::from_utf8(slice).unwrap();
        //let txt = unsafe { core::str::from_utf8_unchecked(slice) };
        send_can_message(can, ID, &slice).await;
        println!("{}",txt);

    }
    Ok(())
}


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello world!");
    let p = embassy_stm32::init(Default::default());
    //115200 seems to be recommended
    let mut i2c = I2c::new(
        p.I2C2,
        p.PB10,
        p.PB11,
        Irqs,
        NoDma,
        NoDma,
        Hertz(115200),
        Default::default(),
    );

    embassy_stm32::pac::AFIO.mapr().modify(|w| w.set_can1_remap(2));
    let mut can=init_can(p.CAN,p.PB8,p.PB9);

    //check for connection and exit on errror
    let mut data = [0u8; 1];
    match i2c.blocking_write_read(ADDRESS, &[WHOAMI], &mut data) {
        Ok(()) => println!("WHOAMI: {}", data[0]),
        Err(Error::Timeout) => {
            error!("Operation timed out");
            return;
        },
        Err(e) => {
            error!("I2c Error: {:?}", e);
            return;
        },
    }

    let ID = init_sensor_module_can(&mut can,"GNSS7","GPS", p.ADC1, p.ADC2, p.PA0, p.PA1).await;


    loop{
        check_gnss(ID, &mut i2c, &mut can).await.unwrap();
        Timer::after_millis(250).await;
    }
}
