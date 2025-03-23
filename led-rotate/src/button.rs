use crate::executor::{wake_task, ExtWaker};

use core::{
    future::poll_fn,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    task::Poll,
};
use rtt_target::rprintln;
use stm32f3::stm32f303::{interrupt, Interrupt, EXTI, NVIC};

const INVALID_TASK_ID: usize = 0xFFFF_FFFF;
const DEFAULT_TASK: AtomicUsize = AtomicUsize::new(INVALID_TASK_ID);
static WAKE_TASK: AtomicUsize = DEFAULT_TASK;

static BUTTON_PRESSED: AtomicBool = AtomicBool::new(false);

pub fn button_init() {
    let exti = unsafe { &*EXTI::ptr() };
    exti.imr1.write(|w| w.mr0().set_bit());
    exti.rtsr1.write(|w| w.tr0().set_bit());

    unsafe {
        NVIC::unmask(Interrupt::EXTI0);
    }
}

pub async fn button_wait() {
    poll_fn(|cx| {
        if BUTTON_PRESSED.load(Ordering::Acquire) {
            BUTTON_PRESSED.store(false, Ordering::Release);
            Poll::Ready(())
        } else {
            WAKE_TASK.store(cx.waker().task_id(), Ordering::Relaxed);
            Poll::Pending
        }
    })
    .await
}

#[interrupt]
fn EXTI0() {
    rprintln!("[button] Button pressed interrupt");

    let exti = unsafe { &*EXTI::ptr() };

    // Clear interrupt
    exti.pr1.write(|w| w.pr0().set_bit());

    if !BUTTON_PRESSED.swap(true, Ordering::Release) {
        let task_id = WAKE_TASK.load(Ordering::Acquire);
        if task_id != INVALID_TASK_ID {
            wake_task(task_id);
        }
    } else {
        rprintln!("[button] Button already pressed, ignoring");
    }
}
