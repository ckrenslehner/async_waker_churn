# Waker Churn

This repository explores "waker churn" (thanks @jamesmunns for the term) when a signal tries to wake multiple tasks.
The basic problem boils down to choosing the correct "WakerRegistration" type which can securely wake multiple tasks. 
The repository contains five different versions and tests them.

The following versions are tested:
- Naive implementation which only stores one Waker -> Only the task gets polled which registers its waker last.
- Implementation using `embassy_sync::waitqueue::AtomicWaker` -> Only calls one task which registered its waker last. This is intentional and the expected behavior of `AtomicWaker`.
- Implementation using `embassy_sync::waitqueue::WakerRegistration` ->  Works but required a lot of polls of the `poll_fn` because the two tasks fight for the spot to the waker.
- Implementation using `embassy_sync::waitqueue::MultiWakerRegistration` -> All tasks are woken up.
- Implementation using `maitake_sync::WaitQueue` -> All tasks are woken up.

To sum up: 
If you are not sure if multiple tasks will want to wait for a change in a signal it's the safest to use either `embassy_sync::waitqueue::MultiWakerRegistration` or `maitake_sync::WaitQueue`.