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
use seg_disp::char7dp::Char7DP;

use crate::uptime::Uptime;

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

const GPIO_SEG_OFFSET: u32 = 1;
const GPIO_SEL_OFFSET: u32 = 9;

fn seg_disp_configure(io_bank0: &pac::IO_BANK0, sio: &pac::SIO) {
    for i in (GPIO_SEG_OFFSET..(GPIO_SEG_OFFSET + 8)).chain(GPIO_SEL_OFFSET..(GPIO_SEL_OFFSET + 4))
    {
        const GPIO_FUNC_SIO: u8 = 5;
        io_bank0.gpio[i as usize]
            .gpio_ctrl
            .write(|w| unsafe { w.funcsel().bits(GPIO_FUNC_SIO) });
    }
    sio.gpio_oe_set
        .write(|w| unsafe { w.bits(0xff << GPIO_SEG_OFFSET | 0x0f << GPIO_SEL_OFFSET) });
}

fn seg_disp_update(char7dp: Char7DP, digit_index: usize, sio: &pac::SIO) {
    sio.gpio_out_set.write(|w| unsafe {
        w.bits(
            (char7dp.state() as u32) << GPIO_SEG_OFFSET
                | (1 << digit_index & 0x0f) << GPIO_SEL_OFFSET,
        )
    });
    sio.gpio_out_clr.write(|w| unsafe {
        w.bits(
            (!char7dp.state() as u32) << GPIO_SEG_OFFSET
                | (!(1 << digit_index) & 0x0f) << GPIO_SEL_OFFSET,
        )
    });
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
