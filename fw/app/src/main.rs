#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use core::panic::PanicInfo;
use embedded_hal::{
    digital::v2::{InputPin, OutputPin},
    prelude::_embedded_hal_adc_OneShot,
};

use app_core::{
    action::Action,
    common::Duration,
    features::charger::ChargerAction,
    state::State,
    task::{scheduler::Scheduler, FnTask, NextRun, Task},
};
use embedded_alloc::Heap;
use rp_pico::{
    entry,
    hal::pac,
    hal::{self, gpio::PinState},
};
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

    let pins = rp_pico::Pins::new(
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
            day: 7,
            day_of_week: hal::rtc::DayOfWeek::Friday,
            hour: 9,
            minute: 1,
            second: 0,
        },
    )
    .unwrap();

    let uptime = Uptime::new(core.SYST, 5);

    fn adc_f32(value: u16) -> f32 {
        const ADC_MAX: u16 = 0x0fff;
        const ADC_VREF: f32 = 3.0;

        ADC_VREF * value as f32 / ADC_MAX as f32
    }

    let mut adc = hal::Adc::new(pac.ADC, &mut pac.RESETS);
    let mut bat_v1_pin = pins.gpio26.into_floating_input();
    let mut bat_v2_pin = pins.gpio27.into_floating_input();

    let ext_power_detect_pin = pins.vbus_detect.into_floating_input();

    let mut ncharge_pin = pins.gpio18.into_push_pull_output_in_state(PinState::High);

    let pac = unsafe { pac::Peripherals::steal() };
    seg_disp_configure(&pac.IO_BANK0, &pac.SIO);

    let app_display = app_core::features::display::Display::default();
    let mut app_charger = app_core::features::charger::Charger::default();

    let mut scheduler = Scheduler::<State, Action>::new([
        Box::new(FnTask::new(move |state: &mut State| {
            let now = rtc.now().unwrap();

            state.rtc = app_core::state::RTC {
                hour: now.hour,
                minute: now.minute,
                second: now.second,
            };

            (None, NextRun::After(Duration::from_ticks(200_000)))
        })) as _,
        Box::new(app_display) as _,
        Box::new(FnTask::new(move |state: &mut State| {
            let mut sum1 = 0;
            let mut sum2 = 0;
            const N: u16 = 16;
            for _ in 0..N {
                let v1: u16 = adc.read(&mut bat_v1_pin).unwrap();
                sum1 += v1;
                let v2: u16 = adc.read(&mut bat_v2_pin).unwrap();
                sum2 += v2;
            }
            sum2 -= sum1;
            let v1 = adc_f32(sum1) / N as f32;
            let v2 = adc_f32(sum2) / N as f32;

            state.bat_voltage = (v1, v2);
            state.ext_power = ext_power_detect_pin.is_high().unwrap();

            app_charger.run(state)
        })) as _,
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
                Action::Battery(action) => match action {
                    ChargerAction::Charge => {
                        ncharge_pin.set_low().unwrap();
                    }
                    ChargerAction::Hold => {
                        ncharge_pin.set_high().unwrap();
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
