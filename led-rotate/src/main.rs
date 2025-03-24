#![no_std]
#![no_main]

mod led;
use led::Leds;

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use embassy_executor::Spawner;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Level, Output, Pull, Speed},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::{with_timeout, Duration};

const DOUBLE_CLICK_DELAY: u64 = 250;
const HOLD_DELAY: u64 = 1000;

static CHANNEL: Channel<ThreadModeRawMutex, ButtonEvent, 4> = Channel::new();

#[derive(Debug)]
pub enum ButtonEvent {
    Hold,
    SingleClick,
    DoubleClick,
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    rtt_init_print!();

    let p = embassy_stm32::init(Default::default());

    let leds = [
        Output::new(p.PE8, Level::Low, Speed::Low),
        Output::new(p.PE9, Level::Low, Speed::Low),
        Output::new(p.PE10, Level::Low, Speed::Low),
        Output::new(p.PE11, Level::Low, Speed::Low),
        Output::new(p.PE12, Level::Low, Speed::Low),
        Output::new(p.PE13, Level::Low, Speed::Low),
        Output::new(p.PE14, Level::Low, Speed::Low),
        Output::new(p.PE15, Level::Low, Speed::Low),
    ];

    let leds = Leds::new(leds);
    spawner.spawn(blink(leds)).unwrap();

    let button = ExtiInput::new(p.PA0, p.EXTI0, Pull::Down);
    spawner.spawn(button_task(button)).unwrap();
}

#[embassy_executor::task]
async fn button_task(mut button: ExtiInput<'static>) {
    button.wait_for_rising_edge().await;
    loop {
        if with_timeout(
            Duration::from_millis(HOLD_DELAY),
            button.wait_for_falling_edge(),
        )
        .await
        .is_err()
        {
            rprintln!("[button] Hold");
            CHANNEL.send(ButtonEvent::Hold).await;
            button.wait_for_falling_edge().await;
        } else if with_timeout(
            Duration::from_millis(DOUBLE_CLICK_DELAY),
            button.wait_for_rising_edge(),
        )
        .await
        .is_err()
        {
            rprintln!("[button] Single click");
            CHANNEL.send(ButtonEvent::SingleClick).await;
        } else {
            rprintln!("[button] Double click");
            CHANNEL.send(ButtonEvent::DoubleClick).await;
            button.wait_for_falling_edge().await;
        }
        button.wait_for_rising_edge().await;
    }
}

#[embassy_executor::task]
async fn blink(mut leds: Leds<'static>) {
    loop {
        leds.set_high();
        if let Ok(event) = with_timeout(Duration::from_millis(500), CHANNEL.receive()).await {
            leds.set_low();
            leds.process_event(event).await;
        }
    }
}
