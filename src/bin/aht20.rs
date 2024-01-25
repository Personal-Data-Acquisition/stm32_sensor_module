#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::i2c::{Error, I2c, Instance};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, peripherals};
use embassy_time::Timer;

use sensor_lib_aht20 as aht20;

const ADDRESS: u8 = 0x42;
const WHOAMI: u8 = 0x0F;

// This is the standard mode speed as defined by the i2c standard.
pub const I2C_STANDARD_MODE_SPEED = Hertz(100000);


bind_interrupts!(struct Irqs {
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});



#[embassy_executor::main]
async fn main(_spawner: Spawner) { 
    let p = embassy_stm32::init(Default::default());
   
    let mut i2c = I2c::new(
        p.I2C1,
        p.PB10,
        p.PB11,
        Irqs,
        NoDma,
        NoDma,
        I2C_STANDARD_MODE_SPEED,
        Default::default(),
    );

    let mut sensor_instance = aht20::Sensor::new(i2c, aht20::SENSOR_ADDR);
   
    //This could impliment an error handler on the return type if we
    //standardized on how to report errors over the can bus.
    let mut inited_sensor = sensor_instance.init(&mut delay).unwrap();
    
    let mut sd = inited_sensor.read_sensor(&mut delay).unwrap();

    //infinite loop
    loop {
        info!("Humidity: {}, Temp(C): {}", 
              sd.calculate_humidity(), 
              sd.calculate_temperature()
              );
    

        delay.delay_ms(250 as u16);
        sd = inited_sensor.read_sensor(&mut delay).unwrap();
    }

}

