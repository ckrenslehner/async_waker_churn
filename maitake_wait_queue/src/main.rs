#![no_std]
#![no_main]

use core::cell::Cell;
use defmt::*;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::ThreadModeMutex;
use embassy_time::Timer;
use maitake_sync::WaitQueue;
use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

#[derive(Debug, Clone, Copy, Default, PartialEq, defmt::Format)]
enum State {
    #[default]
    NotReady,
    Ready(u32),
}

struct Signal {
    state: ThreadModeMutex<Cell<State>>,
    waker_registration: WaitQueue,
}

impl Signal {
    fn new() -> Self {
        Self {
            state: ThreadModeMutex::new(Cell::new(State::NotReady)),
            waker_registration: WaitQueue::new(),
        }
    }
}

type SyncSignal = Signal;

async fn signal_wait(signal: &SyncSignal, current_state: State) {
    signal
        .waker_registration
        .wait_for(|| current_state != signal.state.lock(|s| s.get()))
        .await
        .expect("Failed to wait");
}

/// This __does__ work! And better, no manual "leaf" future implementation via `poll_fn` is needed. Also the number of waiters does not need to be specified upfront.
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let _p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    static SIGNAL: StaticCell<SyncSignal> = StaticCell::new();
    let signal = SIGNAL.init(Signal::new());
    let mut counter = 0;

    spawner.must_spawn(wait_for_signal("TaskTwo", signal, true));
    spawner.must_spawn(wait_for_signal("TaskOne", signal, false));
    spawner.must_spawn(wait_for_signal("TaskThree", signal, true));

    loop {
        Timer::after_millis(500).await;
        counter += 1;

        signal.state.lock(|s| {
            s.set(State::Ready(counter));
        });
        signal.waker_registration.wake_all();
    }
}

#[embassy_executor::task(pool_size = 3)]
async fn wait_for_signal(name: &'static str, signal: &'static SyncSignal, odd: bool) {
    info!("Starting {} task", name);

    loop {
        let current_state = signal.state.lock(|s| s.get());

        match (odd, current_state) {
            (true, State::Ready(x)) if x % 2 == 1 => {
                info!("{}: Odd state: {:?}", name, current_state);
            }
            (false, State::Ready(x)) if x % 2 == 0 => {
                info!("{}: Even state: {:?}", name, current_state);
            }
            _ => {}
        }

        signal_wait(signal, current_state).await;
    }
}
