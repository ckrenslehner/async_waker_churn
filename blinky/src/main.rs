#![no_std]
#![no_main]

// Import defmt_rtt and panic_probe as unused dependencies. Forces the linker to include them in the binary.
use {defmt_rtt as _, embassy_sync::mutex, panic_probe as _};
use embassy_sync::blocking_mutex::CriticalSectionMutex;

static MY_CELL: static_cell::StaticCell<u8> = static_cell::StaticCell::new();

struct MyStruct {
    pub a: u8,
    pub b: u8,
}

impl Drop for MyStruct {
    fn drop(&mut self) {
        defmt::info!("Dropping MyStruct");
    }
}

/// Proc macro to generate the main function
/// -> ! means it never returns
#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) -> ! {
    // Init the peripherals with default settings for RCC (clocks, muxing, etc.)
    let config = embassy_stm32::Config::default();
    let p = embassy_stm32::init(config);

    // Init the LED output
    let mut status_led = embassy_stm32::gpio::Output::new(
        p.PD0,
        embassy_stm32::gpio::Level::Low,
        embassy_stm32::gpio::Speed::Low,
    );

    {
        // Create a new MyStruct
        let my_struct = MyStruct { a: 1, b: 2 };
        defmt::info!("MyStruct created: a={}, b={}", my_struct.a, my_struct.b);
    }

    {
        // Create a new MyStruct
        let my_struct = MyStruct { a: 3, b: 4 };
        defmt::info!("MyStruct created: a={}, b={}", my_struct.a, my_struct.b);
        drop(my_struct);
        defmt::info!("MyStruct dropped");
    }

    let mut counter = 0;

    {
        //
        let init_number = MY_CELL.init(0x42);
        print(init_number);
    }

    let my_mutex: CriticalSectionMutex<u8> = CriticalSectionMutex::new(10);

    // Infinite loop
    loop {
        embassy_time::Timer::after_millis(500).await;
        counter += 1;
        status_led.toggle();

        defmt::info!("Counter: {}", counter);
        defmt::debug!("Counter: {}", counter);
        defmt::trace!("Counter: {}", counter);
    }
}

fn print(s: &'static u8) {
    defmt::info!("{}", s);
}

fn needs_to_run_in_cs(_cs: critical_section::CriticalSection) {

}