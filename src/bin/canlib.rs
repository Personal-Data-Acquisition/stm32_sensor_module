

use defmt::*;
use embassy_stm32::{bind_interrupts};
use embassy_stm32::can::bxcan::filter::Mask32;
use embassy_stm32::can::bxcan::{Fifo, Frame, StandardId};
use embassy_stm32::can::{RxPin, TxPin, Can, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler};
use embassy_stm32::peripherals::CAN;

bind_interrupts!(struct Irqs {
USB_LP_CAN1_RX0 => Rx0InterruptHandler<CAN>;
CAN1_RX1 => Rx1InterruptHandler<CAN>;
CAN1_SCE => SceInterruptHandler<CAN>;
USB_HP_CAN1_TX => TxInterruptHandler<CAN>;
});

//#[embassy_executor::task]
pub async fn send_can_message(can: &mut Can<'_, CAN>, id: u8, data: &[u8]) {
    let std_id = unwrap!(StandardId::new(id as _));
    for chunk in data.chunks(8) {
        let frame = match chunk.len() {
            1 => Frame::new_data(std_id, [chunk[0]; 1]),
            2 => Frame::new_data(std_id, [chunk[0], chunk[1]]),
            3 => Frame::new_data(std_id, [chunk[0], chunk[1], chunk[2]]),
            4 => Frame::new_data(std_id, [chunk[0], chunk[1], chunk[2], chunk[3]]),
            5 => Frame::new_data(std_id, [chunk[0], chunk[1], chunk[2], chunk[3], chunk[4]]),
            6 => Frame::new_data(std_id, [chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5]]),
            7 => Frame::new_data(std_id, [chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6]]),
            _ => Frame::new_data(std_id, [chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7]]),
        };
        can.write(&frame).await;
        while !can.is_transmitter_idle() {}
    }
}

pub fn init_can<R: RxPin<CAN>, T: TxPin<CAN>>(can: CAN,rx_pin: R, tx_pin: T) -> &'static mut Can<'static, CAN> {
    let can: &'static mut Can<'static, CAN> = static_cell::make_static!(Can::new(can, rx_pin, tx_pin, Irqs));
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
    return can;
}
//just here to appease the ide
fn main() {}