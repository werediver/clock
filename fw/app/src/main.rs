#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

use core::{alloc::Layout, panic::PanicInfo};

use embedded_alloc::Heap;
use hal::Clock;
use rp_pico::{entry, hal, hal::pac};
use rtt_target::{rprintln, rtt_init_print};
use seg_disp::Char7DP;

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

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let mut digit_index = 0u8;

    seg_disp_configure(&pac.IO_BANK0, &pac.SIO);

    loop {
        let char7dp = Char7DP::try_from_u8(4 - digit_index).unwrap();
        let char7dp = if digit_index == 2 {
            char7dp.with_dp()
        } else {
            char7dp
        };

        seg_disp_update(char7dp, digit_index, &pac.SIO);

        digit_index = (digit_index + 1) % 4;

        delay.delay_ms(5);
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

fn seg_disp_update(char7dp: Char7DP, digit_index: u8, sio: &pac::SIO) {
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

#[alloc_error_handler]
fn oom(layout: Layout) -> ! {
    rprintln!(
        "failed to allocate {} bytes aligned on {} bytes)",
        layout.size(),
        layout.align()
    );
    loop {
        cortex_m::asm::wfi();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    loop {
        cortex_m::asm::wfi();
    }
}
