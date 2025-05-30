use std::ffi::CStr;
use std::os::raw::c_void;
use libmpv_client_sys::{mpv_event, mpv_event_client_message, mpv_event_command, mpv_event_end_file, mpv_event_hook, mpv_event_id, mpv_event_id_MPV_EVENT_AUDIO_RECONFIG, mpv_event_id_MPV_EVENT_CLIENT_MESSAGE, mpv_event_id_MPV_EVENT_COMMAND_REPLY, mpv_event_id_MPV_EVENT_END_FILE, mpv_event_id_MPV_EVENT_FILE_LOADED, mpv_event_id_MPV_EVENT_GET_PROPERTY_REPLY, mpv_event_id_MPV_EVENT_HOOK, mpv_event_id_MPV_EVENT_IDLE, mpv_event_id_MPV_EVENT_LOG_MESSAGE, mpv_event_id_MPV_EVENT_NONE, mpv_event_id_MPV_EVENT_PLAYBACK_RESTART, mpv_event_id_MPV_EVENT_PROPERTY_CHANGE, mpv_event_id_MPV_EVENT_QUEUE_OVERFLOW, mpv_event_id_MPV_EVENT_SEEK, mpv_event_id_MPV_EVENT_SET_PROPERTY_REPLY, mpv_event_id_MPV_EVENT_SHUTDOWN, mpv_event_id_MPV_EVENT_START_FILE, mpv_event_id_MPV_EVENT_TICK, mpv_event_id_MPV_EVENT_VIDEO_RECONFIG, mpv_event_log_message, mpv_event_property, mpv_event_start_file};
use crate::*;
use crate::error::{error_to_result, RustError};

pub struct EventId(pub(crate) mpv_event_id);

impl EventId {
    pub const NONE: EventId = EventId(mpv_event_id_MPV_EVENT_NONE);
    pub const SHUTDOWN: EventId = EventId(mpv_event_id_MPV_EVENT_SHUTDOWN);
    pub const LOG_MESSAGE: EventId = EventId(mpv_event_id_MPV_EVENT_LOG_MESSAGE);
    pub const GET_PROPERTY_REPLY: EventId = EventId(mpv_event_id_MPV_EVENT_GET_PROPERTY_REPLY);
    pub const SET_PROPERTY_REPLY: EventId = EventId(mpv_event_id_MPV_EVENT_SET_PROPERTY_REPLY);
    pub const COMMAND_REPLY: EventId = EventId(mpv_event_id_MPV_EVENT_COMMAND_REPLY);
    pub const START_FILE: EventId = EventId(mpv_event_id_MPV_EVENT_START_FILE);
    pub const END_FILE: EventId = EventId(mpv_event_id_MPV_EVENT_END_FILE);
    pub const FILE_LOADED: EventId = EventId(mpv_event_id_MPV_EVENT_FILE_LOADED);
    pub const IDLE: EventId = EventId(mpv_event_id_MPV_EVENT_IDLE);
    pub const TICK: EventId = EventId(mpv_event_id_MPV_EVENT_TICK);
    pub const CLIENT_MESSAGE: EventId = EventId(mpv_event_id_MPV_EVENT_CLIENT_MESSAGE);
    pub const VIDEO_RECONFIG: EventId = EventId(mpv_event_id_MPV_EVENT_VIDEO_RECONFIG);
    pub const AUDIO_RECONFIG: EventId = EventId(mpv_event_id_MPV_EVENT_AUDIO_RECONFIG);
    pub const SEEK: EventId = EventId(mpv_event_id_MPV_EVENT_SEEK);
    pub const PLAYBACK_RESTART: EventId = EventId(mpv_event_id_MPV_EVENT_PLAYBACK_RESTART);
    pub const PROPERTY_CHANGE: EventId = EventId(mpv_event_id_MPV_EVENT_PROPERTY_CHANGE);
    pub const QUEUE_OVERFLOW: EventId = EventId(mpv_event_id_MPV_EVENT_QUEUE_OVERFLOW);
    pub const HOOK: EventId = EventId(mpv_event_id_MPV_EVENT_HOOK);
}

#[derive(Debug)]
pub enum Event {
    /// Nothing happened. Happens on timeouts or sporadic wakeups.
    None,
    /// Happens when the player quits. The player enters a state where it tries to disconnect all clients.
    ///
    /// Most requests to the player will fail, and the client should react to this and quit with mpv_destroy() as soon as possible.
    Shutdown,
    LogMessage(LogMessage),
    /// Reply to a mpv_get_property_async() request.
    GetPropertyReply { error: Result<i32>, userdata: u64, property: Property },
    /// Reply to a mpv_set_property_async() request.
    SetPropertyReply { error: Result<i32>, userdata: u64 },
    /// Reply to a mpv_command_async() or mpv_command_node_async() request.
    CommandReply { error: Result<i32>, userdata: u64, command: Command},
    /// Notification before playback start of a file (before the file is loaded).
    StartFile(StartFile),
    /// Notification after playback end (after the file was unloaded).
    EndFile(EndFile),
    /// Notification when the file has been loaded (headers were read etc.), and decoding starts.
    FileLoaded,
    /// Idle mode was entered.
    ///
    /// In this mode, no file is played, and the playback core waits for new commands.
    ///
    /// (The command line player normally quits instead of entering idle mode, unless --idle was specified. If mpv was started with mpv_create(), idle mode is enabled by default.)
    #[deprecated = "This is equivalent to using mpv_observe_property() on the \"idle-active\" property. The event is redundant, and might be removed in the far future. As a further warning, this event is not necessarily sent at the right point anymore (at the start of the program), while the property behaves correctly."]
    Idle,
    /// Sent every time after a video frame is displayed.
    ///
    /// Note that currently this will be sent in lower frequency if there is no video, or playback is paused - but that will be removed in the future, and it will be restricted to video frames only.
    #[deprecated = "Use mpv_observe_property() with relevant properties instead (such as \"playback-time\")."]
    Tick,
    /// Triggered by the script-message input command.
    ///
    /// The command uses the first argument of the command as client name (see mpv_client_name()) to dispatch the message, and passes along all arguments starting from the second argument as strings.
    ClientMessage(ClientMessage),
    /// Happens after video changed in some way. This can happen on resolution changes, pixel format changes, or video filter changes. The event is sent after the video filters and the VO are reconfigured. Applications embedding a mpv window should listen to this event in order to resize the window if needed.
    ///
    /// Note that this event can happen sporadically, and you should check yourself whether the video parameters really changed before doing something expensive.
    VideoReconfig,
    /// Similar to MPV_EVENT_VIDEO_RECONFIG.
    ///
    /// This is relatively uninteresting, because there is no such thing as audio output embedding.
    AudioReconfig,
    /// Happens when a seek was initiated. Playback stops.
    ///
    /// Usually it will resume with MPV_EVENT_PLAYBACK_RESTART as soon as the seek is finished.
    Seek,
    /// There was a discontinuity of some sort (like a seek), and playback was reinitialized.
    ///
    /// Usually happens on start of playback and after seeking. The main purpose is allowing the client to detect when a seek request is finished.
    PlaybackRestart,
    /// Event sent due to mpv_observe_property().
    PropertyChange { userdata: u64, property: Property },
    /// Happens if the internal per-mpv_handle ringbuffer overflows, and at least 1 event had to be dropped.
    ///
    /// This can happen if the client doesn't read the event queue quickly enough with mpv_wait_event(), or if the client makes a very large number of asynchronous calls at once.
    ///
    /// Event delivery will continue normally once this event was returned (this forces the client to empty the queue completely).
    QueueOverflow,
    /// Triggered if a hook handler was registered with mpv_hook_add(), and the hook is invoked.
    ///
    /// If you receive this, you *must* handle it, and continue the hook with mpv_hook_continue().
    Hook { userdata: u64, hook: Hook },
}

impl Event {
    pub(crate) fn from_ptr(ptr: *const mpv_event) -> Result<Event> {
        if ptr.is_null() {
            return Err(Error::Rust(RustError::Pointer))
        }

        let event = unsafe { *ptr };

        match event.event_id {
            libmpv_client_sys::mpv_event_id_MPV_EVENT_NONE => Ok(Event::None),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_SHUTDOWN => Ok(Event::Shutdown),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_LOG_MESSAGE => {
                Ok(Event::LogMessage(unsafe { LogMessage::from_ptr(event.data)? }))
            },
            libmpv_client_sys::mpv_event_id_MPV_EVENT_GET_PROPERTY_REPLY => {
                Ok(Event::GetPropertyReply {
                    error: error_to_result(event.error),
                    userdata: event.reply_userdata,
                    property: unsafe { Property::from_ptr(event.data)? },
                })
            },
            libmpv_client_sys::mpv_event_id_MPV_EVENT_SET_PROPERTY_REPLY => {
                Ok(Event::SetPropertyReply {
                    error: error_to_result(event.error),
                    userdata: event.reply_userdata,
                })
            },
            libmpv_client_sys::mpv_event_id_MPV_EVENT_COMMAND_REPLY => {
                Ok(Event::CommandReply {
                    error: error_to_result(event.error),
                    userdata: event.reply_userdata,
                    command: unsafe { Command::from_ptr(event.data)? },
                })
            },
            libmpv_client_sys::mpv_event_id_MPV_EVENT_START_FILE => {
                Ok(Event::StartFile(unsafe { StartFile::from_ptr(event.data)? }))
            },
            libmpv_client_sys::mpv_event_id_MPV_EVENT_END_FILE => {
                Ok(Event::EndFile(unsafe { EndFile::from_ptr(event.data)? }))
            },
            libmpv_client_sys::mpv_event_id_MPV_EVENT_FILE_LOADED => Ok(Event::FileLoaded),
            #[allow(deprecated)]
            libmpv_client_sys::mpv_event_id_MPV_EVENT_IDLE => Ok(Event::Idle),
            #[allow(deprecated)]
            libmpv_client_sys::mpv_event_id_MPV_EVENT_TICK => Ok(Event::Tick),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_CLIENT_MESSAGE => {
                Ok(Event::ClientMessage(unsafe { ClientMessage::from_ptr(event.data)? }))
            },
            libmpv_client_sys::mpv_event_id_MPV_EVENT_VIDEO_RECONFIG => Ok(Event::VideoReconfig),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_AUDIO_RECONFIG => Ok(Event::AudioReconfig),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_SEEK => Ok(Event::Seek),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_PLAYBACK_RESTART => Ok(Event::PlaybackRestart),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_PROPERTY_CHANGE => {
                Ok(Event::PropertyChange {
                    userdata: event.reply_userdata,
                    property: unsafe { Property::from_ptr(event.data)? },
                })
            },
            libmpv_client_sys::mpv_event_id_MPV_EVENT_QUEUE_OVERFLOW => Ok(Event::QueueOverflow),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_HOOK => {
                Ok(Event::Hook {
                    userdata: event.reply_userdata,
                    hook: unsafe { Hook::from_ptr(event.data)? },
                })
            },
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub struct Property {
    /// Name of the property.
    pub name: String,
    /// Data field of the property.
    ///
    /// This is always the same format as the requested format, except when the property could not be retrieved (unavailable, or an error happened), in which case the format is `Format::None`.
    pub data: PropertyValue,
}

impl Property {
    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self> {
        if ptr.is_null() {
            return Err(Error::Rust(RustError::Pointer));
        }

        let event_prop = unsafe { *(ptr as *const mpv_event_property) };

        if event_prop.name.is_null() {
            return Err(Error::Rust(RustError::Pointer));
        }

        let name = unsafe { CStr::from_ptr(event_prop.name).to_str()?.to_string() };

        let data = unsafe { PropertyValue::from_mpv(event_prop.format, event_prop.data)? };

        Ok(Self { name, data })
    }
}

#[derive(Debug)]
pub enum LogLevel {
    /// critical/aborting errors
    Fatal,
    /// simple errors
    Error,
    /// possible problems
    Warn,
    /// informational message
    Info,
    /// noisy informational message
    Verbose,
    /// very noisy technical information
    Debug,
    /// extremely noisy
    Trace,
}

#[derive(Debug)]
pub struct LogMessage {
    pub level: LogLevel,
    /// The module prefix, identifies the sender of the message.
    ///
    /// As a special case, if the message buffer overflows, this will be set to the string "overflow" (which doesn't appear as prefix otherwise), and the text field will contain an informative message.
    pub prefix: String,
    /// The log message. It consists of 1 line of text, and is terminated with a newline character.
    pub text: String,
}

impl LogMessage {
    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self> {
        if ptr.is_null() {
            return Err(Error::Rust(RustError::Pointer))
        }

        let event_log_message = unsafe { *(ptr as *const mpv_event_log_message) };

        let level = match event_log_message.log_level {
            libmpv_client_sys::mpv_log_level_MPV_LOG_LEVEL_FATAL => LogLevel::Fatal,
            libmpv_client_sys::mpv_log_level_MPV_LOG_LEVEL_ERROR => LogLevel::Error,
            libmpv_client_sys::mpv_log_level_MPV_LOG_LEVEL_WARN => LogLevel::Warn,
            libmpv_client_sys::mpv_log_level_MPV_LOG_LEVEL_INFO => LogLevel::Info,
            libmpv_client_sys::mpv_log_level_MPV_LOG_LEVEL_V => LogLevel::Verbose,
            libmpv_client_sys::mpv_log_level_MPV_LOG_LEVEL_DEBUG => LogLevel::Debug,
            libmpv_client_sys::mpv_log_level_MPV_LOG_LEVEL_TRACE => LogLevel::Trace,
            _ => unimplemented!()
        };

        if event_log_message.prefix.is_null() {
            return Err(Error::Rust(RustError::Pointer))
        }

        let prefix = unsafe { CStr::from_ptr(event_log_message.prefix) }.to_str()?.to_string();

        if event_log_message.text.is_null() {
            return Err(Error::Rust(RustError::Pointer))
        }

        let text = unsafe { CStr::from_ptr(event_log_message.text) }.to_str()?.to_string();

        Ok(Self { level, prefix, text })
    }
}

/// Arbitrary arguments chosen by the sender of the message. What these arguments mean is up to the sender and receiver.
#[derive(Debug)]
pub struct ClientMessage(pub Vec<String>);

impl ClientMessage {
    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self> {
        if ptr.is_null() {
            return Err(Error::Rust(RustError::Pointer))
        }
        
        let event_client_message = unsafe { *(ptr as *const mpv_event_client_message) };

        let mut args = Vec::with_capacity(event_client_message.num_args as usize);

        if event_client_message.args.is_null() {
            return Err(Error::Rust(RustError::Pointer))
        }

        let event_args = unsafe { std::slice::from_raw_parts(event_client_message.args, event_client_message.num_args as usize) };
        for event_arg in event_args {
            if event_arg.is_null() {
                return Err(Error::Rust(RustError::Pointer))
            }

            args.push(unsafe { CStr::from_ptr(*event_arg).to_str()?.to_string() });
        }

        Ok(Self(args))
    }
}

#[derive(Debug)]
pub struct StartFile {
    /// Playlist entry ID of the file being loaded now.
    pub playlist_entry_id: i64,
}

impl StartFile {
    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self> {
        if ptr.is_null() {
            return Err(Error::Rust(RustError::Pointer))
        }

        let event_start_file = unsafe { *(ptr as *const mpv_event_start_file) };

        Ok(Self {
            playlist_entry_id: event_start_file.playlist_entry_id,
        })
    }
}

#[derive(Debug)]
pub enum EndFileReason {
    /// The end of file was reached.
    ///
    /// Sometimes this may also happen on incomplete or corrupted files, or if the network connection was interrupted when playing a remote file.
    /// It also happens if the playback range was restricted with --end or --frames or similar.
    Eof,
    /// Playback was stopped by an external action (e.g. playlist controls).
    Stop,
    /// Playback was stopped by the quit command or player shutdown.
    Quit,
    /// Some kind of error happened that lead to playback abort.
    ///
    /// Does not necessarily happen on incomplete or broken files (in these cases, both MPV_END_FILE_REASON_ERROR or MPV_END_FILE_REASON_EOF are possible).
    Error(Error),
    /// The file was a playlist or similar.
    ///
    /// When the playlist is read, its entries will be appended to the playlist after the entry of the current file, the entry of the current file is removed, and a MPV_EVENT_END_FILE event is sent with reason set to MPV_END_FILE_REASON_REDIRECT.
    /// Then playback continues with the playlist contents.
    Redirect,
}

#[derive(Debug)]
pub struct EndFile {
    pub reason: EndFileReason,
    /// Playlist entry ID of the file that was being played or attempted to be played.
    ///
    /// This has the same value as the `playlist_entry_id` field in the corresponding `StartFile` event.
    pub playlist_entry_id: i64,
    /// If loading ended, because the playlist entry to be played was for example a playlist, and the current playlist entry is replaced with a number of other entries.
    ///
    /// This may happen at least with `EndFileReason::Redirect` (other event types may use this for similar but different purposes in the future).
    /// In this case, `playlist_insert_id` will be set to the playlist entry ID of the first inserted entry, and `playlist_insert_num_entries` to the total number of inserted playlist entries.
    /// Note this in this specific case, the ID of the last inserted entry is `playlist_insert_id`+num-1.
    /// Beware that depending on circumstances, you may observe the new playlist entries before seeing the event (e.g. reading the "playlist" property or getting a property change notification before receiving the event).
    pub playlist_insert_id: i64,
    /// See `playlist_insert_id`. Only non-0 if `playlist_insert_id` is valid. Never negative.
    pub playlist_insert_num_entries: i32,
}

impl EndFile {
    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self> {
        if ptr.is_null() {
            return Err(Error::Rust(RustError::Pointer))
        }

        let event_end_file = unsafe { *(ptr as *const mpv_event_end_file) };

        let reason = match event_end_file.reason {
            libmpv_client_sys::mpv_end_file_reason_MPV_END_FILE_REASON_EOF => EndFileReason::Eof,
            libmpv_client_sys::mpv_end_file_reason_MPV_END_FILE_REASON_STOP => EndFileReason::Stop,
            libmpv_client_sys::mpv_end_file_reason_MPV_END_FILE_REASON_QUIT => EndFileReason::Quit,
            libmpv_client_sys::mpv_end_file_reason_MPV_END_FILE_REASON_ERROR => EndFileReason::Error(Error::from(event_end_file.error)),
            libmpv_client_sys::mpv_end_file_reason_MPV_END_FILE_REASON_REDIRECT => EndFileReason::Redirect,
            _ => unimplemented!(),
        };

        Ok(Self {
            reason,
            playlist_entry_id: event_end_file.playlist_entry_id,
            playlist_insert_id: event_end_file.playlist_insert_id,
            playlist_insert_num_entries: event_end_file.playlist_insert_num_entries as i32,
        })
    }
}

#[derive(Debug)]
pub struct Hook {
    /// The hook name as passed to mpv_hook_add().
    pub name: String,
    /// Internal ID that must be passed to mpv_hook_continue().
    pub id: u64,
}

impl Hook {
    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self> {
        if ptr.is_null() {
            return Err(Error::Rust(RustError::Pointer))
        }

        let event_hook = unsafe { *(ptr as *const mpv_event_hook) };

        if event_hook.name.is_null() {
            return Err(Error::Rust(RustError::Pointer))
        }

        let name = unsafe { CStr::from_ptr(event_hook.name) }.to_str()?.to_string();

        let id = event_hook.id;

        Ok(Self { name, id })
    }
}

#[derive(Debug)]
pub struct Command {
    /// Result data of the command.
    ///
    /// Note that success/failure is signaled separately via mpv_event.error. This field is only for result data in case of success.
    ///
    /// Most commands leave it at MPV_FORMAT_NONE.
    pub result: Node,
}

impl Command {
    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self> {
        if ptr.is_null() {
            return Err(Error::Rust(RustError::Pointer))
        }

        let event_command = unsafe { *(ptr as *const mpv_event_command) };

        let result = unsafe { Node::from_node_ptr(&event_command.result)? };

        Ok(Self { result })
    }
}