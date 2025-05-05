#![no_std]
#![no_main]

// Import defmt_rtt and panic_probe as unused dependencies. Forces the linker to include them in the binary.
use {defmt_rtt as _, panic_probe as _};

/// Proc macro to generate the main function
/// -> ! means it never returns
#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) -> ! {
    // Init the peripherals with default settings for RCC (clocks, muxing, etc.)
    let p = embassy_stm32::init(Default::default());

    // Init the LED output
    let mut status_led = embassy_stm32::gpio::Output::new(
        p.PD0,
        embassy_stm32::gpio::Level::Low,
        embassy_stm32::gpio::Speed::Low,
    );

    let mut counter = 0;

    // Infinite loop
    loop {
        embassy_time::Timer::after_millis(500).await;
        counter += 1;
        status_led.toggle();

        defmt::info!("Counter: {}", counter);
    }
}
