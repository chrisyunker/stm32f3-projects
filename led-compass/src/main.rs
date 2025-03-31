#![no_std]
#![no_main]

mod led;
use led::Leds;
mod sensor;
use sensor::*;

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use embassy_executor::Spawner;
use embassy_stm32::{
    mode::Blocking,
    gpio::{Level, Output, Pull, Speed},
};
use embassy_time::{Duration, Timer};


// LSM303AGR Registers
const LSM303AGR_ACC_ADDR: u8 = 0x19;
const LSM303AGR_MAG_ADDR: u8 = 0x1E;

// Magnetometer registers
const CFG_REG_A_M: u8 = 0x60;
const CFG_REG_C_M: u8 = 0x62;
const OUTX_L_REG_M: u8 = 0x68;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    rtt_init_print!();

    let per = embassy_stm32::init(Default::default());

    let mut lsm = Lsm303agr::new(per);
    lsm.init().await.unwrap();
    spawner.spawn(sample_mag(lsm)).unwrap();
}

#[embassy_executor::task]
async fn sample_mag(mut lsm: Lsm303agr<'static>) {
    loop {
        let m = lsm.read_magnetometer().await.unwrap();
    
        rprintln!("mag: {:?}", m);
        Timer::after(Duration::from_millis(500)).await;
    }
}