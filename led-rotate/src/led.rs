use rtt_target::rprintln;
use stm32f3xx_hal::gpio::{gpioe, Output, PushPull};
use switch_hal::{ActiveHigh, IntoSwitch, OutputSwitch, Switch};

type Led = Switch<gpioe::PEx<Output<PushPull>>, ActiveHigh>;

#[derive(Debug)]
enum LED {
    OFF,
    N,
    NW,
    NE,
    W,
    E,
    SW,
    SE,
    S,
}

pub struct LedTask {
    pub n: Led,
    pub nw: Led,
    pub ne: Led,
    pub w: Led,
    pub e: Led,
    pub sw: Led,
    pub se: Led,
    pub s: Led,
    led_loc: LED,
}

impl LedTask {
    pub fn new<PE8Mode, PE9Mode, PE10Mode, PE11Mode, PE12Mode, PE13Mode, PE14Mode, PE15Mode>(
        pe8: gpioe::PE8<PE8Mode>,
        pe9: gpioe::PE9<PE9Mode>,
        pe10: gpioe::PE10<PE10Mode>,
        pe11: gpioe::PE11<PE11Mode>,
        pe12: gpioe::PE12<PE12Mode>,
        pe13: gpioe::PE13<PE13Mode>,
        pe14: gpioe::PE14<PE14Mode>,
        pe15: gpioe::PE15<PE15Mode>,
        moder: &mut gpioe::MODER,
        otyper: &mut gpioe::OTYPER,
    ) -> Self {
        Self {
            n: pe9
                .into_push_pull_output(moder, otyper)
                .downgrade()
                .into_active_high_switch(),
            nw: pe8
                .into_push_pull_output(moder, otyper)
                .downgrade()
                .into_active_high_switch(),
            ne: pe10
                .into_push_pull_output(moder, otyper)
                .downgrade()
                .into_active_high_switch(),
            w: pe15
                .into_push_pull_output(moder, otyper)
                .downgrade()
                .into_active_high_switch(),
            e: pe11
                .into_push_pull_output(moder, otyper)
                .downgrade()
                .into_active_high_switch(),
            sw: pe14
                .into_push_pull_output(moder, otyper)
                .downgrade()
                .into_active_high_switch(),
            se: pe12
                .into_push_pull_output(moder, otyper)
                .downgrade()
                .into_active_high_switch(),
            s: pe13
                .into_push_pull_output(moder, otyper)
                .downgrade()
                .into_active_high_switch(),
            led_loc: LED::OFF,
        }
    }

    pub fn rotate(&mut self) {
        match &mut self.led_loc {
            LED::N => {
                self.n.off().unwrap();
                self.ne.on().unwrap();
                self.led_loc = LED::NE;
            }
            LED::NE => {
                self.ne.off().unwrap();
                self.e.on().unwrap();
                self.led_loc = LED::E;
            }
            LED::E => {
                self.e.off().unwrap();
                self.se.on().unwrap();
                self.led_loc = LED::SE;
            }
            LED::SE => {
                self.se.off().unwrap();
                self.s.on().unwrap();
                self.led_loc = LED::S;
            }
            LED::S => {
                self.s.off().unwrap();
                self.sw.on().unwrap();
                self.led_loc = LED::SW;
            }
            LED::SW => {
                self.sw.off().unwrap();
                self.w.on().unwrap();
                self.led_loc = LED::W;
            }
            LED::W => {
                self.w.off().unwrap();
                self.nw.on().unwrap();
                self.led_loc = LED::NW;
            }
            LED::NW => {
                self.nw.off().unwrap();
                self.n.on().unwrap();
                self.led_loc = LED::N;
            }
            LED::OFF => {
                self.n.on().unwrap();
                self.led_loc = LED::N;
            }
        }
        rprintln!("[led] new loc: {:?}", &self.led_loc);
    }
}
