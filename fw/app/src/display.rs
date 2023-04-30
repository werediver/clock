use rp_pico::pac;
use seg_disp::char7dp::Char7DP;

const GPIO_SEG_OFFSET: u32 = 1;
const GPIO_SEL_OFFSET: u32 = 9;

pub fn seg_disp_configure(io_bank0: &pac::IO_BANK0, sio: &pac::SIO) {
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

pub fn seg_disp_update(char7dp: Char7DP, digit_index: usize, sio: &pac::SIO) {
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
