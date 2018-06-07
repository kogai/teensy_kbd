#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(asm)]

extern crate bit_field;
extern crate volatile;

mod clock;
mod ocs;
mod port;
mod sim;
mod uart;
mod watchdog;

use clock::Mcg;
use core::fmt::Write;
use port::{Port, PortName};
use sim::{ClockGate, Sim};
use watchdog::Watchdog;

extern "C" fn main() -> ! {
    let (wdog, sim, mcg, osc, pin) = unsafe {
        (
            Watchdog::new(),
            Sim::new(),
            Mcg::new(),
            ocs::Osc::new(),
            Port::new(PortName::C).pin(5),
        )
    };

    wdog.disable();
    osc.enable(clock::TEENSY_32_CAPACITANCE);
    sim.enable_clock_gate(ClockGate::PortC);
    /*
     * Set the dividers for the various clocks:
     *      Core: 72Mhz
     *      Bus: 36Mhz
     *      Flash: 24Mhz
     */
    // TODO: set the USB divider
    sim.set_dividers(1, 2, 3);

    // We can now move the MCG to using the external oscillator
    if let clock::Clock::Fei(mut fei) = mcg.clock() {
        // Our 16MHz xtal is "very fast", and needs to be divided
        // by 512 to be in the acceptable FLL range
        fei.enable_xtal(clock::OscRange::VeryHigh);
        let fbe = fei.use_external(512);

        // PLL is 27/6 * xtal == 72MHz
        let pbe = fbe.enable_pull(27, 6);
        pbe.use_pll();
    } else {
        panic!("Somehow the clock wasn't in FEI mode")
    }
    let mut uart = unsafe {
        let rx = port::Port::new(port::PortName::B).pin(16).make_rx();
        let tx = port::Port::new(port::PortName::B).pin(17).make_tx();
        uart::Uart::new(0, Some(rx), Some(tx), (468, 24))
    };

    let _ = writeln!(uart, "Hello, world!");
    let mut gpio = pin.make_gpio();
    gpio.output();
    gpio.high();

    loop {}
}

extern "C" {
    fn _stack_top() -> !;
}

#[link_section = ".vectors"]
#[no_mangle]
pub static _VECTORS: [unsafe extern "C" fn() -> !; 2] = [_stack_top, main];

#[link_section = ".flashconfig"]
#[no_mangle]
pub static _FLASHCONFIG: [u8; 16] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xDE, 0xF9, 0xFF, 0xFF,
];

#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn rust_begin_panic(
    _msg: core::fmt::Arguments,
    _file: &'static str,
    _line: u32,
) -> ! {
    loop {}
}
