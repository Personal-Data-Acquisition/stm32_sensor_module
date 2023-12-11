#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::can::bxcan::filter::Mask32;
use embassy_stm32::can::bxcan::{Fifo, Frame, StandardId};
use embassy_stm32::can::{
    Can, CanTx, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler,
};
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::peripherals::CAN;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USB_LP_CAN1_RX0 => Rx0InterruptHandler<CAN>;
    CAN1_RX1 => Rx1InterruptHandler<CAN>;
    CAN1_SCE => SceInterruptHandler<CAN>;
    USB_HP_CAN1_TX => TxInterruptHandler<CAN>;
});

#[embassy_executor::task]
pub async fn send_can_message(tx: &'static mut CanTx<'static, 'static, CAN>) {
    loop {
        let frame = Frame::new_data(unwrap!(StandardId::new(0 as _)), [0]);
        tx.write(&frame).await;
        embassy_time::Timer::after_secs(1).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    let mut p = embassy_stm32::init(Default::default());

    // The next two lines are a workaround for testing without transceiver.
    // To synchronise to the bus the RX input needs to see a high level.
    // Use `mem::forget()` to release the borrow on the pin but keep the
    // pull-up resistor enabled.
    let rx_pin = Input::new(&mut p.PA15, Pull::Up);
    core::mem::forget(rx_pin);

    let can: &'static mut Can<'static, CAN> = static_cell::make_static!(Can::new(p.CAN, p.PA11, p.PA12, Irqs));
    can.as_mut()
        .modify_filters()
        .enable_bank(0, Fifo::Fifo0, Mask32::accept_all());
    can.set_bitrate(500_000);
    can.as_mut()
        .modify_config()
        //.set_bit_timing(0x001c0000) // http://www.bittiming.can-wiki.info/ this is 500kbps
        //.set_bit_timing(0x00050000) // http://www.bittiming.can-wiki.info/ this is 500kbps
        .set_loopback(false)
        .enable();
    //let (tx, mut rx) = can.split();

    //let tx: &'static mut CanTx<'static, 'static, CAN> = static_cell::make_static!(tx);
    //spawner.spawn(send_can_message(tx)).unwrap();
    let mut i: u8 = 0;
    let mut data_array:[u8;8] = [0,0,0,0,0,0,0,0];
    loop {
        for idx in  (0..8).rev(){
            if(data_array[idx]==254){
                data_array[idx]=0;
            }
            else {
                data_array[idx]+=1;
                break;
            }
        }

        let tx_frame = Frame::new_data(unwrap!(StandardId::new(i as _)), data_array);
        //info!("Writing");
        can.write(&tx_frame).await;
        // info!("Reading");
        //let envelope = can.read().await.unwrap();
        //println!("Received: {:?}", envelope);
        //info!("Sent");
        while(!can.is_transmitter_idle()){}
        //Timer::after_millis(10).await;
    }
}