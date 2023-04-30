#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use core::panic::PanicInfo;

use app_core::{
    action::Action,
    scheduler::Scheduler,
    state::State,
    task::{FnTask, NextRun},
};
use embedded_alloc::Heap;
use rp_pico::{entry, hal, hal::pac};
use rtt_target::{rprintln, rtt_init_print};

use crate::{
    display::{seg_disp_configure, seg_disp_update},
    uptime::Uptime,
};

mod display;
mod uptime;
mod uptime_delay;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    init_heap();

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = hal::Sio::new(pac.SIO);

    let _pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let rtc = hal::rtc::RealTimeClock::new(
        pac.RTC,
        clocks.rtc_clock,
        &mut pac.RESETS,
        hal::rtc::DateTime {
            year: 2023,
            month: 4,
            day_of_week: hal::rtc::DayOfWeek::Friday,
            day: 7,
            hour: 9,
            minute: 1,
            second: 0,
        },
    )
    .unwrap();

    let uptime = Uptime::new(core.SYST, 5);

    let pac = unsafe { pac::Peripherals::steal() };

    seg_disp_configure(&pac.IO_BANK0, &pac.SIO);

    let app_display = app_core::features::display::Display::default();

    let mut scheduler = Scheduler::<State, Action>::new([
        Box::new(FnTask::new(move |state: &mut State| {
            let now = rtc.now().unwrap();

            state.rtc = app_core::state::RTC {
                hour: now.hour,
                minute: now.minute,
                second: now.second,
            };

            (None, NextRun::InOrder)
        })) as _,
        Box::new(app_display) as _,
    ]);

    let mut state = State::default();

    loop {
        if let Some(action) = scheduler.run(uptime.get_instant(), &mut state) {
            match action {
                Action::Display(action) => match action {
                    seg_disp::disp::Action::Render(c, i) => {
                        seg_disp_update(c, i, &pac.SIO);
                    }
                },
            }
        }
    }
}

#[global_allocator]
static HEAP: Heap = Heap::empty();

fn init_heap() {
    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 64 * 1024;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    loop {
        cortex_m::asm::wfi();
    }
}
