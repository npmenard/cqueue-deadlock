# Build
Run `cargo build --release`
Flash `espflash flash  target/xtensa-esp32-espidf/release/cqueue-deadlock --monitor -B 926100`

# Bug with 1.83
After a few minutes the WDT will trigger because of a deadlock in [concurrent-queue](https://github.com/smol-rs/concurrent-queue) 
pthread task and main task race to do a push/pop to the queue but neither is able to make progress
```
I (112643) cqueue_deadlock: next loop
E (122641) task_wdt: Task watchdog got triggered. The following tasks/users did not reset the watchdog in time:
E (122641) task_wdt:  - IDLE0 (CPU 0)
E (122641) task_wdt: Tasks currently running:
E (122641) task_wdt: CPU 0: pthread
E (122641) task_wdt: CPU 1: IDLE1
E (122641) task_wdt: Aborting.
E (122641) task_wdt: Print CPU 0 (current core) backtrace




Backtrace: 0x400df26f:0x3ffe7790 0x400de03d:0x3ffe77e0 0x400dce05:0x3ffe7800 0x400dcb14:0x3ffe7860 0x400dceaa:0x3ffe7950 0x400ddbae:0x3ffe7a60 0x400df55c:0x3ffe7b40 0x400dd80b:0x3ffe7b60 0x400f8b33:0x3ffe7bb0 0x400f89b4:0x3ffe7bd0 0x4010e95c:0x3ffe7bf0
0x400df26f - concurrent_queue::bounded::Bounded<T>::pop
    at /host/.micro-rdk-docker-caches/cargo-registry/registry/src/index.crates.io-6f17d22bba15001f/concurrent-queue-2.5.0/src/bounded.rs:228
0x400de03d - concurrent_queue::ConcurrentQueue<T>::pop
    at ??:??
0x400dce05 - <concurrent_queue::TryIter<T> as core::iter::traits::iterator::Iterator>::next
    at /host/.micro-rdk-docker-caches/cargo-registry/registry/src/index.crates.io-6f17d22bba15001f/concurrent-queue-2.5.0/src/lib.rs:494
0x400dcb14 - async_io::reactor::Reactor::process_timers
    at /host/.micro-rdk-docker-caches/cargo-registry/registry/src/index.crates.io-6f17d22bba15001f/async-io-2.4.0/src/reactor.rs:211
0x400dceaa - async_io::reactor::ReactorLock::react
    at /host/.micro-rdk-docker-caches/cargo-registry/registry/src/index.crates.io-6f17d22bba15001f/async-io-2.4.0/src/reactor.rs:280
0x400ddbae - async_io::driver::main_loop
    at /host/.micro-rdk-docker-caches/cargo-registry/registry/src/index.crates.io-6f17d22bba15001f/async-io-2.4.0/src/driver.rs:69
0x400df55c - async_io::driver::unparker::{{closure}}::{{closure}}
    at /host/.micro-rdk-docker-caches/cargo-registry/registry/src/index.crates.io-6f17d22bba15001f/async-io-2.4.0/src/driver.rs:33
0x400dd80b - std::thread::Builder::spawn_unchecked_::{{closure}}::{{closure}}
    at /opt/rust/rustup/toolchains/esp/lib/rustlib/src/rust/library/std/src/thread/mod.rs:538
0x400f8b33 - <alloc::boxed::Box<F,A> as core::ops::function::FnOnce<Args>>::call_once
    at /opt/rust/rustup/toolchains/esp/lib/rustlib/src/rust/library/alloc/src/boxed.rs:2454
0x400f89b4 - std::sys::pal::unix::thread::Thread::new::thread_start
    at /opt/rust/rustup/toolchains/esp/lib/rustlib/src/rust/library/std/src/sys/pal/unix/thread.rs:105
0x4010e95c - pthread_task_func
    at /opt/esp/esp-idf/components/pthread/pthread.c:196




ELF file SHA256: 000000000

CPU halted.
```

# Bug with 1.85
after a few minutes the program will crash with double exception of load prohibited (see below) or deadlock as well
```
I (16759) cqueue_deadlock: next loop
Guru Meditation Error: Core  0 panic'ed (Double exception).

Core  0 register dump:
PC      : 0x40095722  PS      : 0x00040c36  A0      : 0x00000027  A1      : 0x3ffe77c0
A2      : 0x3ffb3dc0  A3      : 0x00000000  A4      : 0x00000000  A5      : 0x00000000
0x3ffb3dc0 - async_io::reactor::Reactor::get::REACTOR
    at ??:??
A6      : 0x0000c350  A7      : 0x3ffdd090  A8      : 0x800de157  A9      : 0x3ffe7780
A10     : 0x00000002  A11     : 0x3ffcc7ac  A12     : 0x00000000  A13     : 0x3ffcc7b8
A14     : 0x0000c350  A15     : 0x3ffcc7b4  SAR     : 0x0000000c  EXCCAUSE: 0x00000002
EXCVADDR: 0x400840c2  LBEG    : 0x40083b65  LEND    : 0x40083b6d  LCOUNT  : 0x00000027
0x40083b65 - esp_timer_impl_get_counter_reg
    at /opt/esp/esp-idf/components/esp_timer/src/esp_timer_impl_lac.c:118
0x40083b6d - esp_timer_impl_get_counter_reg
    at /opt/esp/esp-idf/components/esp_timer/src/esp_timer_impl_lac.c:128


Backtrace: 0x4009571f:0x3ffe77c0 0x00000024:0x400840f2 |<-CORRUPTED




ELF file SHA256: 000000000

CPU halted.
```

```
Guru Meditation Error: Core  0 panic'ed (LoadProhibited). Exception was unhandled.

Core  0 register dump:
PC      : 0x400df89b  PS      : 0x00060c30  A0      : 0x00060423  A1      : 0x3ffbd8e0
0x400df89b - core::task::wake::Context::waker
    at /opt/rust/rustup/toolchains/esp/lib/rustlib/src/rust/library/core/src/task/wake.rs:250
A2      : 0x3ffbfdb4  A3      : 0x00000004  A4      : 0x00060820  A5      : 0x00000000
A6      : 0x00000000  A7      : 0x00000000  A8      : 0x00060820  A9      : 0x3ffbd8c0
A10     : 0x3ffb3dc0  A11     : 0xffffffff  A12     : 0x0000004c  A13     : 0x00000000
0x3ffb3dc0 - async_io::reactor::Reactor::get::REACTOR
    at ??:??
A14     : 0x0000004c  A15     : 0x00000002  SAR     : 0x0000000c  EXCCAUSE: 0x0000001c
EXCVADDR: 0x00060820  LBEG    : 0x40083b65  LEND    : 0x40083b6d  LCOUNT  : 0x00000027
0x40083b65 - esp_timer_impl_get_counter_reg
    at /opt/esp/esp-idf/components/esp_timer/src/esp_timer_impl_lac.c:118
0x40083b6d - esp_timer_impl_get_counter_reg
    at /opt/esp/esp-idf/components/esp_timer/src/esp_timer_impl_lac.c:128


Backtrace: 0x400df898:0x3ffbd8e0 0x00060420:0x3ffbd910 |<-CORRUPTED
0x400df898 - async_lock::once_cell::OnceCell<T>::get_or_init_blocking
    at /host/.micro-rdk-docker-caches/cargo-registry/registry/src/index.crates.io-1949cf8c6b5b557f/async-lock-3.4.0/src/once_cell.rs:516
```
