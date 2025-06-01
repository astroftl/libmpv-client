# libmpv-client
A Rust wrapper over libmpv.

The primary interface of this crate is [`Handle`], start there.

Currently, only [`client.h`](https://github.com/mpv-player/mpv/blob/release/0.40/include/mpv/client.h) is implemented, which is sufficient for writing mpv [cplugins](https://mpv.io/manual/stable/#c-plugins).

The [latest rustdocs](https://astroftl.github.io/libmpv-client/libmpv_client/) are hosted on GitHub Pages and are kept up to date with `main`.
Due to limitations of this, documentation for other branches or tags much be built yourself. You can do this from your own crate with `cargo doc -p libmpv-client`.

### Windows Support
This crate supports the `MPV_CPLUGIN_DYNAMIC_SYM` function pointers provided by mpv, allowing for working DLLs on Windows.

## Installation
This crate is not yet hosted on crates.io. Until this changes, add the dependency by link to the Git repo:
```toml
[dependencies]
libmpv-client = { git = "https://github.com/astroftl/libmpv-client" }

# Specify the "next" branch for the latest and greatest.
libmpv-client = { git = "https://github.com/astroftl/libmpv-client", branch = "next" }

# Specify a tag to remain pinned to a specific version.
libmpv-client = { git = "https://github.com/astroftl/libmpv-client", tag = "0.2.0" }
```

## mpv cplugin Setup
To use this crate for mpv cplugins (which is its intended purpose), you have to create a Rust library crate with type `cdylib`.

In your Cargo.toml:
```toml
[lib]
crate-type = ["cdylib"]
```

## Example
```rust
use libmpv_client::*;

#[unsafe(no_mangle)]
extern "C" fn mpv_open_cplugin(ptr: *mut mpv_handle) -> std::os::raw::c_int {
    let handle = Handle::from_ptr(ptr);

    println!("Hello from Rust!");

    loop {
        match handle.wait_event(0.0) {
            Ok(event) => {
                match event {
                    Event::Shutdown => {
                        println!("Goodbye from Rust!");
                        return 0;
                    },
                    Event::None => {},
                    event => {
                        println!("Rust got event: {event:?}");
                    },
                }
            }
            Err(e) => {
                println!("Rust got error: {e:?}");
            }
        }
    }
}
```

[`Handle`]: target/doc/libmpv_client/handle/struct.Handle.html