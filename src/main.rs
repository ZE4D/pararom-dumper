#![no_std]
#![no_main]
use arduino_hal::{
    port::{
        mode::{Input, Output, PullUp},
        Pin,
    },
    prelude::*,
};
use panic_halt as _;
use ufmt::uWrite;

//Capacity of ROM in bytes
const CAPACITY: u32 = 131_072;

//Print ASCII characters along with hex values in hex dump
const WITH_ASCII_DUMP: bool = true;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut chip_enable = pins.d2.into_output().downgrade();
    // LED to toggle while reading
    let mut led = pins.d3.into_output().downgrade();

    let mut data_pins = [
        pins.d46.into_pull_up_input().downgrade(),
        pins.d47.into_pull_up_input().downgrade(),
        pins.d48.into_pull_up_input().downgrade(),
        pins.d49.into_pull_up_input().downgrade(),
        pins.d50.into_pull_up_input().downgrade(),
        pins.d51.into_pull_up_input().downgrade(),
        pins.d52.into_pull_up_input().downgrade(),
        pins.d53.into_pull_up_input().downgrade(),
    ];

    let mut addr_pins = [
        pins.d22.into_output().downgrade(),
        pins.d23.into_output().downgrade(),
        pins.d24.into_output().downgrade(),
        pins.d25.into_output().downgrade(),
        pins.d26.into_output().downgrade(),
        pins.d27.into_output().downgrade(),
        pins.d28.into_output().downgrade(),
        pins.d29.into_output().downgrade(),
        pins.d30.into_output().downgrade(),
        pins.d31.into_output().downgrade(),
        pins.d32.into_output().downgrade(),
        pins.d33.into_output().downgrade(),
        pins.d34.into_output().downgrade(),
        pins.d35.into_output().downgrade(),
        pins.d36.into_output().downgrade(),
        pins.d37.into_output().downgrade(),
        pins.d38.into_output().downgrade(),
    ];

    setup(&mut addr_pins);
    chip_enable.set_low();
    led.set_high();

    loop {
        let mut addr = 0;
        while addr < CAPACITY {
            let mut bytes: [u8; 16] = [0; 16];
            for i in 0..16 {
                write_addr(&mut addr_pins, addr);
                let byte = read_byte(&mut data_pins);
                bytes[i] = byte;
                addr = addr + 1;
            }

            to_serial(&bytes, &mut serial);
            led.toggle();
        }

        led.set_low();

        //Done, so loop forever
        loop {}
    }
}

fn to_serial<W: uWrite<Error = void::Void>>(bytes: &[u8], serial: &mut W) {
    for byte in bytes {
        ufmt::uwrite!(serial, "{:02X} ", *byte).void_unwrap();
    }

    if WITH_ASCII_DUMP {
        ufmt::uwrite!(serial, "  ").void_unwrap();
        for byte in bytes {
            let mut c = '.';
            if *byte > 32 && *byte < 127 {
                c = *byte as char;
            }
            ufmt::uwrite!(serial, "{} ", c).void_unwrap();
        }
    }

    ufmt::uwriteln!(serial, "").void_unwrap();
}

fn setup(addr_pins: &mut [Pin<Output>]) {
    for pin in addr_pins.iter_mut() {
        pin.set_low();
    }
}

fn write_addr(addr_pins: &mut [Pin<Output>], addr: u32) {
    let mut mask: u32 = 1;
    for pin in addr_pins.iter_mut() {
        if (mask & addr) != 0 {
            pin.set_high();
        } else {
            pin.set_low();
        }
        mask = mask << 1;
    }
}

fn read_byte(data_pins: &mut [Pin<Input<PullUp>>]) -> u8 {
    let mut data: u8 = 0;
    let mut mask: u8 = 1;

    arduino_hal::delay_us(1); // delay to let data lines settle
    for pin in data_pins.iter_mut() {
        if pin.is_high() {
            data |= mask;
        }
        mask = mask << 1;
    }
    data
}
