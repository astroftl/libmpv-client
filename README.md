# libmpv-client
A Rust wrapper over libmpv.

Currently, only [`client.h`](https://github.com/mpv-player/mpv/blob/release/0.40/include/mpv/client.h) is implemented, which is sufficient for writing mpv [cplugins](https://mpv.io/manual/stable/#c-plugins).

### Windows Support
This crate supports the `MPV_CPLUGIN_DYNAMIC_SYM` function pointers provided by mpv, allowing for working DLLs on Windows.

## Example
```rust
use libmpv_client::*;

#[unsafe(no_mangle)]
extern "C" fn mpv_open_cplugin(handle: *mut mpv_handle) -> std::os::raw::c_int {
    let client = Handle::from_ptr(handle);

    println!("Hi from Rust!");

    loop {
        match client.wait_event(0.0) {
            Ok(event) => {
                match event {
                    Event::Shutdown => {
                        println!("Goodbye from Rust!");
                        client.destroy();
                        return 0;
                    },
                    Event::None => {},
                    _ => {},
                }
            }
            Err(e) => {
                println!("wait_event error: {e:?}");
            }
        }
    }
}
```