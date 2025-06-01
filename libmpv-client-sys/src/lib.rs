#![cfg(not(doctest))]
#![allow(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::invalid_html_tags)]
#![allow(clippy::missing_safety_doc)]
#![warn(missing_docs)]

//! [bindgen](https://docs.rs/bindgen/latest/bindgen/) bindings for libmpv's [`client.h`](https://github.com/mpv-player/mpv/blob/master/include/mpv/client.h).
//!
//! Provides wrappings around the libmpv functions to utilize mpv's `MPV_CPLUGIN_DYNAMIC_SYM` option for [cplugins](https://mpv.io/manual/stable/#c-plugins),
//! which is optional on Linux and required on Windows

mod mpv_funcs;
mod mpv_data;
mod mpv_pfns;

pub use mpv_data::*;

#[cfg(feature = "dyn-sym")]
use mpv_pfns::*;

#[cfg(not(feature = "dyn-sym"))]
use mpv_funcs::*;

/// The version of the mpv `client.h` (encoded as `(((major) << 16) | (minor) | 0UL)`) that this crate was built against.
///
/// Currently, `major` is 2 and `minor` is 5.
pub const EXPECTED_MPV_VERSION: u32 = 131077;

use std::fmt::{Debug, Formatter};

impl Debug for mpv_node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.format {
            mpv_data::mpv_format_MPV_FORMAT_NONE => f.debug_struct("mpv_node").field("format", &self.format).finish(),
            mpv_data::mpv_format_MPV_FORMAT_STRING => f.debug_struct("mpv_node").field("format", &self.format).field("string", unsafe { &self.u.string }).finish(),
            mpv_data::mpv_format_MPV_FORMAT_FLAG => f.debug_struct("mpv_node").field("format", &self.format).field("flag", unsafe { &self.u.flag }).finish(),
            mpv_data::mpv_format_MPV_FORMAT_INT64 => f.debug_struct("mpv_node").field("format", &self.format).field("int64", unsafe { &self.u.int64 }).finish(),
            mpv_data::mpv_format_MPV_FORMAT_DOUBLE => f.debug_struct("mpv_node").field("format", &self.format).field("double", unsafe { &self.u.double_ }).finish(),
            mpv_data::mpv_format_MPV_FORMAT_NODE_ARRAY => f.debug_struct("mpv_node").field("format", &self.format).field("node_array", unsafe { &*self.u.list }).finish(),
            mpv_data::mpv_format_MPV_FORMAT_NODE_MAP => f.debug_struct("mpv_node").field("format", &self.format).field("node_map", unsafe { &*self.u.list }).finish(),
            mpv_data::mpv_format_MPV_FORMAT_BYTE_ARRAY => f.debug_struct("mpv_node").field("format", &self.format).field("byte_array", unsafe { &*self.u.ba }).finish(),
            _ => unreachable!()
        }
    }
}

use paste::paste;

macro_rules! mpv_functions {
    ($(
        $(#[doc = $doc:expr])*
        $rust_name:ident($($param:ident: $param_type:ty),*$(,)?)$( -> $ret_type:ty)?
    );*$(;)?) => {
        #[cfg(feature = "dyn-sym")]
        mod mpv_function_pointers {
            use super::*;

            $(
                paste! {
                    #[unsafe(no_mangle)]
                    pub static mut [<pfn_mpv_ $rust_name>]: Option<unsafe extern "C" fn($($param_type),*)$( -> $ret_type)?> = None;
                }
            )*
        }

        $(
            paste! {
                $(#[doc = $doc])*
                pub unsafe fn $rust_name($($param: $param_type),*)$( -> $ret_type)? {
                    #[cfg(feature = "dyn-sym")]
                    unsafe { [<pfn_mpv_ $rust_name>].expect("mpv function pointers not populated")($($param),*) }
                    #[cfg(not(feature = "dyn-sym"))]
                    unsafe { [<mpv_ $rust_name>]($($param),*) }
                }
            }
        )*
    };
}

mpv_functions! {
    #[doc = " Return the MPV_CLIENT_API_VERSION the mpv source has been compiled with."]
    client_api_version() -> ::std::os::raw::c_ulong;
    #[doc = " Return a string describing the error. For unknown errors, the string\n \"unknown error\" is returned.\n\n @param error error number, see enum mpv_error\n @return A static string describing the error. The string is completely\n         static, i.e. doesn't need to be deallocated, and is valid forever."]
    error_string(error: ::std::os::raw::c_int) -> *const ::std::os::raw::c_char;
    #[doc = " General function to deallocate memory returned by some of the API functions.\n Call this only if it's explicitly documented as allowed. Calling this on\n mpv memory not owned by the caller will lead to undefined behavior.\n\n @param data A valid pointer returned by the API, or NULL."]
    free(data: *mut ::std::os::raw::c_void);
    #[doc = " Return the name of this client handle. Every client has its own unique\n name, which is mostly used for user interface purposes.\n\n @return The client name. The string is read-only and is valid until the\n         mpv_handle is destroyed."]
    client_name(ctx: *mut mpv_handle) -> *const ::std::os::raw::c_char;
    #[doc = " Return the ID of this client handle. Every client has its own unique ID. This\n ID is never reused by the core, even if the mpv_handle at hand gets destroyed\n and new handles get allocated.\n\n IDs are never 0 or negative.\n\n Some mpv APIs (not necessarily all) accept a name in the form \"@<id>\" in\n addition of the proper mpv_client_name(), where \"<id>\" is the ID in decimal\n form (e.g. \"@123\"). For example, the \"script-message-to\" command takes the\n client name as first argument, but also accepts the client ID formatted in\n this manner.\n\n @return The client ID."]
    client_id(ctx: *mut mpv_handle) -> i64;
    #[doc = " Create a new mpv instance and an associated client API handle to control\n the mpv instance. This instance is in a pre-initialized state,\n and needs to be initialized to be actually used with most other API\n functions.\n\n Some API functions will return MPV_ERROR_UNINITIALIZED in the uninitialized\n state. You can call mpv_set_property() (or mpv_set_property_string() and\n other variants, and before mpv 0.21.0 mpv_set_option() etc.) to set initial\n options. After this, call mpv_initialize() to start the player, and then use\n e.g. mpv_command() to start playback of a file.\n\n The point of separating handle creation and actual initialization is that\n you can configure things which can't be changed during runtime.\n\n Unlike the command line player, this will have initial settings suitable\n for embedding in applications. The following settings are different:\n - stdin/stdout/stderr and the terminal will never be accessed. This is\n   equivalent to setting the --no-terminal option.\n   (Technically, this also suppresses C signal handling.)\n - No config files will be loaded. This is roughly equivalent to using\n   --config=no. Since libmpv 1.15, you can actually re-enable this option,\n   which will make libmpv load config files during mpv_initialize(). If you\n   do this, you are strongly encouraged to set the \"config-dir\" option too.\n   (Otherwise it will load the mpv command line player's config.)\n   For example:\n      mpv_set_option_string(mpv, \"config-dir\", \"/my/path\"); // set config root\n      mpv_set_option_string(mpv, \"config\", \"yes\"); // enable config loading\n      (call mpv_initialize() _after_ this)\n - Idle mode is enabled, which means the playback core will enter idle mode\n   if there are no more files to play on the internal playlist, instead of\n   exiting. This is equivalent to the --idle option.\n - Disable parts of input handling.\n - Most of the different settings can be viewed with the command line player\n   by running \"mpv --show-profile=libmpv\".\n\n All this assumes that API users want a mpv instance that is strictly\n isolated from the command line player's configuration, user settings, and\n so on. You can re-enable disabled features by setting the appropriate\n options.\n\n The mpv command line parser is not available through this API, but you can\n set individual options with mpv_set_property(). Files for playback must be\n loaded with mpv_command() or others.\n\n Note that you should avoid doing concurrent accesses on the uninitialized\n client handle. (Whether concurrent access is definitely allowed or not has\n yet to be decided.)\n\n @return a new mpv client API handle. Returns NULL on error. Currently, this\n         can happen in the following situations:\n         - out of memory\n         - LC_NUMERIC is not set to \"C\" (see general remarks)"]
    create() -> *mut mpv_handle;
    #[doc = " Initialize an uninitialized mpv instance. If the mpv instance is already\n running, an error is returned.\n\n This function needs to be called to make full use of the client API if the\n client API handle was created with mpv_create().\n\n Only the following options are required to be set _before_ mpv_initialize():\n      - options which are only read at initialization time:\n        - config\n        - config-dir\n        - input-conf\n        - load-scripts\n        - script\n        - player-operation-mode\n        - input-app-events (macOS)\n      - all encoding mode options\n\n @return error code"]
    initialize(ctx: *mut mpv_handle) -> ::std::os::raw::c_int;
    #[doc = " Disconnect and destroy the mpv_handle. ctx will be deallocated with this\n API call.\n\n If the last mpv_handle is detached, the core player is destroyed. In\n addition, if there are only weak mpv_handles (such as created by\n mpv_create_weak_client() or internal scripts), these mpv_handles will\n be sent MPV_EVENT_SHUTDOWN. This function may block until these clients\n have responded to the shutdown event, and the core is finally destroyed."]
    destroy(ctx: *mut mpv_handle);
    #[doc = " Similar to mpv_destroy(), but brings the player and all clients down\n as well, and waits until all of them are destroyed. This function blocks. The\n advantage over mpv_destroy() is that while mpv_destroy() merely\n detaches the client handle from the player, this function quits the player,\n waits until all other clients are destroyed (i.e. all mpv_handles are\n detached), and also waits for the final termination of the player.\n\n Since mpv_destroy() is called somewhere on the way, it's not safe to\n call other functions concurrently on the same context.\n\n Since mpv client API version 1.29:\n  The first call on any mpv_handle will block until the core is destroyed.\n  This means it will wait until other mpv_handle have been destroyed. If you\n  want asynchronous destruction, just run the \"quit\" command, and then react\n  to the MPV_EVENT_SHUTDOWN event.\n  If another mpv_handle already called mpv_terminate_destroy(), this call will\n  not actually block. It will destroy the mpv_handle, and exit immediately,\n  while other mpv_handles might still be uninitializing.\n\n Before mpv client API version 1.29:\n  If this is called on a mpv_handle that was not created with mpv_create(),\n  this function will merely send a quit command and then call\n  mpv_destroy(), without waiting for the actual shutdown."]
    terminate_destroy(ctx: *mut mpv_handle);
    #[doc = " Create a new client handle connected to the same player core as ctx. This\n context has its own event queue, its own mpv_request_event() state, its own\n mpv_request_log_messages() state, its own set of observed properties, and\n its own state for asynchronous operations. Otherwise, everything is shared.\n\n This handle should be destroyed with mpv_destroy() if no longer\n needed. The core will live as long as there is at least 1 handle referencing\n it. Any handle can make the core quit, which will result in every handle\n receiving MPV_EVENT_SHUTDOWN.\n\n This function can not be called before the main handle was initialized with\n mpv_initialize(). The new handle is always initialized, unless ctx=NULL was\n passed.\n\n @param ctx Used to get the reference to the mpv core; handle-specific\n            settings and parameters are not used.\n            If NULL, this function behaves like mpv_create() (ignores name).\n @param name The client name. This will be returned by mpv_client_name(). If\n             the name is already in use, or contains non-alphanumeric\n             characters (other than '_'), the name is modified to fit.\n             If NULL, an arbitrary name is automatically chosen.\n @return a new handle, or NULL on error"]
    create_client(
        ctx: *mut mpv_handle,
        name: *const ::std::os::raw::c_char,
    ) -> *mut mpv_handle;
    #[doc = " This is the same as mpv_create_client(), but the created mpv_handle is\n treated as a weak reference. If all mpv_handles referencing a core are\n weak references, the core is automatically destroyed. (This still goes\n through normal uninit of course. Effectively, if the last non-weak mpv_handle\n is destroyed, then the weak mpv_handles receive MPV_EVENT_SHUTDOWN and are\n asked to terminate as well.)\n\n Note if you want to use this like refcounting: you have to be aware that\n mpv_terminate_destroy() _and_ mpv_destroy() for the last non-weak\n mpv_handle will block until all weak mpv_handles are destroyed."]
    create_weak_client(
        ctx: *mut mpv_handle,
        name: *const ::std::os::raw::c_char,
    ) -> *mut mpv_handle;
    #[doc = " Load a config file. This loads and parses the file, and sets every entry in\n the config file's default section as if mpv_set_option_string() is called.\n\n The filename should be an absolute path. If it isn't, the actual path used\n is unspecified. (Note: an absolute path starts with '/' on UNIX.) If the\n file wasn't found, MPV_ERROR_INVALID_PARAMETER is returned.\n\n If a fatal error happens when parsing a config file, MPV_ERROR_OPTION_ERROR\n is returned. Errors when setting options as well as other types or errors\n are ignored (even if options do not exist). You can still try to capture\n the resulting error messages with mpv_request_log_messages(). Note that it's\n possible that some options were successfully set even if any of these errors\n happen.\n\n @param filename absolute path to the config file on the local filesystem\n @return error code"]
    load_config_file(
        ctx: *mut mpv_handle,
        filename: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    #[doc = " Return the internal time in nanoseconds. This has an arbitrary start offset,\n but will never wrap or go backwards.\n\n Note that this is always the real time, and doesn't necessarily have to do\n with playback time. For example, playback could go faster or slower due to\n playback speed, or due to playback being paused. Use the \"time-pos\" property\n instead to get the playback status.\n\n Unlike other libmpv APIs, this can be called at absolutely any time (even\n within wakeup callbacks), as long as the context is valid.\n\n Safe to be called from mpv render API threads."]
    get_time_ns(ctx: *mut mpv_handle) -> i64;
    #[doc = " Same as mpv_get_time_ns but in microseconds."]
    get_time_us(ctx: *mut mpv_handle) -> i64;
    #[doc = " Frees any data referenced by the node. It doesn't free the node itself.\n Call this only if the mpv client API set the node. If you constructed the\n node yourself (manually), you have to free it yourself.\n\n If node->format is MPV_FORMAT_NONE, this call does nothing. Likewise, if\n the client API sets a node with this format, this function doesn't need to\n be called. (This is just a clarification that there's no danger of anything\n strange happening in these cases.)"]
    free_node_contents(node: *mut mpv_node);
    #[doc = " Set an option. Note that you can't normally set options during runtime. It\n works in uninitialized state (see mpv_create()), and in some cases in at\n runtime.\n\n Using a format other than MPV_FORMAT_NODE is equivalent to constructing a\n mpv_node with the given format and data, and passing the mpv_node to this\n function.\n\n Note: this is semi-deprecated. For most purposes, this is not needed anymore.\n       Starting with mpv version 0.21.0 (version 1.23) most options can be set\n       with mpv_set_property() (and related functions), and even before\n       mpv_initialize(). In some obscure corner cases, using this function\n       to set options might still be required (see\n       \"Inconsistencies between options and properties\" in the manpage). Once\n       these are resolved, the option setting functions might be fully\n       deprecated.\n\n @param name Option name. This is the same as on the mpv command line, but\n             without the leading \"--\".\n @param format see enum mpv_format.\n @param[in] data Option value (according to the format).\n @return error code"]
    set_option(
        ctx: *mut mpv_handle,
        name: *const ::std::os::raw::c_char,
        format: mpv_format,
        data: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
    #[doc = " Convenience function to set an option to a string value. This is like\n calling mpv_set_option() with MPV_FORMAT_STRING.\n\n @return error code"]
    set_option_string(
        ctx: *mut mpv_handle,
        name: *const ::std::os::raw::c_char,
        data: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    #[doc = " Send a command to the player. Commands are the same as those used in\n input.conf, except that this function takes parameters in a pre-split\n form.\n\n The commands and their parameters are documented in input.rst.\n\n Does not use OSD and string expansion by default (unlike mpv_command_string()\n and input.conf).\n\n @param[in] args NULL-terminated list of strings. Usually, the first item\n                 is the command, and the following items are arguments.\n @return error code"]
    command(
        ctx: *mut mpv_handle,
        args: *mut *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    #[doc = " Same as mpv_command(), but allows passing structured data in any format.\n In particular, calling mpv_command() is exactly like calling\n mpv_command_node() with the format set to MPV_FORMAT_NODE_ARRAY, and\n every arg passed in order as MPV_FORMAT_STRING.\n\n Does not use OSD and string expansion by default.\n\n The args argument can have one of the following formats:\n\n MPV_FORMAT_NODE_ARRAY:\n      Positional arguments. Each entry is an argument using an arbitrary\n      format (the format must be compatible to the used command). Usually,\n      the first item is the command name (as MPV_FORMAT_STRING). The order\n      of arguments is as documented in each command description.\n\n MPV_FORMAT_NODE_MAP:\n      Named arguments. This requires at least an entry with the key \"name\"\n      to be present, which must be a string, and contains the command name.\n      The special entry \"_flags\" is optional, and if present, must be an\n      array of strings, each being a command prefix to apply. All other\n      entries are interpreted as arguments. They must use the argument names\n      as documented in each command description. Some commands do not\n      support named arguments at all, and must use MPV_FORMAT_NODE_ARRAY.\n\n @param[in] args mpv_node with format set to one of the values documented\n                 above (see there for details)\n @param[out] result Optional, pass NULL if unused. If not NULL, and if the\n                    function succeeds, this is set to command-specific return\n                    data. You must call mpv_free_node_contents() to free it\n                    (again, only if the command actually succeeds).\n                    Not many commands actually use this at all.\n @return error code (the result parameter is not set on error)"]
    command_node(
        ctx: *mut mpv_handle,
        args: *mut mpv_node,
        result: *mut mpv_node,
    ) -> ::std::os::raw::c_int;
    #[doc = " This is essentially identical to mpv_command() but it also returns a result.\n\n Does not use OSD and string expansion by default.\n\n @param[in] args NULL-terminated list of strings. Usually, the first item\n                 is the command, and the following items are arguments.\n @param[out] result Optional, pass NULL if unused. If not NULL, and if the\n                    function succeeds, this is set to command-specific return\n                    data. You must call mpv_free_node_contents() to free it\n                    (again, only if the command actually succeeds).\n                    Not many commands actually use this at all.\n @return error code (the result parameter is not set on error)"]
    command_ret(
        ctx: *mut mpv_handle,
        args: *mut *const ::std::os::raw::c_char,
        result: *mut mpv_node,
    ) -> ::std::os::raw::c_int;
    #[doc = " Same as mpv_command, but use input.conf parsing for splitting arguments.\n This is slightly simpler, but also more error prone, since arguments may\n need quoting/escaping.\n\n This also has OSD and string expansion enabled by default."]
    command_string(
        ctx: *mut mpv_handle,
        args: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    #[doc = " Same as mpv_command, but run the command asynchronously.\n\n Commands are executed asynchronously. You will receive a\n MPV_EVENT_COMMAND_REPLY event. This event will also have an\n error code set if running the command failed. For commands that\n return data, the data is put into mpv_event_command.result.\n\n The only case when you do not receive an event is when the function call\n itself fails. This happens only if parsing the command itself (or otherwise\n validating it) fails, i.e. the return code of the API call is not 0 or\n positive.\n\n Safe to be called from mpv render API threads.\n\n @param reply_userdata the value mpv_event.reply_userdata of the reply will\n                       be set to (see section about asynchronous calls)\n @param args NULL-terminated list of strings (see mpv_command())\n @return error code (if parsing or queuing the command fails)"]
    command_async(
        ctx: *mut mpv_handle,
        reply_userdata: u64,
        args: *mut *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    #[doc = " Same as mpv_command_node(), but run it asynchronously. Basically, this\n function is to mpv_command_node() what mpv_command_async() is to\n mpv_command().\n\n See mpv_command_async() for details.\n\n Safe to be called from mpv render API threads.\n\n @param reply_userdata the value mpv_event.reply_userdata of the reply will\n                       be set to (see section about asynchronous calls)\n @param args as in mpv_command_node()\n @return error code (if parsing or queuing the command fails)"]
    command_node_async(
        ctx: *mut mpv_handle,
        reply_userdata: u64,
        args: *mut mpv_node,
    ) -> ::std::os::raw::c_int;
    #[doc = " Signal to all async requests with the matching ID to abort. This affects\n the following API calls:\n\n      mpv_command_async\n      mpv_command_node_async\n\n All of these functions take a reply_userdata parameter. This API function\n tells all requests with the matching reply_userdata value to try to return\n as soon as possible. If there are multiple requests with matching ID, it\n aborts all of them.\n\n This API function is mostly asynchronous itself. It will not wait until the\n command is aborted. Instead, the command will terminate as usual, but with\n some work not done. How this is signaled depends on the specific command (for\n example, the \"subprocess\" command will indicate it by \"killed_by_us\" set to\n true in the result). How long it takes also depends on the situation. The\n aborting process is completely asynchronous.\n\n Not all commands may support this functionality. In this case, this function\n will have no effect. The same is true if the request using the passed\n reply_userdata has already terminated, has not been started yet, or was\n never in use at all.\n\n You have to be careful of race conditions: the time during which the abort\n request will be effective is _after_ e.g. mpv_command_async() has returned,\n and before the command has signaled completion with MPV_EVENT_COMMAND_REPLY.\n\n @param reply_userdata ID of the request to be aborted (see above)"]
    abort_async_command(ctx: *mut mpv_handle, reply_userdata: u64);
    #[doc = " Set a property to a given value. Properties are essentially variables which\n can be queried or set at runtime. For example, writing to the pause property\n will actually pause or unpause playback.\n\n If the format doesn't match with the internal format of the property, access\n usually will fail with MPV_ERROR_PROPERTY_FORMAT. In some cases, the data\n is automatically converted and access succeeds. For example, MPV_FORMAT_INT64\n is always converted to MPV_FORMAT_DOUBLE, and access using MPV_FORMAT_STRING\n usually invokes a string parser. The same happens when calling this function\n with MPV_FORMAT_NODE: the underlying format may be converted to another\n type if possible.\n\n Using a format other than MPV_FORMAT_NODE is equivalent to constructing a\n mpv_node with the given format and data, and passing the mpv_node to this\n function. (Before API version 1.21, this was different.)\n\n Note: starting with mpv 0.21.0 (client API version 1.23), this can be used to\n       set options in general. It even can be used before mpv_initialize()\n       has been called. If called before mpv_initialize(), setting properties\n       not backed by options will result in MPV_ERROR_PROPERTY_UNAVAILABLE.\n       In some cases, properties and options still conflict. In these cases,\n       mpv_set_property() accesses the options before mpv_initialize(), and\n       the properties after mpv_initialize(). These conflicts will be removed\n       in mpv 0.23.0. See mpv_set_option() for further remarks.\n\n @param name The property name. See input.rst for a list of properties.\n @param format see enum mpv_format.\n @param[in] data Option value.\n @return error code"]
    set_property(
        ctx: *mut mpv_handle,
        name: *const ::std::os::raw::c_char,
        format: mpv_format,
        data: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
    #[doc = " Convenience function to set a property to a string value.\n\n This is like calling mpv_set_property() with MPV_FORMAT_STRING."]
    set_property_string(
        ctx: *mut mpv_handle,
        name: *const ::std::os::raw::c_char,
        data: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    #[doc = " Convenience function to delete a property.\n\n This is equivalent to running the command \"del [name]\".\n\n @param name The property name. See input.rst for a list of properties.\n @return error code"]
    del_property(
        ctx: *mut mpv_handle,
        name: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    #[doc = " Set a property asynchronously. You will receive the result of the operation\n as MPV_EVENT_SET_PROPERTY_REPLY event. The mpv_event.error field will contain\n the result status of the operation. Otherwise, this function is similar to\n mpv_set_property().\n\n Safe to be called from mpv render API threads.\n\n @param reply_userdata see section about asynchronous calls\n @param name The property name.\n @param format see enum mpv_format.\n @param[in] data Option value. The value will be copied by the function. It\n                 will never be modified by the client API.\n @return error code if sending the request failed"]
    set_property_async(
        ctx: *mut mpv_handle,
        reply_userdata: u64,
        name: *const ::std::os::raw::c_char,
        format: mpv_format,
        data: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
    #[doc = " Read the value of the given property.\n\n If the format doesn't match with the internal format of the property, access\n usually will fail with MPV_ERROR_PROPERTY_FORMAT. In some cases, the data\n is automatically converted and access succeeds. For example, MPV_FORMAT_INT64\n is always converted to MPV_FORMAT_DOUBLE, and access using MPV_FORMAT_STRING\n usually invokes a string formatter.\n\n @param name The property name.\n @param format see enum mpv_format.\n @param[out] data Pointer to the variable holding the option value. On\n                  success, the variable will be set to a copy of the option\n                  value. For formats that require dynamic memory allocation,\n                  you can free the value with mpv_free() (strings) or\n                  mpv_free_node_contents() (MPV_FORMAT_NODE).\n @return error code"]
    get_property(
        ctx: *mut mpv_handle,
        name: *const ::std::os::raw::c_char,
        format: mpv_format,
        data: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
    #[doc = " Return the value of the property with the given name as string. This is\n equivalent to mpv_get_property() with MPV_FORMAT_STRING.\n\n See MPV_FORMAT_STRING for character encoding issues.\n\n On error, NULL is returned. Use mpv_get_property() if you want fine-grained\n error reporting.\n\n @param name The property name.\n @return Property value, or NULL if the property can't be retrieved. Free\n         the string with mpv_free()."]
    get_property_string(
        ctx: *mut mpv_handle,
        name: *const ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_char;
    #[doc = " Return the property as \"OSD\" formatted string. This is the same as\n mpv_get_property_string, but using MPV_FORMAT_OSD_STRING.\n\n @return Property value, or NULL if the property can't be retrieved. Free\n         the string with mpv_free()."]
    get_property_osd_string(
        ctx: *mut mpv_handle,
        name: *const ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_char;
    #[doc = " Get a property asynchronously. You will receive the result of the operation\n as well as the property data with the MPV_EVENT_GET_PROPERTY_REPLY event.\n You should check the mpv_event.error field on the reply event.\n\n Safe to be called from mpv render API threads.\n\n @param reply_userdata see section about asynchronous calls\n @param name The property name.\n @param format see enum mpv_format.\n @return error code if sending the request failed"]
    get_property_async(
        ctx: *mut mpv_handle,
        reply_userdata: u64,
        name: *const ::std::os::raw::c_char,
        format: mpv_format,
    ) -> ::std::os::raw::c_int;
    #[doc = " Get a notification whenever the given property changes. You will receive\n updates as MPV_EVENT_PROPERTY_CHANGE. Note that this is not very precise:\n for some properties, it may not send updates even if the property changed.\n This depends on the property, and it's a valid feature request to ask for\n better update handling of a specific property. (For some properties, like\n ``clock``, which shows the wall clock, this mechanism doesn't make too\n much sense anyway.)\n\n Property changes are coalesced: the change events are returned only once the\n event queue becomes empty (e.g. mpv_wait_event() would block or return\n MPV_EVENT_NONE), and then only one event per changed property is returned.\n\n You always get an initial change notification. This is meant to initialize\n the user's state to the current value of the property.\n\n Normally, change events are sent only if the property value changes according\n to the requested format. mpv_event_property will contain the property value\n as data member.\n\n Warning: if a property is unavailable or retrieving it caused an error,\n          MPV_FORMAT_NONE will be set in mpv_event_property, even if the\n          format parameter was set to a different value. In this case, the\n          mpv_event_property.data field is invalid.\n\n If the property is observed with the format parameter set to MPV_FORMAT_NONE,\n you get low-level notifications whether the property _may_ have changed, and\n the data member in mpv_event_property will be unset. With this mode, you\n will have to determine yourself whether the property really changed. On the\n other hand, this mechanism can be faster and uses less resources.\n\n Observing a property that doesn't exist is allowed. (Although it may still\n cause some sporadic change events.)\n\n Keep in mind that you will get change notifications even if you change a\n property yourself. Try to avoid endless feedback loops, which could happen\n if you react to the change notifications triggered by your own change.\n\n Only the mpv_handle on which this was called will receive the property\n change events, or can unobserve them.\n\n Safe to be called from mpv render API threads.\n\n @param reply_userdata This will be used for the mpv_event.reply_userdata\n                       field for the received MPV_EVENT_PROPERTY_CHANGE\n                       events. (Also see section about asynchronous calls,\n                       although this function is somewhat different from\n                       actual asynchronous calls.)\n                       If you have no use for this, pass 0.\n                       Also see mpv_unobserve_property().\n @param name The property name.\n @param format see enum mpv_format. Can be MPV_FORMAT_NONE to omit values\n               from the change events.\n @return error code (usually fails only on OOM or unsupported format)"]
    observe_property(
        mpv: *mut mpv_handle,
        reply_userdata: u64,
        name: *const ::std::os::raw::c_char,
        format: mpv_format,
    ) -> ::std::os::raw::c_int;
    #[doc = " Undo mpv_observe_property(). This will remove all observed properties for\n which the given number was passed as reply_userdata to mpv_observe_property.\n\n Safe to be called from mpv render API threads.\n\n @param registered_reply_userdata ID that was passed to mpv_observe_property\n @return negative value is an error code, >=0 is number of removed properties\n         on success (includes the case when 0 were removed)"]
    unobserve_property(
        mpv: *mut mpv_handle,
        registered_reply_userdata: u64,
    ) -> ::std::os::raw::c_int;
    #[doc = " Return a string describing the event. For unknown events, NULL is returned.\n\n Note that all events actually returned by the API will also yield a non-NULL\n string with this function.\n\n @param event event ID, see see enum mpv_event_id\n @return A static string giving a short symbolic name of the event. It\n         consists of lower-case alphanumeric characters and can include \"-\"\n         characters. This string is suitable for use in e.g. scripting\n         interfaces.\n         The string is completely static, i.e. doesn't need to be deallocated,\n         and is valid forever."]
    event_name(event: mpv_event_id) -> *const ::std::os::raw::c_char;
    #[doc = " Convert the given src event to a mpv_node, and set *dst to the result. *dst\n is set to a MPV_FORMAT_NODE_MAP, with fields for corresponding mpv_event and\n mpv_event.data/mpv_event_* fields.\n\n The exact details are not completely documented out of laziness. A start\n is located in the \"Events\" section of the manpage.\n\n *dst may point to newly allocated memory, or pointers in mpv_event. You must\n copy the entire mpv_node if you want to reference it after mpv_event becomes\n invalid (such as making a new mpv_wait_event() call, or destroying the\n mpv_handle from which it was returned). Call mpv_free_node_contents() to free\n any memory allocations made by this API function.\n\n Safe to be called from mpv render API threads.\n\n @param dst Target. This is not read and fully overwritten. Must be released\n            with mpv_free_node_contents(). Do not write to pointers returned\n            by it. (On error, this may be left as an empty node.)\n @param src The source event. Not modified (it's not const due to the author's\n            prejudice of the C version of const).\n @return error code (MPV_ERROR_NOMEM only, if at all)"]
    event_to_node(dst: *mut mpv_node, src: *mut mpv_event) -> ::std::os::raw::c_int;
    #[doc = " Enable or disable the given event.\n\n Some events are enabled by default. Some events can't be disabled.\n\n (Informational note: currently, all events are enabled by default, except\n  MPV_EVENT_TICK.)\n\n Safe to be called from mpv render API threads.\n\n @param event See enum mpv_event_id.\n @param enable 1 to enable receiving this event, 0 to disable it.\n @return error code"]
    request_event(
        ctx: *mut mpv_handle,
        event: mpv_event_id,
        enable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
    #[doc = " Enable or disable receiving of log messages. These are the messages the\n command line player prints to the terminal. This call sets the minimum\n required log level for a message to be received with MPV_EVENT_LOG_MESSAGE.\n\n @param min_level Minimal log level as string. Valid log levels:\n                      no fatal error warn info v debug trace\n                  The value \"no\" disables all messages. This is the default.\n                  An exception is the value \"terminal-default\", which uses the\n                  log level as set by the \"--msg-level\" option. This works\n                  even if the terminal is disabled. (Since API version 1.19.)\n                  Also see mpv_log_level.\n @return error code"]
    request_log_messages(
        ctx: *mut mpv_handle,
        min_level: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    #[doc = " Wait for the next event, or until the timeout expires, or if another thread\n makes a call to mpv_wakeup(). Passing 0 as timeout will never wait, and\n is suitable for polling.\n\n The internal event queue has a limited size (per client handle). If you\n don't empty the event queue quickly enough with mpv_wait_event(), it will\n overflow and silently discard further events. If this happens, making\n asynchronous requests will fail as well (with MPV_ERROR_EVENT_QUEUE_FULL).\n\n Only one thread is allowed to call this on the same mpv_handle at a time.\n The API won't complain if more than one thread calls this, but it will cause\n race conditions in the client when accessing the shared mpv_event struct.\n Note that most other API functions are not restricted by this, and no API\n function internally calls mpv_wait_event(). Additionally, concurrent calls\n to different mpv_handles are always safe.\n\n As long as the timeout is 0, this is safe to be called from mpv render API\n threads.\n\n @param timeout Timeout in seconds, after which the function returns even if\n                no event was received. A MPV_EVENT_NONE is returned on\n                timeout. A value of 0 will disable waiting. Negative values\n                will wait with an infinite timeout.\n @return A struct containing the event ID and other data. The pointer (and\n         fields in the struct) stay valid until the next mpv_wait_event()\n         call, or until the mpv_handle is destroyed. You must not write to\n         the struct, and all memory referenced by it will be automatically\n         released by the API on the next mpv_wait_event() call, or when the\n         context is destroyed. The return value is never NULL."]
    wait_event(ctx: *mut mpv_handle, timeout: f64) -> *mut mpv_event;
    #[doc = " Interrupt the current mpv_wait_event() call. This will wake up the thread\n currently waiting in mpv_wait_event(). If no thread is waiting, the next\n mpv_wait_event() call will return immediately (this is to avoid lost\n wakeups).\n\n mpv_wait_event() will receive a MPV_EVENT_NONE if it's woken up due to\n this call. But note that this dummy event might be skipped if there are\n already other events queued. All what counts is that the waiting thread\n is woken up at all.\n\n Safe to be called from mpv render API threads."]
    wakeup(ctx: *mut mpv_handle);
    #[doc = " Set a custom function that should be called when there are new events. Use\n this if blocking in mpv_wait_event() to wait for new events is not feasible.\n\n Keep in mind that the callback will be called from foreign threads. You\n must not make any assumptions of the environment, and you must return as\n soon as possible (i.e. no long blocking waits). Exiting the callback through\n any other means than a normal return is forbidden (no throwing exceptions,\n no longjmp() calls). You must not change any local thread state (such as\n the C floating point environment).\n\n You are not allowed to call any client API functions inside of the callback.\n In particular, you should not do any processing in the callback, but wake up\n another thread that does all the work. The callback is meant strictly for\n notification only, and is called from arbitrary core parts of the player,\n that make no considerations for reentrant API use or allowing the callee to\n spend a lot of time doing other things. Keep in mind that it's also possible\n that the callback is called from a thread while a mpv API function is called\n (i.e. it can be reentrant).\n\n In general, the client API expects you to call mpv_wait_event() to receive\n notifications, and the wakeup callback is merely a helper utility to make\n this easier in certain situations. Note that it's possible that there's\n only one wakeup callback invocation for multiple events. You should call\n mpv_wait_event() with no timeout until MPV_EVENT_NONE is reached, at which\n point the event queue is empty.\n\n If you actually want to do processing in a callback, spawn a thread that\n does nothing but call mpv_wait_event() in a loop and dispatches the result\n to a callback.\n\n Only one wakeup callback can be set.\n\n @param cb function that should be called if a wakeup is required\n @param d arbitrary userdata passed to cb"]
    set_wakeup_callback(
        ctx: *mut mpv_handle,
        cb: ::std::option::Option<unsafe extern "C" fn(d: *mut ::std::os::raw::c_void)>,
        d: *mut ::std::os::raw::c_void,
    );
    #[doc = " Block until all asynchronous requests are done. This affects functions like\n mpv_command_async(), which return immediately and return their result as\n events.\n\n This is a helper, and somewhat equivalent to calling mpv_wait_event() in a\n loop until all known asynchronous requests have sent their reply as event,\n except that the event queue is not emptied.\n\n In case you called mpv_suspend() before, this will also forcibly reset the\n suspend counter of the given handle."]
    wait_async_requests(ctx: *mut mpv_handle);
    #[doc = " A hook is like a synchronous event that blocks the player. You register\n a hook handler with this function. You will get an event, which you need\n to handle, and once things are ready, you can let the player continue with\n mpv_hook_continue().\n\n Currently, hooks can't be removed explicitly. But they will be implicitly\n removed if the mpv_handle it was registered with is destroyed. This also\n continues the hook if it was being handled by the destroyed mpv_handle (but\n this should be avoided, as it might mess up order of hook execution).\n\n Hook handlers are ordered globally by priority and order of registration.\n Handlers for the same hook with same priority are invoked in order of\n registration (the handler registered first is run first). Handlers with\n lower priority are run first (which seems backward).\n\n See the \"Hooks\" section in the manpage to see which hooks are currently\n defined.\n\n Some hooks might be reentrant (so you get multiple MPV_EVENT_HOOK for the\n same hook). If this can happen for a specific hook type, it will be\n explicitly documented in the manpage.\n\n Only the mpv_handle on which this was called will receive the hook events,\n or can \"continue\" them.\n\n @param reply_userdata This will be used for the mpv_event.reply_userdata\n                       field for the received MPV_EVENT_HOOK events.\n                       If you have no use for this, pass 0.\n @param name The hook name. This should be one of the documented names. But\n             if the name is unknown, the hook event will simply be never\n             raised.\n @param priority See remarks above. Use 0 as a neutral default.\n @return error code (usually fails only on OOM)"]
    hook_add(
        ctx: *mut mpv_handle,
        reply_userdata: u64,
        name: *const ::std::os::raw::c_char,
        priority: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
    #[doc = " Respond to a MPV_EVENT_HOOK event. You must call this after you have handled\n the event. There is no way to \"cancel\" or \"stop\" the hook.\n\n Calling this will will typically unblock the player for whatever the hook\n is responsible for (e.g. for the \"on_load\" hook it lets it continue\n playback).\n\n It is explicitly undefined behavior to call this more than once for each\n MPV_EVENT_HOOK, to pass an incorrect ID, or to call this on a mpv_handle\n different from the one that registered the handler and received the event.\n\n @param id This must be the value of the mpv_event_hook.id field for the\n           corresponding MPV_EVENT_HOOK.\n @return error code"]
    hook_continue(ctx: *mut mpv_handle, id: u64) -> ::std::os::raw::c_int;
    #[doc = " Return a UNIX file descriptor referring to the read end of a pipe. This\n pipe can be used to wake up a poll() based processing loop. The purpose of\n this function is very similar to mpv_set_wakeup_callback(), and provides\n a primitive mechanism to handle coordinating a foreign event loop and the\n libmpv event loop. The pipe is non-blocking. It's closed when the mpv_handle\n is destroyed. This function always returns the same value (on success).\n\n This is in fact implemented using the same underlying code as for\n mpv_set_wakeup_callback() (though they don't conflict), and it is as if each\n callback invocation writes a single 0 byte to the pipe. When the pipe\n becomes readable, the code calling poll() (or select()) on the pipe should\n read all contents of the pipe and then call mpv_wait_event(c, 0) until\n no new events are returned. The pipe contents do not matter and can just\n be discarded. There is not necessarily one byte per readable event in the\n pipe. For example, the pipes are non-blocking, and mpv won't block if the\n pipe is full. Pipes are normally limited to 4096 bytes, so if there are\n more than 4096 events, the number of readable bytes can not equal the number\n of events queued. Also, it's possible that mpv does not write to the pipe\n once it's guaranteed that the client was already signaled. See the example\n below how to do it correctly.\n\n Example:\n\n  int pipefd = mpv_get_wakeup_pipe(mpv);\n  if (pipefd < 0)\n      error();\n  while (1) {\n      struct pollfd pfds[1] = {\n          { .fd = pipefd, .events = POLLIN },\n      };\n      // Wait until there are possibly new mpv events.\n      poll(pfds, 1, -1);\n      if (pfds[0].revents & POLLIN) {\n          // Empty the pipe. Doing this before calling mpv_wait_event()\n          // ensures that no wakeups are missed. It's not so important to\n          // make sure the pipe is really empty (it will just cause some\n          // additional wakeups in unlikely corner cases).\n          char unused[256];\n          read(pipefd, unused, sizeof(unused));\n          while (1) {\n              mpv_event *ev = mpv_wait_event(mpv, 0);\n              // If MPV_EVENT_NONE is received, the event queue is empty.\n              if (ev->event_id == MPV_EVENT_NONE)\n                  break;\n              // Process the event.\n              ...\n          }\n      }\n  }\n\n @deprecated this function will be removed in the future. If you need this\n             functionality, use mpv_set_wakeup_callback(), create a pipe\n             manually, and call write() on your pipe in the callback.\n\n @return A UNIX FD of the read end of the wakeup pipe, or -1 on error.\n         On MS Windows/MinGW, this will always return -1."]
    get_wakeup_pipe(ctx: *mut mpv_handle) -> ::std::os::raw::c_int;
}

pub use mpv_stubs::setup_mpv_stubs;

mod mpv_stubs {
    use std::ffi::c_void;
    use crate::mpv_node;
    #[cfg(feature = "dyn-sym")]
    use crate::mpv_pfns::{pfn_mpv_free, pfn_mpv_free_node_contents};

    #[allow(unused_variables)]
    /// Provide stub functions for [`free()`](crate::free) and [`free_node_contents()`](crate::free_node_contents) for use in a test environment disconnected from mpv.
    pub fn setup_mpv_stubs(free: extern "C" fn(data: *mut c_void), free_node_contents: extern "C" fn(node: *mut mpv_node)) {
        #[cfg(feature = "dyn-sym")]
        unsafe {
            pfn_mpv_free = Some(free);
            pfn_mpv_free_node_contents = Some(free_node_contents);
        }
    }
}