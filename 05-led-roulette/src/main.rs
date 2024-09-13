#![no_std]
#![no_main]

use cortex_m_rt::entry;
use microbit as _;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello World");

    loop {}
}
