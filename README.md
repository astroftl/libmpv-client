```rust
use libmpv_client::mpv_handle;
use libmpv_client::event::Event;
use libmpv_client::handle::Handle;

#[unsafe(no_mangle)]
extern "C" fn mpv_open_cplugin(handle: *mut mpv_handle) -> std::os::raw::c_int {
    let mut client = Handle::from_ptr(handle);

    println!("mpv_open_cplugin: hi from Rust!");

    loop {
        match client.wait_event(0.0) {
            Event::Shutdown => { return 0; },
            Event::None => {},
            event => { println!("Got event: {:?}", event); },
        }
   }
}
```