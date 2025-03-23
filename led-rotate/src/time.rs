use crate::executor::{wake_task, ExtWaker};

use core::{
    cell::{RefCell, RefMut},
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU32, Ordering},
    task::{Context, Poll},
};
use cortex_m::peripheral::{syst::SystClkSource, SYST};
use cortex_m_rt::exception;
use critical_section::Mutex;
use heapless::{binary_heap::Min, BinaryHeap};
use rtt_target::rprintln;
use stm32f3::stm32f303::{interrupt, NVIC, TIM2};
use stm32f3xx_hal::{
    pac,
    prelude::*,
    rcc::{Clocks, APB1},
    time::duration::Milliseconds,
    timer,
};

const MAX_DEADLINES: usize = 8;
static WAKE_DEADLINES: Mutex<RefCell<BinaryHeap<(u32, usize), Min, MAX_DEADLINES>>> =
    Mutex::new(RefCell::new(BinaryHeap::new()));

fn schedule_wakeup(
    mut rm_deadlines: RefMut<BinaryHeap<(u32, usize), Min, MAX_DEADLINES>>,
    tim2: &mut timer::Timer<TIM2>,
    now: u32,
) {
    while let Some((deadline, task_id)) = rm_deadlines.peek() {
        if *deadline > now {
            let duration = *deadline - now;
            rprintln!("[timer] Next Timer: {}", duration);
            tim2.clear_event(timer::Event::Update);
            tim2.enable_interrupt(timer::Event::Update);
            tim2.start(Milliseconds::new(duration));
        } else {
            rprintln!("[timer] Invoking task: {}", *task_id);
            wake_task(*task_id);
            rm_deadlines.pop();
            continue;
        }
        break;
    }
    if rm_deadlines.is_empty() {
        rprintln!("[timer] Deadlines queue is empty");
        tim2.disable_interrupt(timer::Event::Update);
    }
}

enum TimerState {
    Init,
    Wait,
}

pub struct Timer {
    end_time: u32,
    state: TimerState,
}

impl<'a> Timer {
    pub fn new(duration: u32) -> Self {
        let end_time = Ticker::now() + duration;
        Self {
            end_time: end_time,
            state: TimerState::Init,
        }
    }

    fn register(&self, task_id: usize) {
        let new_deadline = self.end_time;
        critical_section::with(|cs| {
            let mut rm_deadlines = WAKE_DEADLINES.borrow_ref_mut(cs);
            let is_earliest = if let Some((next_deadline, _)) = rm_deadlines.peek() {
                new_deadline < *next_deadline
            } else {
                true
            };
            if rm_deadlines.push((new_deadline, task_id)).is_err() {
                panic!("Deadline dropped for task {}!", task_id);
            }

            if is_earliest {
                let mut rm_tim2 = TICKER.tim2.borrow_ref_mut(cs);
                let tim2 = rm_tim2.as_mut().unwrap();
                let now = TICKER.msec.load(Ordering::Acquire);
                schedule_wakeup(rm_deadlines, tim2, now);
            }
        });
    }
}

impl Future for Timer {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let task_id = cx.waker().task_id();
        match self.state {
            TimerState::Init => {
                rprintln!("[timer] Init: {}, end_time: {}", task_id, self.end_time);
                self.register(task_id);
                self.state = TimerState::Wait;
                Poll::Pending
            }
            TimerState::Wait => {
                if Ticker::now() > self.end_time {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

pub async fn delay(duration: u32) {
    Timer::new(duration).await;
}

static TICKER: Ticker = Ticker {
    msec: AtomicU32::new(0),
    tim2: Mutex::new(RefCell::new(None)),
};

pub struct Ticker {
    msec: AtomicU32,
    tim2: Mutex<RefCell<Option<timer::Timer<pac::TIM2>>>>,
}

impl Ticker {
    pub fn init(mut syst: SYST, tim2: TIM2, clocks: Clocks, apb1: &mut APB1) {
        let ticks_per_ms = clocks.sysclk().0 / 1_000;
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(ticks_per_ms - 1);
        syst.clear_current();
        syst.enable_counter();
        syst.enable_interrupt();

        let mut timer = timer::Timer::new(tim2, clocks, apb1);
        let timer_interrupt = timer.interrupt();

        timer.enable_interrupt(timer::Event::Update);

        critical_section::with(|cs| {
            TICKER.tim2.replace(cs, Some(timer));
        });

        unsafe {
            NVIC::unmask(timer_interrupt);
        }
    }

    pub fn now() -> u32 {
        TICKER.msec.load(Ordering::Acquire)
    }
}

#[exception]
fn SysTick() {
    TICKER.msec.fetch_add(1, Ordering::Release);
}

#[interrupt]
fn TIM2() {
    let now = TICKER.msec.load(Ordering::Acquire);
    rprintln!("[timer] [{}] timer expired", now);

    critical_section::with(|cs| {
        let mut rm_tim2 = TICKER.tim2.borrow_ref_mut(cs);
        let tim2 = rm_tim2.as_mut().unwrap();
        tim2.clear_event(timer::Event::Update);

        schedule_wakeup(WAKE_DEADLINES.borrow_ref_mut(cs), tim2, now);
    });
}
