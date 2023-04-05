#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

use core::{alloc::Layout, panic::PanicInfo};

use embedded_alloc::Heap;
use hal::Clock;
use rp_pico::{entry, hal, hal::pac};
use rtt_target::{rprintln, rtt_init_print};
use seg_disp::char7dp::Char7DP;

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

    let sio = hal::Sio::new(pac.SIO);

    let _pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let pac = unsafe { pac::Peripherals::steal() };

    seg_disp_configure(&pac.IO_BANK0, &pac.SIO);

    let mut i = 0usize;

    let mut disp = seg_disp::disp::Disp::<4>::default();
    loop {
        disp.set_chars(dec_to_char7dp(i / 50));

        match disp.run() {
            seg_disp::disp::Action::Render(c, i) => {
                seg_disp_update(c, i, &pac.SIO);
            }
        }

        (i, _) = i.overflowing_add(1);

        delay.delay_ms(5);
    }
}

fn dec_to_char7dp<const N: usize>(n: usize) -> [Char7DP; N] {
    let mut chars = [Char7DP::space(); N];

    const fn gen_exp10<const N: usize>() -> [usize; N] {
        let mut exp10 = [0usize; N];

        let mut i = 0;
        let mut value = 1;
        while i < N {
            exp10[i] = value;
            if i < N - 1 {
                value *= 10;
            }
            i += 1;
        }

        exp10
    }

    struct Const<const N: usize> {}

    impl<const N: usize> Const<N> {
        const EXP10: [usize; N] = gen_exp10();
    }

    for i in 0..N {
        let p = n / Const::<N>::EXP10[i];
        chars[i] = if p > 0 || i == 0 {
            let q = (p % 10) as u8;
            Char7DP::try_from_u8(q).unwrap()
        } else {
            Char7DP::space()
        }
    }

    chars
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
