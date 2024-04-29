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
use half::f16;

const ADDRESS: u8 = 0x68;
const WHOAMI: u8 = 0x0F;

bind_interrupts!(struct Irqs {
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});

async fn send_reading(can: &mut Can<'_, CAN>, id: u8, label:char, data:[f32;3]){
    let mut bytes: [u8; 7] = [label as u8, 0,0,0,0,0,0]; // Initialize a fixed-size array to hold the bytes
    let mut index = 1;
    for &f in data.iter() {
        // Convert f32 to f16
        let f16_value = f16::from_f32(f);
        // Extract the bytes of the f16 value
        let f16_bytes: [u8; 2] = f16_value.to_bits().to_le_bytes();
        // Copy the bytes into the existing array
        bytes[index..index + 2].copy_from_slice(&f16_bytes);
        // Update the index
        index += 2;
    }
    send_can_message(can, id, &bytes).await;
}

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

    embassy_stm32::pac::AFIO.mapr().modify(|w| w.set_can1_remap(2));
    let mut can=init_can(p.CAN,p.PB8,p.PB9);

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

    let rng = init_rng(p.ADC1, p.ADC2, p.PA0, p.PA1).await;

    let can_id = init_sensor_module_can(&mut can,"ACCELEROMETER","ACC_MPU9250", &rng).await;

    match mpu {
        Ok(mut mpu) => {
            info!("mpu initialized");
            loop {
                sensor_check_inbox(&mut can).await;
                // to get all supported measurements:
                let all: MargMeasurements<[f32; 3]> = mpu.all().unwrap();
                println!("ACCEL{:?}", all.accel);
                println!("MAG{:?}", all.mag);
                println!("GYRO{:?}", all.gyro);
                println!("TEMP{:?}\n", all.temp);

                send_reading(can,can_id,'a',all.accel).await;
                send_reading(can,can_id,'m',all.mag).await;
                send_reading(can,can_id,'g',all.gyro).await;

                Timer::after_millis(1).await;

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
