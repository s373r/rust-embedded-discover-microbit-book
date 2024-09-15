#![no_main]
#![no_std]

mod serial_setup;

use core::fmt::Write;

use cortex_m_rt::entry;
use heapless::String;
use lsm303agr::{AccelOutputDataRate, Lsm303agr, MagOutputDataRate, Measurement};
use microbit::hal::prelude::*;
use microbit::hal::uarte::{Baudrate, Parity};
use microbit::hal::Uarte;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

#[cfg(feature = "v1")]
use microbit::{hal::twi, pac::twi0::frequency::FREQUENCY_A};

#[cfg(feature = "v2")]
use microbit::{hal::twim, pac::twim0::frequency::FREQUENCY_A};

use crate::serial_setup::UartePort;

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
        let serial = Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    #[cfg(feature = "v1")]
    let i2c = { twi::Twi::new(board.TWI0, board.i2c.into(), FREQUENCY_A::K100) };

    #[cfg(feature = "v2")]
    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut sensor = Lsm303agr::new_with_i2c(i2c);

    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz50).unwrap();

    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    const LINE_MAX_LEN: usize = 32;

    let mut line: String<LINE_MAX_LEN> = String::new();

    loop {
        let received_byte = nb::block!(serial.read()).unwrap();

        // For clarity, we echo the input characters of the command
        nb::block!(serial.write(received_byte)).unwrap();

        match received_byte {
            // Received an ENTER press, execute if it is a valid command
            b'\r' => {
                match line.as_str() {
                    "magnetometer" | "mag" => {
                        let Measurement { x, y, z } = sensor.mag_data().unwrap();

                        write!(serial, "magnetism: x {x} y {y} z {z}").unwrap();
                    }
                    "accelerometer" | "acc" => {
                        let Measurement { x, y, z } = sensor.accel_data().unwrap();

                        write!(serial, "Acceleration: x {x} y {y} z {z}").unwrap();
                    }
                    unexpected_command => {
                        write!(serial, "\r\nUnexpected command: {unexpected_command}").unwrap();
                    }
                }

                write!(serial, "\r\n").unwrap();

                line.clear();
            }
            // Backspace
            8 => {
                line.pop();
            }
            // We get any other character, save it
            _ => {
                let push_result = line.push(received_byte as char);

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
