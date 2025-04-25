#![no_std]
#![no_main]

use core::cell::{Cell, RefCell};
use core::future::poll_fn;
use core::task::Poll;
use defmt::*;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::ThreadModeMutex;
use embassy_time::Timer;
use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

#[derive(Debug, Clone, Copy, Default, PartialEq, defmt::Format)]
enum State {
    #[default]
    NotReady,
    Ready(u32),
}

type WakerType = embassy_sync::waitqueue::MultiWakerRegistration<2>;

struct Signal {
    state: Cell<State>,
    waker_registration: RefCell<WakerType>,
}

impl Signal {
    fn new() -> Self {
        Self {
            state: Cell::new(State::NotReady),
            waker_registration: RefCell::new(WakerType::new()),
        }
    }
}

type SyncSignal = ThreadModeMutex<Signal>;

async fn signal_wait(signal: &SyncSignal, current_state: State) {
    poll_fn(|cx| {
        trace!("Running waker with address: {:?}", cx.waker().data());

        signal.lock(|s| {
            if s.state.get() != current_state {
                info!("Signal is ready");
                Poll::Ready(())
            } else {
                info!("Signal not ready, registering waker");
                s.waker_registration.borrow_mut().register(cx.waker());
                Poll::Pending
            }
        })
    })
    .await;
}

/// This __does__ work!
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let _p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    static SIGNAL: StaticCell<SyncSignal> = StaticCell::new();
    let signal = SIGNAL.init(ThreadModeMutex::new(Signal::new()));
    let mut counter = 0;

    spawner.must_spawn(wait_for_signal("TaskTwo", signal, true));
    spawner.must_spawn(wait_for_signal("TaskOne", signal, false));

    loop {
        Timer::after_millis(500).await;
        counter += 1;

        signal.lock(|s| {
            s.state.set(State::Ready(counter));
            s.waker_registration.borrow_mut().wake();
        });
    }
}

#[embassy_executor::task(pool_size = 2)]
async fn wait_for_signal(name: &'static str, signal: &'static SyncSignal, odd: bool) {
    info!("Starting {} task", name);

    loop {
        let current_state = signal.lock(|s| s.state.get());

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
