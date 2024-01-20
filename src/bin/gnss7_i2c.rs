#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use defmt::export::slice;
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::i2c::{Error, I2c, Instance};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, peripherals, Peripherals};
use embassy_stm32::rcc::low_level::RccPeripheral;
use embassy_stm32::usart::{BasicInstance, RxDma, Uart};
use embassy_time::Timer;
use heapless::String;
use {defmt_rtt as _, panic_probe as _};
use ublox::*;
use ublox::{CfgPrtI2cBuilder, I2cPortId, DataBits, Parity, StopBits, InProtoMask, OutProtoMask};
const ADDRESS: u8 = 0x42;
const WHOAMI: u8 = 0x0F;

bind_interrupts!(struct Irqs {
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});

async fn checkGNSS<'d, T: Instance, TXDMA, RXDMA>(i2c: &mut I2c<'d, T, TXDMA, RXDMA>) -> Result<(), Error> {
    // 0xFD (MSB) and 0xFE (LSB) are the registers that contain number of bytes available
    // 0xFF contains gps data stream.
    let mut avail=[0u8;2];
    i2c.blocking_write_read(0x42,&[0xfd],&mut avail);
    let mut bytes_available = (avail[0] as usize) << 8 | avail[1] as usize;
    info!("Bytes available:{}",bytes_available);
    while(bytes_available>0){
        //read in 256 byte chunks
        let to_read = if bytes_available > 256 { 256 } else { bytes_available };
        bytes_available-=to_read;
        let mut arr=[0u8;256];
        //use slice to get around not having vectors
        let mut slice: &mut [u8] = &mut arr[0..to_read];
        //read
        i2c.blocking_read(0x42, &mut slice);
        //print as string
        let mut txt=['\0';256];
        let txt = core::str::from_utf8(slice).unwrap();
        //let txt = unsafe { core::str::from_utf8_unchecked(slice) };
        info!("{}",txt);

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
    //check for connection and exit on errror
    let mut data = [0u8; 1];
    match i2c.blocking_write_read(ADDRESS, &[WHOAMI], &mut data) {
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


    //prepare parser with fixed buffer
    let mut buf = [0; 256];
    let buf = ublox::FixedLinearBuffer::new(&mut buf[..]);
    let mut parser = ublox::Parser::new(buf);

    while (true){
        checkGNSS(&mut i2c).await;
        Timer::after_millis(250).await;
    }
}
