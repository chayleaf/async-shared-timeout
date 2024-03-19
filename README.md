[![crates.io](https://img.shields.io/crates/v/async-shared-timeout.svg)](https://crates.io/crates/async-shared-timeout) [![docs.rs](https://docs.rs/async-shared-timeout/badge.svg)](https://docs.rs/async-shared-timeout)

# Async Shared Timeout

A Rust crate for creating a shared timeout. A sample use case is having multiple streams open from the client, and expiring them only when all of them stopped sending data. Another example is a proxy with a timeout - the proxy times out only when both the local and the remote ends time out.

# Feature flags:
 
**Wrapper**

- `wrapper` - enable a wrapper around types that you can use for easier resetting. By default,
              only future support is enabled (reset the timer upon future completion).
- `read-write` - enable async `Read`/`Write` trait support for the wrapper (reset the timer
                 upon successful read/write operations)
- `stream` - enable `Stream` support for the wrapper (reset the timer upon stream advancement).

**Integration with other runtimes**

- `std` (enabled by default) - enable `std` integration. Currently it's only used to enable
                               `Arc` and `AsRawFd` support for the wrapper.
- `tokio` (enabled by default) - [`tokio`](https://docs.rs/tokio) support
- `async-io` - support [`async-io`](https://docs.rs/async-io) as the timer runtime.
- `futures-io` - support [`futures-io`](https://docs.rs/futures-io) traits.
- `async-std` - [`async-std`](https://docs.rs/async-std) support (enables `async-io` and `futures-io`).

## Changelog

- 0.1.0 - initial release
- 0.1.1 - `AsRawFd` support for `Wrapper`
- 0.2.0 - minor API cleanup
- 0.2.1 - updated dependencies, added `Timeout::new_tokio`

## License

TL;DR do whatever you want.

Licensed under either the [BSD Zero Clause License](LICENSE-0BSD) (https://opensource.org/licenses/0BSD), the [Apache 2.0 License](LICENSE-APACHE) (http://www.apache.org/licenses/LICENSE-2.0) or the [MIT License](LICENSE-MIT) (http://opensource.org/licenses/MIT), at your choice.

