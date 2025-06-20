//! Definition and implementation of [`Handle`], this crate's primary interface to mpv.

use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::ptr::null;

use libmpv_client_sys as mpv;
use libmpv_client_sys::mpv_node;
use crate::*;
use crate::error::{error_to_result, error_to_result_code};
use crate::event::LogLevel;
use crate::types::traits::{MpvRecv, MpvSend, MpvSendInternal};

/// The primary interface to mpv.
///
/// This [`Handle`] must be created by mpv; usually it is passed in from mpv to a cplugin via `mpv_open_cplugin(*mpv_handle)`.
/// See [`Handle::from_ptr()`] for an example.
pub struct Handle {
    handle: *mut mpv_handle
}

impl Handle {
    /// Creates a [`Handle`] from the provided pointer to a [`mpv_handle`].
    ///
    /// This [`mpv_handle`] must be created by mpv, usually passed in via mpv's call into `mpv_open_cplugin(*mpv_handle)`.
    ///
    /// # Example
    /// ```
    ///# use libmpv_client::*;
    ///#
    /// #[unsafe(no_mangle)]
    /// extern "C" fn mpv_open_cplugin(ptr: *mut mpv_handle) -> std::os::raw::c_int {
    ///     let handle = Handle::from_ptr(ptr);
    ///     // ...
    ///#     0
    /// }
    /// ```
    #[must_use]
    pub fn from_ptr(handle: *mut mpv_handle) -> Self {
        Handle {
            handle
        }
    }

    /// Return the [`MPV_CLIENT_API_VERSION`](libmpv_client_sys::MPV_CLIENT_API_VERSION) the mpv source has been compiled with.
    pub fn client_api_version() -> u64 {
        unsafe { mpv::client_api_version() as u64 }
    }

    /// Return the name of this client [`Handle`].
    ///
    /// Every client has its own unique name, which is mostly used for user interface purposes.
    pub fn client_name(&self) -> Result<String> {
        let c_str = unsafe { CStr::from_ptr(mpv::client_name(self.handle)) };
        Ok(c_str.to_str()?.to_string())
    }

    /// Return the ID of this client [`Handle`].
    ///
    /// Every client has its own unique ID. This ID is never reused by the core, even if the [`Handle`] at hand gets destroyed and new handles get allocated.
    ///
    /// Some mpv APIs (not necessarily all) accept a name in the form `@id` in addition to the proper [`Handle::client_name()`], where `id` is the ID in decimal form (e.g. `@123`).
    /// For example, the [`script-message-to`](https://mpv.io/manual/stable/#command-interface-script-message-to[-]]])
    /// command takes the client name as the first argument but also accepts the client ID formatted in this manner.
    pub fn client_id(&self) -> i64 {
        unsafe { mpv::client_id(self.handle) }
    }

    /// Create a new mpv instance and an associated client API [`Client`] to control the mpv instance.
    ///
    /// This instance is in a pre-initialized state and needs to be initialized to be actually used with most other API functions.
    ///
    /// Some API functions will return [`Error::Uninitialized`] in the uninitialized state.
    /// You can call [`Handle::set_property()`] to set initial options.
    /// After this, call [`Client::initialize()`] to start the player, and then use e.g. [`Handle::command()`] to start playback of a file.
    ///
    /// The point of separating [`Client`] creation and actual initialization is that you can configure things which can't be changed during runtime.
    ///
    /// Unlike the command line player, this will have initial settings suitable for embedding in applications. The following settings are different:
    /// - `stdin`/`stdout`/`stderr` and the terminal will never be accessed.
    ///   This is equivalent to setting the [`--terminal=no`](https://mpv.io/manual/stable/#options-terminal) option. (Technically, this also suppresses C signal handling.)
    /// - No config files will be loaded. This is roughly equivalent to using [`--no-config`](https://mpv.io/manual/stable/#options-no-config).
    ///   Since libmpv 1.15, you can actually re-enable this option, which will make libmpv load config files during [`Client::initialize()`].
    ///   If you do this, you are strongly encouraged to set the [`config-dir`](https://mpv.io/manual/stable/#options-config-dir) option too.
    ///   (Otherwise it will load the mpv command line player's config.)
    /// - Idle mode is enabled, which means the playback core will enter idle mode if there are no more files to play on the internal playlist, instead of exiting.
    ///   This is equivalent to the [`--idle`](https://mpv.io/manual/stable/#options-idle) option.
    /// - Disable parts of input handling.
    /// - Most of the different settings can be viewed with the command line player by running `mpv --show-profile=libmpv`.
    ///
    /// All this assumes that API users want an mpv instance that is strictly isolated from the command line player's configuration, user settings, and so on.
    /// You can re-enable disabled features by setting the appropriate options.
    ///
    /// The mpv command line parser is not available through this API, but you can set individual options with [`Handle::set_property()`].
    /// Files for playback must be loaded with [`Handle::command()`] or others.
    ///
    /// # Concurrency
    /// Note that you should avoid doing concurrent accesses on the uninitialized client handle.
    /// (Whether concurrent access is definitely allowed or not has yet to be decided by mpv.)
    pub fn create() -> Client {
        let handle = unsafe { mpv::create() };
        Client(Handle::from_ptr(handle))
    }

    /// Create a new [`Client`] connected to the same player core as `self`.
    /// This context has its own event queue, its own [`Handle::request_event()`] state, its own [`Handle::request_log_messages()`] state,
    /// its own set of observed properties, and its own state for asynchronous operations. Otherwise, everything is shared.
    ///
    /// This client should be destroyed with [`Client::destroy()`] if no longer needed.
    /// The core will live as long as there is at least 1 handle referencing it.
    /// Any handle can make the core quit, which will result in every handle receiving [`Event::Shutdown`].
    pub fn create_client(&self, name: &str) -> Result<Client> {
        let name_str = CString::new(name)?;

        let handle = unsafe { mpv::create_client(self.handle, name_str.as_ptr()) };
        Ok(Client(Handle::from_ptr(handle)))
    }

    /// This is the same as [`Handle::create_client()`], but the created [`mpv_handle`] is treated as a weak reference.
    /// If all handles referencing a core are weak references, the core is automatically destroyed. (This still goes through normal shutdown, of course.
    /// Effectively, if the last non-weak handle is destroyed, then the weak handles receive [`Event::Shutdown`] and are asked to terminate as well.)
    ///
    /// Note if you want to use this like refcounting: you have to be aware that [`Client::terminate_destroy()`] _and_ [`Client::destroy()`]
    /// for the last non-weak [`mpv_handle`] will block until all weak handles are destroyed.
    pub fn create_weak_client(&self, name: &str) -> Result<Client> {
        let name_str = CString::new(name)?;

        let handle = unsafe { mpv::create_weak_client(self.handle, name_str.as_ptr()) };
        Ok(Client(Handle::from_ptr(handle)))
    }

    /// Load a config file. This parses the file and sets every entry in the config file's default section as if [`Handle::set_option()`] is called.
    ///
    /// The filename should be an absolute path. If it isn't, the actual path used is unspecified. (Note: an absolute path starts with '`/`' on UNIX.)
    /// If the file wasn't found, [`Error::InvalidParameter`] is returned.
    ///
    /// If a fatal error happens when parsing a config file, [`Error::OptionError`] is returned.
    /// Errors when setting options as well as other types or errors are ignored (even if options do not exist).
    /// You can still try to capture the resulting error messages with [`Handle::request_log_messages()`].
    /// Note that it's possible that some options were successfully set even if any of these errors happen.
    pub fn load_config_file(&self, filename: &str) -> Result<()> {
        let filename_str = CString::new(filename)?;

        let err = unsafe { mpv::load_config_file(self.handle, filename_str.as_ptr()) };
        error_to_result(err)
    }

    /// Return the internal time in nanoseconds. This has an arbitrary start offset but will never wrap or go backwards.
    ///
    /// Note that this is always the real time and doesn't necessarily have to do with playback time.
    /// For example, playback could go faster or slower due to playback speed, or due to playback being paused.
    /// Use the `time-pos` property instead to get the playback status.
    ///
    /// Unlike other libmpv APIs, this can be called at absolutely any time (even within wakeup callbacks), as long as the [`Handle`] is valid.
    ///
    /// Safe to be called from mpv render API threads.
    pub fn get_time_ns(&self) -> i64 {
        unsafe { mpv::get_time_ns(self.handle) }
    }

    /// Same as [`Handle::get_time_ns`] but in microseconds.
    pub fn get_time_us(&self) -> i64 {
        unsafe { mpv::get_time_us(self.handle) }
    }

    /// Set an option. Note that you can't normally set options during runtime. It works in an uninitialized state (see [`Handle::create()`]), and in some cases in at runtime.
    ///
    /// Using a format other than [`Node`] is equivalent to constructing a [`Node`] with the given format and data and passing it to this function.
    ///
    /// # Example
    /// ```
    ///# #![allow(deprecated)]
    ///# use libmpv_client::*;
    ///#
    ///# fn example_func(ptr: *mut mpv_handle) -> Result<()> {
    ///#     let handle = Handle::from_ptr(ptr);
    /// handle.set_option("idle", "yes")?;
    ///#     Ok(())
    ///# }
    /// ```
    #[deprecated = "For most purposes, this is not needed anymore.\
    \
    Starting with mpv version 0.21.0 (version 1.23) most options can be set with `Handle::set_property()` (and related functions), and even before `Handle::initialize()`.\
    In some obscure corner cases, using this function to set options might still be required (see \"Inconsistencies between options and properties\" in the manpage).\
    Once these are resolved, the option setting functions might be fully deprecated."]
    pub fn set_option<T: MpvSend>(&self, name: &str, data: T) -> Result<()> {
        let name_str = CString::new(name)?;

        data.to_mpv(|x| {
            let err = unsafe { mpv::set_option(self.handle, name_str.as_ptr(), T::MPV_FORMAT.0, x) };
            error_to_result_code(err)
        }).map(|_| ())
    }

    /// Send a command to the player. Commands are the same as those used in `input.conf`, except that this function takes parameters in a pre-split form.
    ///
    /// The commands and their parameters are documented in input.rst.
    ///
    /// Does not use OSD and string expansion by default (unlike [`Handle::command_string()`] and input.conf).
    ///
    /// # Params
    /// - `command` - Usually, the first item is the command, and the following items are arguments.
    ///
    /// # Example
    /// ```
    ///# use libmpv_client::*;
    ///#
    ///# fn example_func(ptr: *mut mpv_handle) -> Result<()> {
    ///#     let handle = Handle::from_ptr(ptr);
    /// handle.command(&["script-message-to", "commands", "type", "seek absolute-percent", "6"])?;
    ///#     Ok(())
    ///# }
    /// ```
    pub fn command(&self, command: &[impl AsRef<str>]) -> Result<()> {
        let mut owned_strings = Vec::with_capacity(command.len());
        for s in command {
            owned_strings.push(CString::new(s.as_ref())?);
        }

        let mut cstrs: Vec<_> = owned_strings.iter().map(|s| s.as_ptr()).collect();
        cstrs.push(null());

        let err = unsafe { mpv::command(self.handle, cstrs.as_mut_ptr()) };
        error_to_result(err)
    }

    /// Same as [`Handle::command_ret()`], but allows passing structured data in any format.
    ///
    /// In particular, calling [`Handle::command()`] is exactly like calling [`Handle::command_node()`] with the format set to [`NodeArray`],
    /// and every arg passed in order as [`String`].
    ///
    /// Does not use OSD and string expansion by default.
    ///
    /// # Params
    /// The `command` [`Node`] can be one of the following formats:
    /// - [`Node::Array`]: Positional arguments. Each entry is an argument using an arbitrary format (the format must be compatible with the used command).
    ///   Usually, the first item is the command name (as a [`Node::String`]). The order of arguments is as documented in each command description.
    /// - [`Node::Map`]: Named arguments. This requires at least an entry with the key "name" to be present, which must be a string and contain the command name.
    ///   The special entry "_flags" is optional, and if present, must be an array of strings, each being a command prefix to apply.
    ///   All other entries are interpreted as arguments.
    ///   They must use the argument names as documented in each command description. Some commands do not support named arguments at all and must use [`Node::Array`].
    ///
    /// # Return
    /// If the function succeeds, [`Result<Node>`] is command-specific return data. Few commands actually use this.
    ///
    /// # Example
    /// ```
    ///# use libmpv_client::*;
    ///#
    ///# fn example_func(ptr: *mut mpv_handle) -> Result<()> {
    ///#     let handle = Handle::from_ptr(ptr);
    /// // For convenience, you use node_array!(), which accepts any arbitrary types
    /// // implementing Into<Node> and produces a Node::Array...
    /// handle.command_node(node_array!("frame-step", 20, "mute"))?;
    ///
    /// // ...or node_map!(), which is similar but takes (Into<String>, Into<Node>) tuples
    /// // and produces a Node::Map.
    /// handle.command_node(node_map! {
    ///     ("name", "show-text"),
    ///     ("text", "peekaboo!"),
    ///     ("duration", 500),
    /// })?;
    ///#     Ok(())
    ///# }
    /// ```
    pub fn command_node(&self, command: Node) -> Result<Node> {
        let mut return_mpv_node = MaybeUninit::uninit();
        
        command.to_mpv(|x| {
            let err = unsafe { mpv::command_node(self.handle, x as *mut mpv_node, return_mpv_node.as_ptr() as *mut mpv_node) };
            error_to_result_code(err)
        }).and_then(|_| {
            let ret = unsafe { Node::from_node_ptr(return_mpv_node.as_ptr()) };
            unsafe { mpv_free_node_contents(return_mpv_node.as_mut_ptr()) }
            ret
        })
    }

    /// This is essentially identical to [`Handle::command()`], but it also returns a result.
    ///
    /// Does not use OSD and string expansion by default.
    ///
    /// # Params
    /// - `command` - Usually, the first item is the command, and the following items are arguments.
    ///
    /// # Return
    /// If the function succeeds, [`Result<Node>`] is command-specific return data. Few commands actually use this.
    pub fn command_ret(&self, command: &[impl AsRef<str>]) -> Result<Node> {
        let mut owned_strings = Vec::with_capacity(command.len());
        for s in command {
            owned_strings.push(CString::new(s.as_ref())?);
        }

        let mut cstrs: Vec<_> = owned_strings.iter().map(|s| s.as_ptr()).collect();
        cstrs.push(null());
        
        let mut return_mpv_node = MaybeUninit::uninit();

        let err = unsafe { mpv::command_ret(self.handle, cstrs.as_mut_ptr(), return_mpv_node.as_mut_ptr()) };
        error_to_result_code(err).and_then(|_| {
            let ret = unsafe { Node::from_node_ptr(return_mpv_node.as_ptr()) };
            unsafe { mpv_free_node_contents(return_mpv_node.as_mut_ptr()) }
            ret
        })
    }

    /// Same as [`Handle::command()`], but use input.conf parsing for splitting arguments.
    ///
    /// This is slightly simpler, but also more error-prone, since arguments may need quoting/escaping.
    ///
    /// This also has OSD and string expansion enabled by default.
    pub fn command_string(&self, command: &str) -> Result<()> {
        let owned_string = CString::new(command)?;

        let err = unsafe { mpv::command_string(self.handle, owned_string.as_ptr()) };
        error_to_result(err)
    }

    /// Set a property to a given value.
    ///
    /// Properties are essentially variables that can be queried or set at runtime. For example, writing to the pause property will actually pause or unpause playback.
    ///
    /// # Params
    /// If the [`MpvFormat::MPV_FORMAT`] of `value` doesn't match with the internal [`mpv_format`](libmpv_client_sys::mpv_format) format of the property,
    /// access usually will fail with [`Error::PropertyFormat`].
    ///
    /// In some cases, the data is automatically converted and access succeeds. For example, mpv converts [`i64`] to [`f64`],
    /// and access using [`String`] usually invokes a string parser.
    ///
    /// The same happens when calling this function with [`Node`]: the underlying format may be converted to another type if possible.
    ///
    /// Using a format other than [`Node`] is equivalent to constructing a [`Node`] with the given format and data and passing it to this function.
    ///
    /// # Example
    /// ```
    ///# use libmpv_client::*;
    ///#
    ///# fn example_func(ptr: *mut mpv_handle) -> Result<()> {
    ///#     let handle = Handle::from_ptr(ptr);
    /// handle.set_property("chapter", 3)?;
    ///#     Ok(())
    ///# }
    /// ```
    pub fn set_property<T: MpvSend>(&self, name: &str, value: T) -> Result<()> {
        let owned_name = CString::new(name)?;

        value.to_mpv(|x| {
            let err = unsafe { mpv::set_property(self.handle, owned_name.as_ptr(), T::MPV_FORMAT.0, x) };
            error_to_result_code(err)
        }).map(|_| ())
    }

    /// Convenience function to delete a property.
    ///
    /// This is equivalent to running the command `del [name]`.
    pub fn del_property(&self, name: &str) -> Result<()> {
        let owned_name = CString::new(name)?;

        let err = unsafe { mpv::del_property(self.handle, owned_name.as_ptr()) };
        error_to_result(err)
    }

    /// Read the value of the given property.
    ///
    /// If the [`MpvFormat::MPV_FORMAT`] of the requested type doesn't match with the internal [`mpv_format`](libmpv_client_sys::mpv_format) format of the property,
    /// access usually will fail with [`Error::PropertyFormat`].
    ///
    /// In some cases, the data is automatically converted and access succeeds. For example, [`i64`] is always converted to [`f64`],
    /// and access using [`String`] usually invokes a string formatter.
    ///
    /// # Example
    /// ```
    ///# use libmpv_client::*;
    ///#
    ///# fn example_func(ptr: *mut mpv_handle) -> Result<()> {
    ///#     let handle = Handle::from_ptr(ptr);
    /// // use turbofish...
    /// let duration = handle.get_property::<f64>("duration")?;
    /// // or explicitly type the assignment...
    /// let node: Node = handle.get_property("metadata")?;
    ///#     Ok(())
    ///# }
    /// ```
    pub fn get_property<T: MpvRecv>(&self, name: &str) -> Result<T> {
        let owned_name = CString::new(name)?;

        unsafe {
            T::from_mpv(|x| {
                let err = mpv::get_property(self.handle, owned_name.as_ptr(), T::MPV_FORMAT.0, x);
                error_to_result_code(err)
            })
        }
    }

    /// Get a notification whenever the given property changes.
    ///
    /// You will receive updates as [`Event::PropertyChange`]. Note that this is not very precise: for some properties, it may not send updates even if the property changed.
    /// This depends on the property, and it's a valid feature request to ask for better update handling of a specific property.
    /// (For some properties, like [`clock`](https://mpv.io/manual/stable/#command-interface-clock), which shows the wall clock, this mechanism doesn't make too much sense anyway.)
    ///
    /// Property changes are coalesced: the change events are returned only once the event queue becomes empty
    /// (e.g., [`Handle::wait_event()`] would block or return [`Event::None`]), and then only one event per changed property is returned.
    ///
    /// You always get an initial change notification. This is meant to initialize the user's state to the current value of the property.
    ///
    /// Normally, change events are sent only if the property value changes within the requested format.
    /// [`PropertyChange.value`](field@event::PropertyChange::value) will contain the [`PropertyValue`](event::PropertyValue).
    ///
    /// If the property is observed with the format parameter set to [`PropertyValue::None`](event::PropertyValue::None), you get low-level notifications whether the property _may_ have changed.
    /// With this mode, you will have to determine yourself whether the property really changed. On the other hand, this mechanism can be faster and use fewer resources.
    ///
    /// Observing a property that doesn't exist is allowed. (Although it may still cause some sporadic change events.)
    ///
    /// Keep in mind that you will get [`Event::PropertyChange`] even if you change a property yourself.
    /// Try to avoid endless feedback loops, which could happen if you react to the change notifications triggered by your own change.
    ///
    /// Only the [`Handle`] on which this was called will receive [`Event::PropertyChange`] events or can unobserve them.
    ///
    /// # Warning
    /// If a property is unavailable or retrieving it caused an error, [`Event::PropertyChange`]'s [`PropertyChange.value`](field@event::PropertyChange::value) will be [`PropertyValue::None`](event::PropertyValue::None),
    /// even if the format parameter was set to a different value.
    ///
    /// # Params
    /// - `userdata`: This will be used for the [`PropertyChange.userdata`](field@event::PropertyChange::userdata) field for the received [`Event::PropertyChange`] events.
    // (Also see the section about asynchronous calls, although this function is somewhat different from actual asynchronous calls.)
    ///
    ///   If you have no use for this, pass 0.
    ///
    ///
    /// Also see [`Handle::unobserve_property()`].
    ///
    /// # Example
    /// ```
    ///# use libmpv_client::*;
    ///#
    ///# fn example_func(ptr: *mut mpv_handle) -> Result<()> {
    ///#     let handle = Handle::from_ptr(ptr);
    /// // you can set userdata = 0 if you don't plan un unobserving the value later
    /// handle.observe_property("playtime-remaining", Format::DOUBLE, 0)?;
    ///#     Ok(())
    ///# }
    /// ```
    pub fn observe_property(&self, name: &str, format: Format, userdata: u64) -> Result<()> {
        let owned_name = CString::new(name)?;

        let err = unsafe { mpv::observe_property(self.handle, userdata, owned_name.as_ptr(), format.0) };
        error_to_result(err)
    }

    /// Undo [`Handle::observe_property`].
    ///
    /// This will remove all observed properties for which the given number was passed as `userdata` to [`Handle::observe_property()`].
    ///
    /// # Params
    /// - `userdata`: `userdata` that was passed to [`Handle::observe_property()`]
    ///
    /// # Returns
    /// [`Result<i32>`] contains the number of properties removed on success.
    ///
    /// # Example
    /// ```
    ///# use libmpv_client::*;
    ///#
    ///# fn example_func(ptr: *mut mpv_handle) -> Result<()> {
    ///#     let handle = Handle::from_ptr(ptr);
    /// // if you want to later unobserve a property, you must provide a userdata
    /// let media_title_userdata: u64 = 12345; // arbitrary, user-defined value
    /// handle.observe_property("media-title", Format::STRING, media_title_userdata)?;
    ///
    /// // later...
    /// handle.unobserve_property(media_title_userdata)?;
    ///#     Ok(())
    ///# }
    /// ```
    pub fn unobserve_property(&self, userdata: u64) -> Result<i32> {
        let err = unsafe { mpv::unobserve_property(self.handle, userdata) };
        error_to_result_code(err)
    }

    /// Enable or disable an [`Event`] given its [`EventId`].
    ///
    /// Some events are enabled by default. Some events can't be disabled.
    ///
    /// (Informational note: currently, all events are enabled by default, except [`Event::Tick`].)
    pub fn request_event(&self, event_id: EventId, enable: bool) -> Result<()> {
        let err = unsafe { mpv::request_event(self.handle, event_id.0, if enable { 1 } else { 0 }) };
        error_to_result(err)
    }

    /// Enable or disable receiving of log messages.
    ///
    /// These are the messages the command line player prints to the terminal.
    /// This call sets the maximum log level for a message to be received with [`Event::LogMessage`].
    ///
    /// # Params
    /// - `max_level`: Maximum log level to subscribe to.
    ///
    /// The value [`LogLevel::None`] disables all messages. This is the default.
    pub fn request_log_messages(&self, max_level: LogLevel) -> Result<()> {
        let err = unsafe { mpv::request_log_messages(self.handle, max_level.to_cstr().as_ptr()) };
        error_to_result(err)
    }

    /// Wait for the next event, or until the timeout expires, or if another thread makes a call to [`Handle::wakeup()`].
    ///
    /// See [`Event`] for the possible events.
    ///
    /// # Params
    /// - `timeout`: Timeout in seconds, after which the function returns even if no event was received. An [`Event::None`] is returned on timeout.
    ///   - A value of 0 will disable waiting and is suitable for polling.
    ///   - Negative values will wait with an infinite timeout.
    ///
    /// # Warning
    /// The internal event queue has a limited size (per client handle). If you don't empty the event queue quickly enough with [`Handle::wait_event()`],
    /// it will overflow and silently discard further events. If this happens, making asynchronous requests will fail as well (with [`Error::EventQueueFull`]).
    ///
    /// # Concurrency
    /// Only one thread is allowed to call this on the same [`Handle`] at a time. The API won't complain if more than one thread calls this,
    /// but it will cause race conditions in the client when accessing the shared `mpv_event` struct.
    ///
    /// Note that most other API functions are not restricted by this, and no API function internally calls [`Handle::wait_event()`].
    /// Additionally, concurrent calls to different [`Handle`]s are always safe.
    ///
    /// # Example
    /// ```
    ///# use libmpv_client::*;
    ///#
    ///# fn example_func(ptr: *mut mpv_handle) -> Result<()> {
    ///#     let handle = Handle::from_ptr(ptr);
    /// match handle.wait_event(0.0)? {
    ///     Event::None => println!("No event was ready yet!"),
    ///     Event::Shutdown => {
    ///         println!("Shutting down!");
    ///         // You must cleanly exit after receiving Event::Shutdown, or else you'll hang mpv.
    ///         return Ok(());
    ///     }
    ///     Event::LogMessage(log_message) => println!("Got a log message: {log_message:?}"),
    ///     event => println!("Got an other event: {event:?}"),
    /// }
    ///#     Ok(())
    ///# }
    /// ```
    /// 
    /// # Warning
    /// cplugins **must** call [`Handle::wait_event()`] at least once after initialization;
    /// mpv will block awaiting a sign of life:.
    ///```
    ///# use std::thread::sleep;
    ///# use std::time::Duration;
    ///# use libmpv_client::Handle;
    ///# use libmpv_client_sys::mpv_handle;
    ///# 
    /// #[unsafe(no_mangle)]
    /// extern "C" fn mpv_open_cplugin(ptr: *mut mpv_handle) -> std::os::raw::c_int {
    ///     let handle = Handle::from_ptr(ptr);
    /// 
    ///     println!("Sleeping 5 seconds pre-wait_event...");
    ///     // mpv will be completely hung during this sleep...
    ///     sleep(Duration::from_secs(5));
    /// 
    ///     // Let mpv know we're alive!
    ///     let _ = handle.wait_event(-1.0);
    /// 
    ///     println!("Sleeping 15 seconds post-wait_event...");
    ///     // mpv will operate normally during this sleep.
    ///     sleep(Duration::from_secs(15));
    /// 
    ///     return 0;
    /// }
    /// ```
    pub fn wait_event(&self, timeout: f64) -> Result<Event> {
        Event::from_ptr(unsafe { mpv::wait_event(self.handle, timeout) })
    }

    /// Interrupt the current [`Handle::wait_event()`] call.
    ///
    /// This will wake up the thread currently waiting in [`Handle::wait_event()`]. If no thread is waiting, the next [`Handle::wait_event()`]
    /// call will return immediately (this is to avoid lost wakeups).
    ///
    /// [`Handle::wait_event()`] will receive an [`Event::None`] if it's woken up due to this call. But note that this dummy event might be
    /// skipped if there are already other events queued. All that counts is that the waiting thread is woken up.
    pub fn wakeup(&self) {
        unsafe { mpv::wakeup(self.handle) }
    }

    /// A hook is like a synchronous event that blocks the player. You register a hook handler with this function. You will get an event,
    /// which you need to handle, and once things are ready, you can let the player continue with [`Handle::hook_continue()`].
    ///
    /// Currently, hooks can't be removed explicitly. But they will be implicitly removed if the [`Handle`] it was registered with is destroyed.
    /// This also continues the hook if it was being handled by the destroyed handle (but this should be avoided, as it might mess up the order of hook execution).
    ///
    /// See [the "Hooks" section in the manpage](https://mpv.io/manual/stable/#hooks) to see which hooks are currently defined.
    ///
    /// Some hooks might be reentrant (so you get multiple [`Event::Hook`] for the same hook). If this can happen for a specific hook type,
    /// it will be explicitly documented in the manpage.
    ///
    /// Only the [`Handle`] on which this was called will receive the hook events or can "continue" them.
    ///
    /// # Priority
    /// Hook handlers are ordered globally by priority and order of registration. Handlers for the same hook with the same priority are invoked
    /// in order of registration (the handler registered first is run first). Handlers with lower priority are run first (which seems backward).
    ///
    /// # Params
    /// - `userdata`: This will be used for the [`Event::Hook.userdata`](field@event::Hook::userdata) field for the received [`Event::Hook`] events.
    ///   If you have no use for this, pass 0.
    /// - `name`: The hook name. This should be [one of the documented names](https://mpv.io/manual/stable/#hooks).
    ///   But if the name is unknown, the hook event will simply never be raised.
    /// - `priority`: See remarks above. Use 0 as a neutral default.
    pub fn hook_add(&self, userdata: u64, name: &str, priority: i32) -> Result<()> {
        let owned_name = CString::new(name)?;

        let err = unsafe { mpv::hook_add(self.handle, userdata, owned_name.as_ptr(), priority) };
        error_to_result(err)
    }

    /// Respond to an [`Event::Hook`] event. You **must** call this after you have handled the event.
    ///
    /// There is no way to "cancel" or "stop" the hook.
    ///
    /// Calling this will typically unblock the player for whatever the hook is responsible for (e.g., for the `on_load` hook it lets it continue playback).
    ///
    /// # Params
    /// - `id`: This must be the value of the [`Hook.id`](field@event::Hook::id) field for the corresponding [`Event::Hook`].
    ///
    /// # Warning
    /// It is explicitly undefined behavior to call this more than once for each [`Event::Hook`], to pass an incorrect ID,
    /// or to call this on a [`Handle`] different from the one that registered the handler and received the event.
    ///
    /// # Example
    /// ```
    ///# use libmpv_client::*;
    ///#
    ///# fn do_something_during_hook() {}
    ///#
    ///# fn example_func(ptr: *mut mpv_handle) -> Result<()> {
    ///#     let handle = Handle::from_ptr(ptr);
    /// match handle.wait_event(0.0)? {
    ///     Event::Hook(hook) => {
    ///         do_something_during_hook();
    ///         // You MUST call hook_continue() on the provided Hook.id,
    ///         // or else you'll hang mpv.
    ///         handle.hook_continue(hook.id)?;
    ///     }
    ///     // ...
    ///     event => {}
    /// }
    ///#     Ok(())
    ///# }
    /// ```
    pub fn hook_continue(&self, id: u64) -> Result<()> {
        let err = unsafe { mpv::hook_continue(self.handle, id) };
        error_to_result(err)
    }
}

/// An owned client created from a [`Handle`].
///
/// Unlike a [`Handle`], it is safe to call [`mpv_destroy`](libmpv_client_sys::destroy) and [`mpv_terminate_destroy`](libmpv_client_sys::terminate_destroy)
/// on an owned [`Client`]; thus the [`Client::destroy()`] and [`Client::terminate_destroy()`] methods are provided here.
///
/// Additionally, since [`mpv_initialize`](libmpv_client_sys::initialize) should only be called on uninitialized instances
/// of mpv, it only makes sense to call it on a created [`Client`], so that is exposed here as [`Client::initialize()`].
///
/// All other usage is the same as [`Handle`].
///
/// # Example
/// ```rust
///# use std::thread::sleep;
///# use std::time::Duration;
///# use libmpv_client::{Event, Handle};
///#
///# use libmpv_client_sys::mpv_handle;
///#
/// #[unsafe(no_mangle)]
/// extern "C" fn mpv_open_cplugin(ptr: *mut mpv_handle) -> std::os::raw::c_int {
///     let handle = Handle::from_ptr(ptr);
///
///     let second_client = handle.create_client("second client").unwrap();
///
///     // Note: in the case of a cplugin, the passed Handle MUST call wait_event
///     // or else mpv will block the entire program waiting for a sign of life.
///     let _ = handle.wait_event(-1.0);
///
///     loop {
///         match second_client.wait_event(0.0) {
///             Err(e) => {
///                 println!("Second client got error: {e:?}");
///             }
///             Ok(event) => {
///                 match event {
///                     Event::Shutdown => {
///                         println!("Goodbye from Rust!");
///
///                         // Clients must be destroyed in a timely manner!
///                         // (though in this case it would get Drop'd anyway...)
///                         second_client.destroy();
///
///                         // Handles require no additional cleanup!
///                         return 0;
///                     },
///                     Event::None => {},
///                     event => {
///                         println!("Second client got event: {event:?}");
///                     },
///                 }
///             }
///         }
///     }
/// }
/// ```
pub struct Client(Handle);

impl Client {
    /// Initialize an uninitialized mpv instance. If the mpv instance is already running, an [`Error`] is returned.
    ///
    /// This function needs to be called to make full use of the client API if the client API handle was created with [`Handle::create()`].
    ///
    /// Only the following options are required to be set _before_ [`Client::initialize()`]:
    /// - options which are only read at initialization time:
    ///   - `config`
    ///   - [`config-dir`](https://mpv.io/manual/stable/#options-config-dir)
    ///   - [`input-conf`](https://mpv.io/manual/stable/#options-input-conf)
    ///   - [`load-scripts`](https://mpv.io/manual/stable/#options-load-scripts)
    ///   - [`script`](https://mpv.io/manual/stable/#options-scripts)
    ///   - [`player-operation-mode`](https://mpv.io/manual/stable/#options-player-operation-mode)
    ///   - `input-app-events` (macOS)
    /// - [all encoding mode options](https://mpv.io/manual/stable/#encoding)
    pub fn initialize(&self) -> Result<()> {
        let err = unsafe { mpv::initialize(self.handle) };
        error_to_result(err)
    }

    /// Disconnect and destroy the [`Client`] and its underlying [`Handle`]. The underlying [`mpv_handle`] will be deallocated with this API call.
    ///
    /// If the last [`mpv_handle`] is detached, the core player is destroyed.
    /// In addition, if there are only weak handles (such as created by [`Handle::create_weak_client()`] or internal scripts), these handles will be sent [`Event::Shutdown`].
    /// This function may block until these clients have responded to the shutdown event, and the core is finally destroyed.
    ///
    /// # Concurrency
    /// Since the underlying [`mpv_handle`] is destroyed somewhere on the way, it's not safe to call other functions concurrently on the same handle.
    ///
    /// # Handles
    /// Note that `mpv_destroy()` cannot be called from a cplugin client. The correct way to terminate (given a [`Handle`]) is to return from
    /// the execution environment in which you were provided a [`mpv_handle`]. In the case of a cplugin, this means returning from `mpv_open_cplugin()`.
    /// mpv will handle cleaning up the underlying client upon return.
    ///
    /// If a [`Handle`] wishes to terminate mpv, send `client.command(&["quit"])` before returning from `mpv_open_cplugin()`.
    pub fn destroy(self) {
        unsafe { mpv::destroy(self.handle) };
        std::mem::forget(self); // forget to prevent Drop from calling destroy a second time
    }

    /// Similar to [`Client::destroy()`], but brings the player and all clients down as well and waits until all of them are destroyed. This function blocks.
    ///
    /// The advantage over [`Client::destroy()`] is that while [`Client::destroy()`] merely detaches the client handle from the player,
    /// this function quits the player, waits until all other clients are destroyed (i.e., all [`mpv_handle`]s are detached), and also waits for the final termination of the player.
    ///
    /// # Concurrency
    /// Since the underlying [`mpv_handle`] is destroyed somewhere on the way, it's not safe to call other functions concurrently on the same handle.
    ///
    /// # Handles
    /// Note that `mpv_destroy()` cannot be called from a cplugin client. The correct way to terminate (given a [`Handle`]) is to return from
    /// the execution environment in which you were provided a [`mpv_handle`]. In the case of a cplugin, this means returning from `mpv_open_cplugin()`.
    /// mpv will handle cleaning up the underlying client upon return.
    ///
    /// If a [`Handle`] wishes to terminate mpv, send `client.command(&["quit"])` before returning from `mpv_open_cplugin()`.
    pub fn terminate_destroy(self) {
        unsafe { mpv::terminate_destroy(self.handle) }
        std::mem::forget(self); // forget to prevent Drop from calling destroy a second time
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { mpv::destroy(self.handle) };
    }
}

impl Deref for Client {
    type Target = Handle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}