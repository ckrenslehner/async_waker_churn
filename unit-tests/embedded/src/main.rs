#![no_std]
#![no_main]

use core::cell::{Cell, RefCell};

type MutexType<T> = embassy_sync::mutex::Mutex<NoopRawMutex, T>;
type SharedLed = MutexType<(embassy_stm32::gpio::Output<'static>, u8)>;

use embassy_stm32::gpio::Level;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::WithTimeout;

// Import defmt_rtt and panic_probe as unused dependencies. Forces the linker to include them in the binary.
use {defmt_rtt as _, panic_probe as _};

/// Proc macro to generate the main function
/// -> ! means it never returns
#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) -> ! {
    // Init the peripherals with default settings for RCC (clocks, muxing, etc.)
    let p = embassy_stm32::init(Default::default());

    static LED_CELL: static_cell::StaticCell<SharedLed> = static_cell::StaticCell::new();
    // Init the LED output
    let status_led = LED_CELL.init(MutexType::new((
        embassy_stm32::gpio::Output::new(
            p.PD0,
            embassy_stm32::gpio::Level::Low,
            embassy_stm32::gpio::Speed::Low,
        ),
        0,
    )));

    let mut counter = 0;

    let mut shared = host::SharedType::new(1);
    defmt::info!("SharedType ID: {}", shared.id());

    host::transform_shared_type(&mut shared);
    defmt::info!("Transformed SharedType ID: {}", shared.id());

    spawner.must_spawn(task(
        "Task 1",
        embassy_time::Duration::from_millis(250),
        status_led,
    ));
    spawner.must_spawn(task(
        "Task 2",
        embassy_time::Duration::from_millis(750),
        status_led,
    ));

    loop {
        embassy_time::Timer::after_millis(500).await;
        // counter += 1;
        // status_led.toggle();

        // defmt::info!("Counter: {}", counter);
    }
    // Infinite loop
    // loop {
    //     embassy_time::Timer::after_millis(500).await;
    //     counter += 1;
    //     status_led.toggle();

    //     defmt::info!("Counter: {}", counter);
    // }
}

#[embassy_executor::task(pool_size = 3)]
async fn task(
    name: &'static str,
    interval: embassy_time::Duration,
    status_led: &'static SharedLed,
) -> ! {
    // This is a task that can be run in the executor.
    // It can be used to run async code in a separate thread.
    defmt::info!("Task started: {}", name);

    loop {
        embassy_time::Timer::after(interval).await;
        defmt::info!("Task {}: Timer expired", name);

        let mut guard = status_led.lock().await;
        let (led, count) = &mut *guard;

        led.toggle();
        do_something_with_count(count);
        do_something_with_count(count);
    }
}

fn do_something_with_count(count: &mut u8) {
    // This function does something with the count value.
    // For example, it could increment it or perform some other operation.
    *count += 1;
    defmt::info!("Count updated: {}", count);
}
