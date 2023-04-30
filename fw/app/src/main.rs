#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use core::panic::PanicInfo;

use app_core::{
    action::Action,
    common::Duration,
    scheduler::Scheduler,
    state::State,
    task::{FnTask, NextRun},
};
use embedded_alloc::Heap;
use rp_pico::{entry, hal, hal::pac};
use rtt_target::{rprintln, rtt_init_print};
use seg_disp::{char7dp::Char7DP, char7dp_seq::Char7DPSeq};

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

    let mut disp = seg_disp::disp::Disp::<4>::new(Duration::from_ticks(2_000), 1.0);

    let mut scheduler =
        Scheduler::<State, Action>::new([Box::new(FnTask::new(move |_: &mut State| {
            let now = rtc.now().unwrap();
            let time = hhmm_to_char7dp_array(now.hour, now.minute, now.second);
            disp.set_chars(time);

            let (action, delay) = disp.run();

            (Some(Action::Display(action)), NextRun::After(delay))
        })) as _]);

    let mut state = State {};

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

fn hhmm_to_char7dp_array(hour: u8, minute: u8, second: u8) -> [Char7DP; 4] {
    let mut time = [Char7DP::space(); 4];

    Char7DPSeq::new(&mut time[0..2]).set_dec(minute as usize, true);
    Char7DPSeq::new(&mut time[2..4]).set_dec(hour as usize, false)[0].set_dp(second & 1 == 0);

    time
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
