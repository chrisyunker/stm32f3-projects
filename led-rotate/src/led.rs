use embassy_stm32::gpio::Output;
use embassy_time::Timer;
use rtt_target::rprintln;

use crate::ButtonEvent;

pub struct Leds<'a> {
    leds: [Output<'a>; 8],
    direction: i8,
    current_led: usize,
}

impl<'a> Leds<'a> {
    pub fn new(leds: [Output<'a>; 8]) -> Self {
        Self {
            leds: leds,
            direction: 1,
            current_led: 0,
        }
    }

    pub fn reverse_direction(&mut self) {
        self.direction *= -1;
        rprintln!("[led] new direction: {}", self.direction);
    }

    pub fn rotate(&mut self) {
        if self.direction > 0 {
            self.current_led = (self.current_led + 1) & 7;
        } else {
            self.current_led = (8 + self.current_led - 1) & 7;
        }
        rprintln!("[led] new led: {}", &self.current_led);
    }

    pub fn set_low(&mut self) {
        self.leds[self.current_led].set_low();
    }

    pub fn set_high(&mut self) {
        self.leds[self.current_led].set_high();
    }

    async fn flash(&mut self) {
        for _ in 0..3 {
            for led in &mut self.leds {
                led.set_high();
            }
            Timer::after_millis(500).await;
            for led in &mut self.leds {
                led.set_low();
            }
        }
        Timer::after_millis(200).await;
    }

    pub async fn process_event(&mut self, event: ButtonEvent) {
        match event {
            ButtonEvent::SingleClick => {
                self.rotate();
            }
            ButtonEvent::DoubleClick => {
                self.reverse_direction();
                self.rotate();
            }
            ButtonEvent::Hold => {
                self.flash().await;
            }
        }
    }
}
