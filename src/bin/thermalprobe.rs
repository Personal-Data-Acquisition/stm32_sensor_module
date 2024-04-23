#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::dma::NoDma;

use arrform::{arrform, ArrForm};

mod canlib;
use canlib::*;


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut can=init_can(p.CAN,p.PA11,p.PA12);

    info!("Hello World!");

    let mut spi_config = Config::default();
    //recommended speed
    spi_config.frequency = Hertz(4_000_000);
    //unclear, found resource suggesting mode_1 being necessary
    spi_config.mode = embassy_stm32::spi::MODE_1;
    //using SPI2 after SPI1 seemed to be non-functional. Need to test on additional boards to be sure
    //Embassy debug console may be reusing some pins
    //peri,sck,mosi,miso
    let mut spi = Spi::new(p.SPI2, p.PB13, p.PB15, p.PB14, NoDma, NoDma, spi_config);
    //chip select
    let mut cs = Output::new(p.PB12, Level::High, Speed::VeryHigh);

    let rng = init_rng(p.ADC1, p.ADC2, p.PA0, p.PA1).await;

    let can_id = init_sensor_module_can(&mut can,"PROBE","THERMAL_PROBE", &rng).await;

    loop {
        //clear
        cs.set_high();
        //start transaction
        cs.set_low();
        // Delay slightly
        Timer::after_micros(1).await;
        //read data
        let mut buf: [u8; 2] = [0_u8; 2];
        unwrap!(spi.blocking_read(&mut buf));
        //end transaction
        cs.set_high();

        // Process the received data
        let temperature_raw = ((buf[0] as u16) << 8) | buf[1] as u16;
        let temperature_celsius = (temperature_raw >> 3) as f32 * 0.25;
        send_can_message(can,can_id,&temperature_celsius.to_le_bytes()).await;
        Timer::after_millis(200).await;

    }
}