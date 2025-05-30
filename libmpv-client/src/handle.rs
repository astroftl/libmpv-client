use std::ffi::{c_void, CStr, CString};
use std::mem::MaybeUninit;
use std::ptr::null;
use std::str::FromStr;

use libmpv_client_sys as mpv;
use libmpv_client_sys::mpv_node;
use crate::*;
use crate::error::error_to_result;
use crate::traits::MpvSend;

pub struct Handle {
    handle: *mut mpv_handle,
    destroyed: bool,
}

impl Handle {
    pub fn from_ptr(handle: *mut mpv_handle) -> Self {
        Handle {
            handle,
            destroyed: false,
        }
    }

    /// Return the MPV_CLIENT_API_VERSION the mpv source has been compiled with.
    pub fn client_api_version() -> u64 {
        unsafe { mpv::client_api_version() as u64 }
    }

    /// Return the name of this client handle. Every client has its own unique name, which is mostly used for user interface purposes.
    pub fn client_name(&self) -> String {
        let c_str = unsafe { CStr::from_ptr(mpv::client_name(self.handle)) };
        c_str.to_string_lossy().to_string()
    }

    /// Return the ID of this client handle.
    ///
    /// Every client has its own unique ID. This  ID is never reused by the core, even if the mpv_handle at hand gets destroyed and new handles get allocated.
    ///
    /// Some mpv APIs (not necessarily all) accept a name in the form "@<id>" in addition of the proper `client_name()`, where "<id>" is the ID in decimal form (e.g. "@123"). For example, the "script-message-to" command takes the client name as first argument, but also accepts the client ID formatted in this manner.
    pub fn client_id(&self) -> i64 {
        unsafe { mpv::client_id(self.handle) }
    }

    /// Create a new mpv instance and an associated client API handle to control the mpv instance. This instance is in a pre-initialized state, and needs to be initialized to be actually used with most other API functions.
    ///
    /// Some API functions will return `Error::Uninitialized` in the uninitialized state. You can call `Handle::set_property()` (or `Handle::set_property_string()` and other variants) to set initial options. After this, call `Handle::initialize()` to start the player, and then use e.g. `Handle::command()` to start playback of a file.
    ///
    /// The point of separating handle creation and actual initialization is that you can configure things which can't be changed during runtime.
    ///
    /// Unlike the command line player, this will have initial settings suitable for embedding in applications. The following settings are different:
    /// - stdin/stdout/stderr and the terminal will never be accessed. This is equivalent to setting the --no-terminal option. (Technically, this also suppresses C signal handling.)
    /// - No config files will be loaded. This is roughly equivalent to using --config=no. Since libmpv 1.15, you can actually re-enable this option, which will make libmpv load config files during `Handle::initialize()`. If you do this, you are strongly encouraged to set the "config-dir" option too. (Otherwise it will load the mpv command line player's config.)
    /// - Idle mode is enabled, which means the playback core will enter idle mode if there are no more files to play on the internal playlist, instead of exiting. This is equivalent to the --idle option.
    /// - Disable parts of input handling.
    /// - Most of the different settings can be viewed with the command line player by running "mpv --show-profile=libmpv".
    ///
    /// All this assumes that API users want a mpv instance that is strictly isolated from the command line player's configuration, user settings, and so on. You can re-enable disabled features by setting the appropriate options.
    ///
    /// The mpv command line parser is not available through this API, but you can set individual options with `Handle::set_property()`. Files for playback must be loaded with `Handle::command()` or others.
    ///
    /// Note that you should avoid doing concurrent accesses on the uninitialized client handle. (Whether concurrent access is definitely allowed or not has yet to be decided.)
    pub fn create() -> Handle {
        let handle = unsafe { mpv::create() };
        Handle::from_ptr(handle)
    }

    /// Initialize an uninitialized mpv instance. If the mpv instance is already running, an `Error` is returned.
    ///
    /// This function needs to be called to make full use of the client API if the client API handle was created with `Handle::create()`.
    ///
    /// Only the following options are required to be set _before_ `Handle::initialize()`:
    /// - options which are only read at initialization time:
    ///   - config
    ///   - config-dir
    ///   - input-conf
    ///   - load-scripts
    ///   - script
    ///   - player-operation-mode
    ///   - input-app-events (macOS)
    /// - all encoding mode options
    ///
    /// # Return
    /// A `Result<i32> where the Ok value is the success code returned from mpv.
    pub fn initialize(&self) -> Result<i32> {
        let err = unsafe { mpv::initialize(self.handle) };
        error_to_result(err)
    }

    /// Disconnect and destroy the `mpv_handle`. The underlying `mpv_handle` will be deallocated with this API call.
    ///
    /// If the last `mpv_handle` is detached, the core player is destroyed. In addition, if there are only weak `mpv_handle`s (such as created by `Handle::create_weak_client()` or internal scripts), these `mpv_handle`s will be sent `MPV_EVENT_SHUTDOWN`. This function may block until these clients have responded to the shutdown event, and the core is finally destroyed.
    ///
    /// # Crate Note
    /// This function is guarded such that the `Handle` may only be destroyed once. `destroy()` is called automatically when the `Handle` is `Drop`'d.
    pub fn destroy(&mut self) {
        if !self.destroyed {
            unsafe { mpv::destroy(self.handle) };
            self.destroyed = true;
        }
    }

    /// Similar to `destroy()`, but brings the player and all clients down as well, and waits until all of them are destroyed. This function blocks.
    ///
    /// The advantage over `destroy()` is that while `destroy()` merely detaches the client handle from the player, this function quits the player, waits until all other clients are destroyed (i.e. all `mpv_handle`s are detached), and also waits for the final termination of the player.
    ///
    /// Since `mpv_destroy()` is called somewhere on the way, it's not safe to call other functions concurrently on the same context.
    ///
    /// # Crate Note
    /// This function is guarded such that the `Handle` may only be destroyed once.
    pub fn terminate_destroy(&mut self) {
        if !self.destroyed {
            unsafe { mpv::terminate_destroy(self.handle) }
            self.destroyed = true;
        }
    }

    /// Create a new client `Handle` connected to the same player core as `Self`. This context has its own event queue, its own `request_event()` state, its own `request_log_messages()` state, its own set of observed properties, and its own state for asynchronous operations. Otherwise, everything is shared.
    ///
    /// This handle should be destroyed with `destroy()` if no longer needed. The core will live as long as there is at least 1 handle referencing it. Any handle can make the core quit, which will result in every handle receiving `Event::Shutdown`.
    pub fn create_client(&self, name: impl ToString) -> Handle {
        let name_str = CString::new(name.to_string().as_bytes()).expect("create_client() name can't contain NULL");

        let handle = unsafe { mpv::create_client(self.handle, name_str.as_ptr()) };
        Handle::from_ptr(handle)
    }

    /// This is the same as `create_client()`, but the created `mpv_handle` is treated as a weak reference. If all `mpv_handles` referencing a core are weak references, the core is automatically destroyed. (This still goes through normal uninit of course. Effectively, if the last non-weak `mpv_handle` is destroyed, then the weak `mpv_handles` receive `Event::Shutdown` and are asked to terminate as well.)
    ///
    /// Note if you want to use this like refcounting: you have to be aware that `terminate_destroy()` _and_ `destroy()` for the last non-weak `mpv_handle` will block until all weak `mpv_handles` are destroyed.
    pub fn create_weak_client(&self, name: impl ToString) -> Handle {
        let name_str = CString::new(name.to_string().as_bytes()).expect("create_weak_client() name can't contain NULL");

        let handle = unsafe { mpv::create_weak_client(self.handle, name_str.as_ptr()) };
        Handle::from_ptr(handle)
    }

    /// Load a config file. This loads and parses the file, and sets every entry in the config file's default section as if `set_option_string()` is called.
    ///
    /// The filename should be an absolute path. If it isn't, the actual path used is unspecified. (Note: an absolute path starts with '/' on UNIX.) If the file wasn't found, `Error::InvalidParameter` is returned.
    ///
    /// If a fatal error happens when parsing a config file, `Error::OptionError` is returned. Errors when setting options as well as other types or errors are ignored (even if options do not exist). You can still try to capture the resulting error messages with `request_log_messages()`. Note that it's possible that some options were successfully set even if any of these errors happen.
    ///
    /// # Return
    /// A `Result<i32> where the Ok value is the success code returned from mpv.
    pub fn load_config_file(&self, filename: impl ToString) -> Result<i32> {
        let filename_str = CString::new(filename.to_string().as_bytes()).expect("load_config_file() filename can't contain NULL");

        let err = unsafe { mpv::load_config_file(self.handle, filename_str.as_ptr()) };
        error_to_result(err)
    }

    /// Return the internal time in nanoseconds. This has an arbitrary start offset, but will never wrap or go backwards.
    ///
    /// Note that this is always the real time, and doesn't necessarily have to do with playback time. For example, playback could go faster or slower due to playback speed, or due to playback being paused. Use the "time-pos" property instead to get the playback status.
    ///
    /// Unlike other libmpv APIs, this can be called at absolutely any time (even within wakeup callbacks), as long as the context is valid.
    ///
    /// Safe to be called from mpv render API threads.
    pub fn get_time_ns(&self) -> i64 {
        unsafe { mpv::get_time_ns(self.handle) }
    }

    /// Same as `get_time_ns` but in microseconds.
    pub fn get_time_us(&self) -> i64 {
        unsafe { mpv::get_time_us(self.handle) }
    }

    /// Set an option. Note that you can't normally set options during runtime. It works in uninitialized state (see `create()`), and in some cases in at runtime.
    ///
    /// Using a format other than `Format::Node` is equivalent to constructing a `Node` with the given format and data, and passing the `Node` to this function.
    ///
    /// # Return
    /// A `Result<i32> where the Ok value is the success code returned from mpv.
    #[deprecated = "For most purposes, this is not needed anymore. Starting with mpv version 0.21.0 (version 1.23) most options can be set with mpv_set_property() (and related functions), and even before mpv_initialize(). In some obscure corner cases, using this function to set options might still be required (see \"Inconsistencies between options and properties\" in the manpage). Once these are resolved, the option setting functions might be fully deprecated."]
    pub fn set_option<T: MpvSend>(&self, name: impl ToString, data: T) -> Result<i32> {
        let name_str = CString::new(name.to_string().as_bytes()).expect("set_option() name can't contain NULL");

        data.to_mpv(|x| {
            let err = unsafe { mpv::set_option(self.handle, name_str.as_ptr(), T::MPV_FORMAT.0, x) };
            error_to_result(err)
        })
    }

    /// Convenience function to set an option to a string value. This is like calling `set_option()` with `Format::String`.
    ///
    /// # Return
    /// A `Result<i32> where the Ok value is the success code returned from mpv.
    #[deprecated = "For most purposes, this is not needed anymore. Starting with mpv version 0.21.0 (version 1.23) most options can be set with mpv_set_property() (and related functions), and even before mpv_initialize(). In some obscure corner cases, using this function to set options might still be required (see \"Inconsistencies between options and properties\" in the manpage). Once these are resolved, the option setting functions might be fully deprecated."]
    pub fn set_option_string(&self, name: impl ToString, data: impl ToString) -> Result<i32> {
        let name_str = CString::new(name.to_string().as_bytes()).expect("set_option_string() name can't contain NULL");
        let data_str = CString::new(data.to_string().as_bytes()).expect("set_option_string() data can't contain NULL");

        let err = unsafe { mpv::set_option_string(self.handle, name_str.as_ptr(), data_str.as_ptr()) };
        error_to_result(err)
    }

    /// Send a command to the player. Commands are the same as those used in input.conf, except that this function takes parameters in a pre-split form.
    ///
    /// The commands and their parameters are documented in input.rst.
    ///
    /// Does not use OSD and string expansion by default (unlike `command_string()` and input.conf).
    ///
    /// # Params
    /// - `command` - Usually, the first item is the command, and the following items are arguments.
    ///
    /// # Return
    /// A `Result<i32> where the Ok value is the success code returned from mpv.
    pub fn command(&self, command: Vec<impl ToString>) -> Result<i32> {
        let owned_strings: Vec<_> = command.iter().map(|s| CString::new(s.to_string().as_bytes()).expect("command cannot contain NULL")).collect();
        let mut cstrs: Vec<_> = owned_strings.iter().map(|s| s.as_ptr()).collect();
        cstrs.push(null());

        let err = unsafe { mpv::command(self.handle, cstrs.as_mut_ptr()) };
        error_to_result(err)
    }

    /// Same as `command()`, but allows passing structured data in any format.
    ///
    /// In particular, calling `command()` is exactly like calling `command_node()` with the format set to `Format::NodeArray`, and every arg passed in order as `Format::String`.
    ///
    /// Does not use OSD and string expansion by default.
    ///
    /// # Params
    /// The `command` `Node` can be one of the following formats:
    /// - `Node::Array`: Positional arguments. Each entry is an argument using an arbitrary format (the format must be compatible to the used command). Usually, the first item is the command name (as `Format::String`). The order of arguments is as documented in each command description.
    /// - `Node::Map`: Named arguments. This requires at least an entry with the key "name" to be present, which must be a string, and contains the command name. The special entry "_flags" is optional, and if present, must be an array of strings, each being a command prefix to apply. All other entries are interpreted as arguments. They must use the argument names as documented in each command description. Some commands do not support named arguments at all, and must use `Format::Array`.
    ///
    /// # Return
    /// If the function succeeds, Ok(Node) is command-specific return data. Not many commands actually use this at all.
    pub fn command_node(&self, command: Node) -> Result<Node> {
        let mut return_mpv_node = MaybeUninit::uninit();
        
        command.to_mpv(|x| {
            let err = unsafe { mpv::command_node(self.handle, x as *mut mpv_node, return_mpv_node.as_ptr() as *mut mpv_node) };
            error_to_result(err)
        }).map(|_| {
            let ret = unsafe { Node::from_node_ptr(return_mpv_node.as_ptr()) };
            unsafe { mpv::free_node_contents(return_mpv_node.as_mut_ptr()) }
            ret
        })?
    }

    /// This is essentially identical to `command()` but it also returns a result.
    ///
    /// Does not use OSD and string expansion by default.
    ///
    /// # Params
    /// - `command` - Usually, the first item is the command, and the following items are arguments.
    ///
    /// # Return
    /// If the function succeeds, Ok(Node) is command-specific return data. Not many commands actually use this at all.
    pub fn command_ret(&self, command: Vec<impl ToString>) -> Result<Node> {
        let owned_strings: Vec<_> = command.iter().map(|s| CString::new(s.to_string().as_bytes()).expect("command cannot contain NULL")).collect();
        let mut cstrs: Vec<_> = owned_strings.iter().map(|s| s.as_ptr()).collect();
        cstrs.push(null());

        let mut return_mpv_node = MaybeUninit::uninit();

        let err = unsafe { mpv::command_ret(self.handle, cstrs.as_mut_ptr(), return_mpv_node.as_mut_ptr()) };
        error_to_result(err).map(|_| {
            let ret = unsafe { Node::from_node_ptr(return_mpv_node.as_ptr()) };
            unsafe { mpv::free_node_contents(return_mpv_node.as_mut_ptr()) }
            ret
        })?
    }

    /// Same as `command()`, but use input.conf parsing for splitting arguments.
    ///
    /// This is slightly simpler, but also more error prone, since arguments may need quoting/escaping.
    ///
    /// This also has OSD and string expansion enabled by default.
    pub fn command_string(&self, command: impl ToString) -> Result<i32> {
        let owned_string = CString::new(command.to_string().as_bytes()).expect("command cannot contain NULL");

        let err = unsafe { mpv::command_string(self.handle, owned_string.as_ptr()) };
        error_to_result(err)
    }

    /// Set a property to a given value. Properties are essentially variables which can be queried or set at runtime.
    ///
    /// For example, writing to the pause property will actually pause or unpause playback.
    ///
    /// # Params
    ///
    /// If the `Format` of `value` doesn't match with the internal format of the property, access usually will fail with `Error::PropertyFormat`.
    ///
    /// In some cases, the data is automatically converted and access succeeds. For example, `Format::Int64` is always converted to `Format::Double`, and access using `Format::String` usually invokes a string parser.
    ///
    /// The same happens when calling this function with `Format::Node`: the underlying format may be converted to another type if possible.
    ///
    /// Using a format other than `Format::Node` is equivalent to constructing a `Node` with the given format and data and passing it to this function.
    pub fn set_property<T: MpvSend>(&self, name: impl ToString, value: T) -> Result<i32> {
        let owned_name = CString::new(name.to_string().as_bytes()).expect("name cannot contain NULL");

        value.to_mpv(|x| {
            let err = unsafe { mpv::set_property(self.handle, owned_name.as_ptr(), T::MPV_FORMAT.0, x) };
            error_to_result(err)
        })
    }

    /// Convenience function to set a property to a string value.
    ///
    /// This is like calling `set_property()` with MPV_FORMAT_STRING.
    pub fn set_property_string(&self, name: impl ToString, value: impl ToString) -> Result<i32> {
        let owned_name = CString::new(name.to_string().as_bytes()).expect("name cannot contain NULL");
        let owned_value = CString::new(value.to_string().as_bytes()).expect("value cannot contain NULL");

        let err = unsafe { mpv::set_property_string(self.handle, owned_name.as_ptr(), owned_value.as_ptr()) };
        error_to_result(err)
    }

    /// Convenience function to delete a property.
    ///
    /// This is equivalent to running the command "del \[name\]".
    pub fn del_property(&self, name: impl ToString) -> Result<i32> {
        let owned_name = CString::new(name.to_string().as_bytes()).expect("name cannot contain NULL");

        let err = unsafe { mpv::del_property(self.handle, owned_name.as_ptr()) };
        error_to_result(err)
    }

    /// Read the value of the given property.
    ///
    /// If the format doesn't match with the internal format of the property, access usually will fail with `Error::PropertyFormat`.
    ///
    /// In some cases, the data is automatically converted and access succeeds. For example, `Format::Int64` is always converted to `Format::Double`, and access using `Format::String` usually invokes a string formatter.
    pub fn get_property<T: MpvSend>(&self, name: impl ToString) -> Result<T> {
        let owned_name = CString::new(name.to_string().as_bytes()).expect("name cannot contain NULL");

        unsafe {
            T::from_mpv(|x| {
                let err = mpv::get_property(self.handle, owned_name.as_ptr(), T::MPV_FORMAT.0, x);
                error_to_result(err)
            })
        }
    }

    /// Return the value of the property with the given name as string. This is equivalent to `get_property()` with `Format::String`.
    ///
    /// # Return
    /// On error, `Error::Generic` is returned. Use `get_property()` if you want fine-grained error reporting.
    pub fn get_property_string(&self, name: impl ToString) -> Result<String> {
        let owned_name = CString::new(name.to_string().as_bytes()).expect("name cannot contain NULL");

        let cstr = unsafe { mpv::get_property_string(self.handle, owned_name.as_ptr()) };

        if cstr.is_null() {
            return Err(Error::Generic)
        }

        let string = unsafe { CStr::from_ptr(cstr) }.to_string_lossy().to_string();

        unsafe { mpv::free(cstr as *mut c_void) };

        Ok(string)
    }

    /// Return the property as "OSD" formatted string. This is the same as `get_property_string()`, but using `Format::OsdString`.
    ///
    /// # Return
    /// On error, `Error::Generic` is returned. Use `get_property()` if you want fine-grained error reporting.
    pub fn get_property_osd_string(&self, name: impl ToString) -> Result<String> {
        let owned_name = CString::new(name.to_string().as_bytes()).expect("name cannot contain NULL");

        let cstr = unsafe { mpv::get_property_osd_string(self.handle, owned_name.as_ptr()) };

        if cstr.is_null() {
            return Err(Error::Generic)
        }

        let string = unsafe { CStr::from_ptr(cstr) }.to_string_lossy().to_string();

        unsafe { mpv::free(cstr as *mut c_void) };

        Ok(string)
    }

    /// Enable or disable the given event.
    ///
    /// Some events are enabled by default. Some events can't be disabled.
    ///
    /// (Informational note: currently, all events are enabled by default, except MPV_EVENT_TICK.)
    pub fn request_event(&self, event_id: EventId, enable: bool) -> Result<i32> {
        let err = unsafe { mpv::request_event(self.handle, event_id.0, if enable { 1 } else { 0 }) };
        error_to_result(err)
    }

    /// Enable or disable receiving of log messages. These are the messages the command line player prints to the terminal. This call sets the minimum required log level for a message to be received with `Event::LogMessage`.
    ///
    /// # Params
    /// - min_level: Minimal log level as string.
    ///   - Valid log levels: no fatal error warn info v debug trace
    ///
    ///   The value "no" disables all messages. This is the default.
    ///
    /// TODO: Make this accept Rusty LogLevels.
    pub fn request_log_messages(&self, min_level: &str) -> Result<i32> {
        let cstr = CString::from_str(min_level)?;

        let err = unsafe { mpv::request_log_messages(self.handle, cstr.as_ptr()) };
        error_to_result(err)
    }

    /// Wait for the next event, or until the timeout expires, or if another thread makes a call to mpv_wakeup().
    ///
    /// Passing 0 as timeout will never wait, and is suitable for polling.
    ///
    /// # Params
    /// - `timeout`: Timeout in seconds, after which the function returns even if no event was received. An `Event::None` is returned on timeout.
    ///   - A value of 0 will disable waiting.
    ///   - Negative values will wait with an infinite timeout.
    ///
    /// # Warning
    /// The internal event queue has a limited size (per client handle). If you don't empty the event queue quickly enough with `wait_event()`, it will overflow and silently discard further events. If this happens, making asynchronous requests will fail as well (with `Error::EventQueueFull`).
    ///
    /// # Concurrency
    /// Only one thread is allowed to call this on the same `Handle` at a time. The API won't complain if more than one thread calls this, but it will cause race conditions in the client when accessing the shared `mpv_event` struct.
    ///
    /// Note that most other API functions are not restricted by this, and no API function internally calls `wait_event()`. Additionally, concurrent calls to different `Handle`s are always safe.
    pub fn wait_event(&self, timeout: f64) -> Result<Event> {
        Event::from_ptr(unsafe { mpv::wait_event(self.handle, timeout) })
    }

    /// Interrupt the current `wait_event()` call.
    ///
    /// This will wake up the thread currently waiting in `wait_event()`. If no thread is waiting, the next `wait_event()` call will return immediately (this is to avoid lost wakeups).
    ///
    /// `wait_event()` will receive an `Event::None` if it's woken up due to this call. But note that this dummy event might be skipped if there are already other events queued. All what counts is that the waiting thread is woken up at all.
    pub fn wakeup(&self) {
        unsafe { mpv::wakeup(self.handle) }
    }

    /// A hook is like a synchronous event that blocks the player. You register a hook handler with this function. You will get an event, which you need to handle, and once things are ready, you can let the player continue with `hook_continue()`.
    ///
    /// Currently, hooks can't be removed explicitly. But they will be implicitly removed if the mpv_handle it was registered with is destroyed. This also continues the hook if it was being handled by the destroyed mpv_handle (but this should be avoided, as it might mess up order of hook execution).
    ///
    /// Hook handlers are ordered globally by priority and order of registration. Handlers for the same hook with same priority are invoked in order of registration (the handler registered first is run first). Handlers with lower priority are run first (which seems backward).
    ///
    /// See the "Hooks" section in the manpage to see which hooks are currently defined.
    ///
    /// Some hooks might be reentrant (so you get multiple `Event::Hook` for the same hook). If this can happen for a specific hook type, it will be explicitly documented in the manpage.
    ///
    /// Only the `Handle` on which this was called will receive the hook events, or can "continue" them.
    ///
    /// # Params
    /// - `userdata`: This will be used for the `Event::Hook.userdata` field for the received `Event::Hook` events. If you have no use for this, pass 0.
    /// - `name`: The hook name. This should be one of the documented names. But if the name is unknown, the hook event will simply be never raised.
    /// - `priority`: See remarks above. Use 0 as a neutral default.
    pub fn hook_add(&self, userdata: u64, name: String, priority: i32) -> Result<i32> {
        let owned_name = CString::new(name.to_string().as_bytes()).expect("name cannot contain NULL");

        let err = unsafe { mpv::hook_add(self.handle, userdata, owned_name.as_ptr(), priority) };
        error_to_result(err)
    }

    /// Respond to an `Event::Hook` event. You must call this after you have handled the event. There is no way to "cancel" or "stop" the hook.
    ///
    /// Calling this will typically unblock the player for whatever the hook is responsible for (e.g. for the "on_load" hook it lets it continue playback).
    ///
    /// # Params
    /// - `id`: This must be the value of the `Event::Hook.id` field for the corresponding `Event::Hook`.
    ///
    /// # Warning
    /// It is explicitly undefined behavior to call this more than once for each `Event::Hook`, to pass an incorrect ID, or to call this on a `Handle` different from the one that registered the handler and received the event.
    pub fn hook_continue(&self, id: u64) -> Result<i32> {
        let err = unsafe { mpv::hook_continue(self.handle, id) };
        error_to_result(err)
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.destroyed {
            unsafe { mpv::destroy(self.handle) };
        }
    }
}