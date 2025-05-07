//! Example of a OnDemandOutput type which initializes the Output only when needed.
//! Uses maitake_sync::Mutex (wait queue) to synchronize access to the output.

#![no_std]
#![no_main]

use core::{
    cell::Cell,
    ops::{Deref, DerefMut},
};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    Peripheral,
    gpio::{AnyPin, Pin},
};
use embassy_time::WithTimeout;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

/// Maitake mutex uses a wait queue which calls wake in a FIFO order.
/// So to be fair, the task which asked for the mutex first will be woken up first.
type Mutex<T> = maitake_sync::Mutex<T>;

/// Output peripheral wrapper which is used to initialize the output only when needed.
type MutexInner<'a> = Option<embassy_stm32::gpio::Output<'a>>;

/// MutexGuard wrapper to have a custom drop implementation.
struct OutputGuard<'a> {
    /// The output peripheral.
    inner: maitake_sync::MutexGuard<'a, MutexInner<'a>>,
    /// The reference count to the output -> We need to decrement it when the guard is dropped.
    reference_count: ReferenceCount<'a>,
}

impl<'a> Deref for OutputGuard<'a> {
    type Target = embassy_stm32::gpio::Output<'a>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl<'a> DerefMut for OutputGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

impl Drop for OutputGuard<'_> {
    fn drop(&mut self) {
        trace!("Dropping MutexGuard");

        let current_refcount = self.reference_count.get();

        if current_refcount == 1 {
            debug!("Last reference dropped, deinitializing output by dropping it");
            // Deinitialize the output via dropping the inner peripheral
            self.inner.take();
        }
    }
}

/// Reference count wrapper which increments the reference count when created and decrements it when dropped.
/// This is used to make `get_or_init` cancel safe.
struct ReferenceCount<'a> {
    /// The reference count to the output
    count: &'a Cell<usize>,
}

impl<'a> Deref for ReferenceCount<'a> {
    type Target = Cell<usize>;

    fn deref(&self) -> &Self::Target {
        &self.count
    }
}

impl<'a> ReferenceCount<'a> {
    /// Create a new reference count wrapper which increments the reference count.
    fn new(count: &'a Cell<usize>) -> Self {
        count.set(count.get() + 1);
        trace!("Incremented reference count: {}", count.get());
        Self { count }
    }
}

/// The reference count is decremented when the wrapper is dropped.
/// So if there is a cancelation, the reference count will be decremented, because it is never stored in a MutexGuard.
impl Drop for ReferenceCount<'_> {
    fn drop(&mut self) {
        let current_refcount = self.count.get();
        defmt::debug_assert!(current_refcount > 0, "Reference count is already 0");

        trace!(
            "Dropping ReferenceCount. Decrementing reference count from: {}",
            current_refcount
        );
        self.count.set(current_refcount - 1);
    }
}

/// Wannabe OnDemandOutput type which initializes the Output only when needed.
/// And deinitializes it when the last reference is dropped.
/// Useful for low power applications where we want to avoid a lock on on entering stop mode because of an active peripheral.
/// 
/// TODO: Test if a init is possible without `'static` lifetime.
struct OnDemandOutput<'a> {
    // -- Configuration --
    /// Output pin
    pin: AnyPin,
    /// Output level
    level: embassy_stm32::gpio::Level,
    /// Output speed
    speed: embassy_stm32::gpio::Speed,

    // -- Peripheral --
    /// The actual peripheral
    output: Mutex<MutexInner<'a>>,
    /// No need for atomics whatsoever, because multithreaded not possible without send + sync?
    reference_count: Cell<usize>,
}

impl<'a> OnDemandOutput<'a> {
    fn new(
        pin: AnyPin,
        level: embassy_stm32::gpio::Level,
        speed: embassy_stm32::gpio::Speed,
    ) -> Self {
        Self {
            pin: pin,
            level: level,
            speed: speed,
            output: Mutex::new(MutexInner::None),
            reference_count: Cell::new(0),
        }
    }

    /// Get the output peripheral, initializing if needed.
    async fn get_or_init(&'a self) -> OutputGuard<'a> {
        // Create a reference count wrapper which increments the reference count
        let reference_count = ReferenceCount::new(&self.reference_count);
        trace!(
            "Waiting for lock. Reference count: {}",
            reference_count.get()
        );

        // Lock the output mutex after the reference count mutex
        // If the future is canceled, `reference_count` will be dropped and the reference count will be decremented
        let mut output = self.output.lock().await;
        trace!("Got output mutex");

        // Initialize the output if it is not initialized yet
        if output.is_none() {
            debug!("Initializing output..");

            *output = Some(embassy_stm32::gpio::Output::new(
                // TODO: Can I somehow use `PeripheralRef` here? I could not figure it out yet.
                unsafe { self.pin.clone_unchecked() },
                self.level,
                self.speed,
            ));
        } else {
            debug!("Output already initialized");
        };

        OutputGuard {
            inner: output,
            reference_count: reference_count,
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");
    let p = embassy_stm32::init(Default::default());

    static CELL: StaticCell<OnDemandOutput<'static>> = StaticCell::new();

    let on_demand = CELL.init(OnDemandOutput::new(
        p.PD0,
        embassy_stm32::gpio::Level::Low,
        embassy_stm32::gpio::Speed::Low,
    ));

    let starting_instant = embassy_time::Instant::from_ticks(0);

    // Start tasks which absolute time reference
    spawner.must_spawn(blink_fast("TaskOne", on_demand, starting_instant));
    spawner.must_spawn(blink_slow("TaskTwo", on_demand, starting_instant));
    spawner.must_spawn(blink_fast_cancel("TaskCancel", on_demand, starting_instant));
}

/// Blink slow
#[embassy_executor::task]
async fn blink_slow(
    name: &'static str,
    output: &'static OnDemandOutput<'static>,
    mut instant: embassy_time::Instant,
) {
    loop {
        info!(
            "Starting {} at {}: Waiting for output",
            name,
            instant.as_millis()
        );

        {
            let mut guard = output.get_or_init().await;
            info!("{}: Got output - Toggle", name);

            // Use the output
            for _ in 0..=3 {
                guard.toggle();
                embassy_time::Timer::after(embassy_time::Duration::from_millis(1000)).await;
            }
        }

        let new_instant = instant + embassy_time::Duration::from_millis(10000);
        embassy_time::Timer::at(new_instant).await;
        instant = new_instant;
    }
}

/// Blink fast
#[embassy_executor::task]
async fn blink_fast(
    name: &'static str,
    output: &'static OnDemandOutput<'static>,
    mut instant: embassy_time::Instant,
) {
    loop {
        info!(
            "Starting {} at {}: Waiting for output",
            name,
            instant.as_millis()
        );

        {
            let mut guard = output.get_or_init().await;
            info!("{}: Got output - Toggle", name);

            // Use the output
            for _ in 0..=3 {
                guard.toggle();
                embassy_time::Timer::after(embassy_time::Duration::from_millis(250)).await;
            }
        }

        let new_instant = instant + embassy_time::Duration::from_millis(10000);
        embassy_time::Timer::at(new_instant).await;
        instant = new_instant;
    }
}

/// Task which tests safe cancellation of the `get_or_init` method.
#[embassy_executor::task]
async fn blink_fast_cancel(
    name: &'static str,
    output: &'static OnDemandOutput<'static>,
    mut instant: embassy_time::Instant,
) {
    loop {
        info!(
            "Starting {} at {}: Waiting for output",
            name,
            instant.as_millis()
        );

        let guard_res = output
            .get_or_init()
            .with_timeout(embassy_time::Duration::from_millis(20))
            .await;

        match guard_res {
            Ok(_) => {}
            Err(_) => {
                info!("{}: Cancelled", name);
            }
        }

        let new_instant = instant + embassy_time::Duration::from_millis(10000);
        embassy_time::Timer::at(new_instant).await;
        instant = new_instant;
    }
}
