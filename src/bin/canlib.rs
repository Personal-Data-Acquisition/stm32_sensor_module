use core::ops::Deref;
use arrform::ArrForm;
use core::pin::pin;
use arrform::arrform;
use defmt::*;
use embassy_stm32::{bind_interrupts};
use embassy_stm32::adc::{Adc, AdcPin};
use embassy_stm32::can::bxcan::filter::Mask32;
use embassy_stm32::can::bxcan::{Fifo, Frame, Id, StandardId};
use embassy_stm32::can::{RxPin, TxPin, Can, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler};
use embassy_stm32::peripherals::{ADC1, ADC2, CAN};
use embassy_time::{Delay, Timer};

bind_interrupts!(struct Irqs {
USB_LP_CAN1_RX0 => Rx0InterruptHandler<CAN>;
CAN1_RX1 => Rx1InterruptHandler<CAN>;
CAN1_SCE => SceInterruptHandler<CAN>;
USB_HP_CAN1_TX => TxInterruptHandler<CAN>;
});

unsafe extern "C" fn HardFault() {
    cortex_m::peripheral::SCB::sys_reset();
}

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
        //0xfe: server config id
        //0x7ff: mask for match all
        .enable_bank(0, Fifo::Fifo0, Mask32::frames_with_std_id(
            unwrap!(StandardId::new(0xfe as _)),unwrap!(StandardId::new(0x7ff as _))));
    can.set_bitrate(1_000_000);
    can.as_mut()
        .modify_config()
        //.set_bit_timing(0x001c0000) // http://www.bittiming.can-wiki.info/ this is 500kbps
        //.set_bit_timing(0x00050000) // http://www.bittiming.can-wiki.info/ this is 500kbps
        .set_loopback(false)
        .enable();
    return can;
}

pub async fn init_rng<P1: AdcPin<ADC1>, P2: AdcPin<ADC2>>(adc1: ADC1, adc2: ADC2, pin1:P1, pin2:P2) -> u32 {
    let mut adc1 = Adc::new(adc1, &mut Delay);
    let mut adc2 = Adc::new(adc2, &mut Delay);
    let mut pin1 = pin1;
    let mut pin2 = pin2;
    let rng:u32 = (adc1.read(&mut pin1).await)as u32 * (adc2.read(&mut pin2).await) as u32;
    return rng
}

pub async fn init_sensor_module_can(can: &mut Can<'_, CAN>,ID:&str,TYPE:&str,rng:&u32)->u8{

    //get random source from adc

    //stagger init based on rng
    Timer::after_millis((rng & 0xff) as u64).await;

    let af = arrform!(64, "{:07}", rng);

    //send random rumber as id
    send_can_message(can,0xff,af.as_bytes()).await;
    loop {
        let msg = if let Some(f) = sensor_check_inbox(can).await {
            f
        } else {
            continue;
        };
        //not talking to me
        if af.as_str().as_bytes()!=(msg.data().unwrap().deref()) {
            continue
        }
        //send id data
        //get config information
        let msg=can.read().await.unwrap().frame;
        let id=msg.data().unwrap()[0];

        let cfg = arrform!(64, "ID:{},TYPE:{}\n", ID, TYPE);
        println!("sending \"{}\"",cfg.as_str());
        send_can_message(can, id, cfg.as_bytes()).await;

        //send ok
        //send_can_message(can, id, &[0]).await;

        return id
    }
}

pub async fn sensor_check_inbox(can: &mut Can<'_, CAN>) -> Option<Frame> {
    while let Ok(frame)=can.try_read(){
        println!("recieved from server {}",frame.frame.data());
        match frame.frame.data().unwrap()[0] {
            5 => {
                info!("RESETTING");
                //give message time to flush
                Timer::after_millis(100).await;
                unsafe{HardFault()}
            }
            _ => {return Some(frame.frame.clone())}
        }
    }
    return None;
}



//just here to appease the ide
fn main() {}