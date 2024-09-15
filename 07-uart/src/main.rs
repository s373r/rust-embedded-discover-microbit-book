#![no_main]
#![no_std]

use core::fmt::Write;
use cortex_m_rt::entry;
use heapless::Vec;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

#[cfg(feature = "v1")]
use microbit::{
    hal::prelude::*,
    hal::uart,
    hal::uart::{Baudrate, Parity},
};

#[cfg(feature = "v2")]
use microbit::{
    hal::prelude::*,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
};

#[cfg(feature = "v2")]
mod serial_setup;
#[cfg(feature = "v2")]
use serial_setup::UartePort;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = microbit::Board::take().unwrap();

    #[cfg(feature = "v1")]
    let mut serial = {
        uart::Uart::new(
            board.UART0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        )
    };

    #[cfg(feature = "v2")]
    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    // NOTE: Make sure the minicom config is present and populated with the correct values:
    // ```shell
    // $ cat ~/.minirc.dfl
    // pu baudrate 115200
    // pu bits 8
    // pu parity N
    // pu stopbits 1
    // pu rtscts No
    // pu xonxoff No
    // ```
    //
    // To launch, use:
    // ```shell
    // minicom -D /dev/ttyACM0 -b 115200
    // ```

    const LINE_MAX_LEN: usize = 32;

    let mut line: Vec<u8, LINE_MAX_LEN> = Vec::new();

    loop {
        let received_byte = nb::block!(serial.read()).unwrap();

        match received_byte {
            // Received an ENTER press, output the reversed line
            b'\r' => {
                for c in line.iter().rev() {
                    nb::block!(serial.write(*c)).unwrap();
                }

                write!(serial, "\r\n").unwrap();

                line.clear();
            }
            // We get any other character, save it
            _ => {
                let push_result = line.push(received_byte);

                // The error can be in case of buffer exhaustion
                if push_result.is_err() {
                    write!(
                        serial,
                        "\r\nThe entered value cannot be longer than {LINE_MAX_LEN} characters!"
                    )
                    .unwrap();

                    line.clear();
                };
            }
        }
    }
}
