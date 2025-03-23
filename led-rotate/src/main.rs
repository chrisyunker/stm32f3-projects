#![no_std]
#![no_main]

mod button;
mod channel;
mod executor;
mod led;
mod time;

use button::{button_init, button_wait};
use channel::{Channel, Receiver, Sender};
use led::LedTask;
use time::Ticker;

use core::pin::pin;
use cortex_m_rt::entry;
use futures::{select_biased, FutureExt};
use panic_rtt_target as _;
use rtt_target::rtt_init_print;
use stm32f3xx_hal::{gpio::gpioe::Parts, pac, prelude::*};

const DEBOUNCE_MSEC: u32 = 100;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .use_pll()
        .pclk1(24.MHz())
        .pclk2(24.MHz())
        .freeze(&mut flash.acr);

    Ticker::init(cp.SYST, dp.TIM2, clocks, &mut rcc.apb1);

    let channel = Channel::new();

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    gpioa
        .pa0
        .into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr);

    let gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let led_task = pin!(async_led_task(gpioe, channel.get_receiver(),));

    button_init();
    let button_task = pin!(async_button_task(channel.get_sender()));

    executor::run_tasks(&mut [led_task, button_task]);
}

async fn async_led_task(mut gpioe: Parts, mut receiver: Receiver<'_, bool>) {
    let mut led = LedTask::new(
        gpioe.pe8,
        gpioe.pe9,
        gpioe.pe10,
        gpioe.pe11,
        gpioe.pe12,
        gpioe.pe13,
        gpioe.pe14,
        gpioe.pe15,
        &mut gpioe.moder,
        &mut gpioe.otyper,
    );
    loop {
        select_biased! {
            _ = receiver.receive().fuse() => {
                led.rotate();
            }
        }
    }
}

async fn async_button_task(sender: Sender<'_, bool>) {
    loop {
        button_wait().await;
        sender.send(true);
        time::delay(DEBOUNCE_MSEC).await;
    }
}
