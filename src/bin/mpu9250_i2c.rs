#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::i2c::{Error, I2c};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, peripherals};
use embassy_stm32::can::Can;
use embassy_stm32::peripherals::{CAN, I2C2};
use embassy_time::{Delay, Timer};
use mpu9250::{MargMeasurements, Mpu9250};
use {defmt_rtt as _, panic_probe as _};
mod canlib;
use canlib::*;

const ADDRESS: u8 = 0x68;
const WHOAMI: u8 = 0x0F;

bind_interrupts!(struct Irqs {
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello world!");
    let p = embassy_stm32::init(Default::default());
    //100k seems to be recommended
    let mut i2c = I2c::new(
        p.I2C2,
        p.PB10,
        p.PB11,
        Irqs,
        NoDma,
        NoDma,
        Hertz(100000),
        Default::default(),
    );

    //let mut can=init_can(p.CAN,p.PA11,p.PA12);

    //check for connection and exit on errror
    let mut data = [0u8; 1];
    match i2c.blocking_write_read(ADDRESS, &[WHOAMI], &mut data){
        Ok(()) => info!("WHOAMI: {}", data[0]),
        Err(Error::Timeout) => {
            error!("Operation timed out");
            return;
        },
        Err(e) => {
            error!("I2c Error: {:?}", e);
            return;
        },
    }

    let mpu = Mpu9250::marg_default(i2c, &mut Delay);


    match mpu {
        Ok(mut mpu) => {
            info!("mpu initialized");
            loop {
                // to get all supported measurements:
                let all: MargMeasurements<[f32; 3]> = mpu.all().unwrap();
                println!("ACCEL{:?}", all.accel);
                println!("MAG{:?}", all.mag);
                println!("GYRO{:?}", all.gyro);
                println!("TEMP{:?}\n", all.temp);
                Timer::after_millis(200).await;

            }
        }
        Err(e) => {
            match e {
                mpu9250::Error::InvalidDevice(msg) => println!("Error: Invalid Device {}", msg),
                mpu9250::Error::ModeNotSupported(_) => println!("Error: Mode Not Supported"),
                mpu9250::Error::BusError(_) => println!("Error: Bus Error"),
                mpu9250::Error::CalibrationError => println!("Error: Calibration Error"),
                mpu9250::Error::ReInitError => println!("Error: Re-initialization Error"),
                mpu9250::Error::DmpRead => println!("Error: DMP Read Error"),
                mpu9250::Error::DmpWrite => println!("Error: DMP Write Error"),
                mpu9250::Error::DmpFirmware => println!("Error: DMP Firmware Error"),
                mpu9250::Error::DmpDataNotReady => println!("Error: DMP Data Not Ready"),
                mpu9250::Error::DmpDataInvalid => println!("Error: DMP Data Invalid"),
            }

        }
    }

}
