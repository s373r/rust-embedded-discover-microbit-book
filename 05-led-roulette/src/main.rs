#![no_std]
#![no_main]

use cortex_m_rt::entry;
use microbit::{board::Board, display::blocking::Display, hal::Timer};
use panic_rtt_target as _;
use rtt_target::rtt_init_print;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);

    let mut leds = [
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ];
    let mut x = 0;
    let mut y = 0;
    let mut dx = 1i8;
    let mut dy = 0i8;

    loop {
        display.show(&mut timer, leds, 100);

        leds[y as usize][x as usize] = 0;

        // The author of the book offers an interesting solution (and effective), without conditions
        // (spoiler: using constant tables):
        // https://docs.rust-embedded.org/discovery/microbit/05-led-roulette/my-solution.html

        if x == 4 && dx == 1 {
            dx = 0;
            dy = 1;
        } else if y == 4 && dy == 1 {
            dx = -1;
            dy = 0;
        } else if x == 0 && dx == -1 {
            dx = 0;
            dy = -1;
        } else if y == 0 && dy == -1 {
            dx = 1;
            dy = 0;
        }

        x += dx;
        y += dy;

        leds[y as usize][x as usize] = 1;
    }
}
