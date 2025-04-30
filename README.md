# Embassy Playground
## Waker Churn
Exploring the behavior of different waker primitives when a signal tries to wake multiple tasks.
Here are some different approaches of storing a waker and waking them from `poll_fn` or similar:
- Naive implementation which only stores one Waker -> Only the task gets polled which registers its waker last.
- Implementation using `embassy_sync::waitqueue::AtomicWaker` -> Only calls one task which registered its waker last. This is intentional and the expected behavior of `AtomicWaker`.
- Implementation using `embassy_sync::waitqueue::WakerRegistration` ->  Works but required a lot of polls of the `poll_fn` because the two tasks fight for the spot to the waker.
- Implementation using `embassy_sync::waitqueue::MultiWakerRegistration` -> All tasks are woken up.
- Implementation using `maitake_sync::WaitQueue` -> All tasks are woken up.

To sum up: 
If you are not sure if multiple tasks will want to wait for a change in a signal it's the safest to use either `embassy_sync::waitqueue::MultiWakerRegistration` or `maitake_sync::WaitQueue`.

## On demand Peripheral
Experimenting with wrapping a peripheral in a struct which controls init and deinit of the peripheral. The basic idea is, that the peripheral can be dropped when not needed at the moment and reinitialized again when needed some time later.
This way the clock of the peripheral can be turned off which enables entering STOP mode in the embassy low-power executor.