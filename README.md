# Exception Handler

**exception-handler** is a Rust library for registering platform-level exception and signal handlers that intercept hardware faults (segfaults, bus errors, illegal instructions) and route them to typed Rust callbacks for recovery, logging, or graceful shutdown.

## Why It Matters

In production systems, a segmentation fault crashes the process instantly — no stack trace, no cleanup, no error report. Signal handlers intercept these fatal signals before the OS terminates the process, giving you a window to log diagnostics, flush buffers, or trigger a controlled restart. Every database engine, web server, and long-running daemon needs this: PostgreSQL, Redis, and Nginx all register custom signal handlers. This library provides a safe, typed abstraction over the unsafe `sigaction` syscall.

## How It Works

### Signal Handling Fundamentals

When the CPU triggers a hardware fault (page fault, division by zero, illegal instruction), the kernel delivers a POSIX signal to the process:

| Signal | Number | Cause |
|--------|--------|-------|
| `SIGSEGV` | 11 | Invalid memory access (null deref, use-after-free) |
| `SIGBUS` | 7 | Bus error (misaligned access, mmapped file truncated) |
| `SIGFPE` | 8 | Arithmetic exception (division by zero) |
| `SIGILL` | 4 | Illegal CPU instruction |
| `SIGABRT` | 6 | Abort (assertion failure, `abort()`) |

### Registration Model

The `ExceptionHandler` accepts a callback `Box<dyn Fn(&ExceptionInfo) -> bool + Send + Sync>`. The callback receives:

```rust
pub struct ExceptionInfo {
    pub signal: i32,       // Signal number (e.g., SIGSEGV = 11)
    pub address: usize,    // Faulting memory address
    pub fault_type: FaultType,  // Read, Write, Execute, or Unknown
}
```

Returning `true` from the callback indicates the handler resolved the fault (e.g., by mapping memory via `mmap` for a lazy-allocation scheme). Returning `false` lets the default handler terminate the process.

### Safety Constraints

Signal handlers run in a restricted environment:
- Only **async-signal-safe** functions may be called (no `malloc`, no `printf`, no mutexes)
- The handler must not access non-`Sync` data
- The handler stack may be limited (typically 8 KiB; use `sigaltstack` for larger)

This library enforces `Send + Sync` at the type level, but the caller is responsible for ensuring the callback body only uses signal-safe operations.

**Complexity:** Signal delivery is O(1) from the kernel's perspective. Handler invocation adds ~100ns overhead per signal on modern hardware.

## Quick Start

```rust
use exception_handler::{ExceptionHandler, ExceptionInfo, FaultType, HandlerError};

fn main() {
    let mut handler = ExceptionHandler::new();

    // Register a handler that logs fault information
    let result = handler.install(Box::new(|info: &ExceptionInfo| -> bool {
        eprintln!("FAULT: signal={} addr={:#x} type={:?}",
            info.signal, info.address, info.fault_type);
        false // Let default handler terminate after logging
    }));

    match result {
        Ok(()) => println!("Handler installed"),
        Err(HandlerError::PlatformUnsupported) => {
            eprintln!("Signal handlers not supported on this platform");
        }
        Err(HandlerError::AlreadyInstalled) => {
            eprintln!("Handler already registered");
        }
    }
}
```

## API

### `ExceptionHandler`
- `new() → ExceptionHandler` — Create without an installed handler
- `install(callback) → Result<(), HandlerError>` — Register the signal callback

### `ExceptionInfo`
| Field | Type | Description |
|-------|------|-------------|
| `signal` | `i32` | POSIX signal number |
| `address` | `usize` | Faulting address |
| `fault_type` | `FaultType` | `Read`, `Write`, `Execute`, or `Unknown` |

### `HandlerError`
- `AlreadyInstalled` — Only one handler may be active per process
- `PlatformUnsupported` — OS does not support custom signal handlers

## Architecture Notes

This crate provides the fault-recovery layer for SuperInstance. When a node experiences a hardware fault, the handler captures diagnostics before the process exits, enabling the γ + η = C framework to detect the failure and reassign work. The fault data feeds into the η layer's health monitoring system.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for the full design.

## References

- Kerrisk, M. (2010). *The Linux Programming Interface*. No Starch Press. Chapter 20–21 on signals.
- Stevens, W. R., & Rago, S. A. (2013). *Advanced Programming in the UNIX Environment* (3rd ed.). Addison-Wesley. §10.
- POSIX.1-2017. *signal.h*. [pubs.opengroup.org](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/signal.h.html)

## License

MIT
