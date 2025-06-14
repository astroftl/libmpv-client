//! The various [`Event`]s and their payloads which may be sent by mpv.

use std::ffi::{c_void, CStr};
use libmpv_client_sys::{mpv_event, mpv_event_client_message, mpv_event_command, mpv_event_end_file, mpv_event_hook, mpv_event_id, mpv_event_id_MPV_EVENT_AUDIO_RECONFIG, mpv_event_id_MPV_EVENT_CLIENT_MESSAGE, mpv_event_id_MPV_EVENT_COMMAND_REPLY, mpv_event_id_MPV_EVENT_END_FILE, mpv_event_id_MPV_EVENT_FILE_LOADED, mpv_event_id_MPV_EVENT_GET_PROPERTY_REPLY, mpv_event_id_MPV_EVENT_HOOK, mpv_event_id_MPV_EVENT_IDLE, mpv_event_id_MPV_EVENT_LOG_MESSAGE, mpv_event_id_MPV_EVENT_PLAYBACK_RESTART, mpv_event_id_MPV_EVENT_PROPERTY_CHANGE, mpv_event_id_MPV_EVENT_QUEUE_OVERFLOW, mpv_event_id_MPV_EVENT_SEEK, mpv_event_id_MPV_EVENT_SET_PROPERTY_REPLY, mpv_event_id_MPV_EVENT_SHUTDOWN, mpv_event_id_MPV_EVENT_START_FILE, mpv_event_id_MPV_EVENT_TICK, mpv_event_id_MPV_EVENT_VIDEO_RECONFIG, mpv_event_log_message, mpv_event_property, mpv_event_start_file, mpv_format};
use crate::*;
use crate::error::error_to_result_code;
use crate::types::traits::MpvRecvInternal;

/// [`Event`] IDs for use with [`Handle::request_event()`].
pub struct EventId(pub(crate) mpv_event_id);

impl EventId {
    /// Requests [`Event::Shutdown`].
    pub const SHUTDOWN: EventId = EventId(mpv_event_id_MPV_EVENT_SHUTDOWN);
    /// Requests [`Event::LogMessage`].
    pub const LOG_MESSAGE: EventId = EventId(mpv_event_id_MPV_EVENT_LOG_MESSAGE);
    /// Requests [`Event::GetPropertyReply`].
    pub const GET_PROPERTY_REPLY: EventId = EventId(mpv_event_id_MPV_EVENT_GET_PROPERTY_REPLY);
    /// Requests [`Event::SetPropertyReply`].
    pub const SET_PROPERTY_REPLY: EventId = EventId(mpv_event_id_MPV_EVENT_SET_PROPERTY_REPLY);
    /// Requests [`Event::CommandReply`].
    pub const COMMAND_REPLY: EventId = EventId(mpv_event_id_MPV_EVENT_COMMAND_REPLY);
    /// Requests [`Event::StartFile`].
    pub const START_FILE: EventId = EventId(mpv_event_id_MPV_EVENT_START_FILE);
    /// Requests [`Event::EndFile`].
    pub const END_FILE: EventId = EventId(mpv_event_id_MPV_EVENT_END_FILE);
    /// Requests [`Event::FileLoaded`].
    pub const FILE_LOADED: EventId = EventId(mpv_event_id_MPV_EVENT_FILE_LOADED);
    /// Requests [`Event::Idle`].
    pub const IDLE: EventId = EventId(mpv_event_id_MPV_EVENT_IDLE);
    /// Requests [`Event::Tick`].
    pub const TICK: EventId = EventId(mpv_event_id_MPV_EVENT_TICK);
    /// Requests [`Event::ClientMessage`].
    pub const CLIENT_MESSAGE: EventId = EventId(mpv_event_id_MPV_EVENT_CLIENT_MESSAGE);
    /// Requests [`Event::VideoReconfig`].
    pub const VIDEO_RECONFIG: EventId = EventId(mpv_event_id_MPV_EVENT_VIDEO_RECONFIG);
    /// Requests [`Event::AudioReconfig`].
    pub const AUDIO_RECONFIG: EventId = EventId(mpv_event_id_MPV_EVENT_AUDIO_RECONFIG);
    /// Requests [`Event::Seek`].
    pub const SEEK: EventId = EventId(mpv_event_id_MPV_EVENT_SEEK);
    /// Requests [`Event::PlaybackRestart`].
    pub const PLAYBACK_RESTART: EventId = EventId(mpv_event_id_MPV_EVENT_PLAYBACK_RESTART);
    /// Requests [`Event::PropertyChange`].
    pub const PROPERTY_CHANGE: EventId = EventId(mpv_event_id_MPV_EVENT_PROPERTY_CHANGE);
    /// Requests [`Event::QueueOverflow`].
    pub const QUEUE_OVERFLOW: EventId = EventId(mpv_event_id_MPV_EVENT_QUEUE_OVERFLOW);
    /// Requests [`Event::Hook`].
    pub const HOOK: EventId = EventId(mpv_event_id_MPV_EVENT_HOOK);
}

/// The possible log levels that mpv can apply to log messages.
#[derive(Debug)]
pub enum LogLevel {
    /// disable logging
    None,
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

const LOG_LEVEL_NONE: &CStr = c"no";
const LOG_LEVEL_FATAL: &CStr = c"fatal";
const LOG_LEVEL_ERROR: &CStr = c"error";
const LOG_LEVEL_WARN: &CStr = c"warn";
const LOG_LEVEL_INFO: &CStr = c"info";
const LOG_LEVEL_VERBOSE: &CStr = c"v";
const LOG_LEVEL_DEBUG: &CStr = c"debug";
const LOG_LEVEL_TRACE: &CStr = c"trace";

impl LogLevel {
    pub(crate) fn to_cstr(&self) -> &CStr {
        match self {
            LogLevel::None => LOG_LEVEL_NONE,
            LogLevel::Fatal => LOG_LEVEL_FATAL,
            LogLevel::Error => LOG_LEVEL_ERROR,
            LogLevel::Warn => LOG_LEVEL_WARN,
            LogLevel::Info => LOG_LEVEL_INFO,
            LogLevel::Verbose => LOG_LEVEL_VERBOSE,
            LogLevel::Debug => LOG_LEVEL_DEBUG,
            LogLevel::Trace => LOG_LEVEL_TRACE,
        }
    }
}

/// Possible reasons for an [`Event::EndFile`].
#[derive(Debug)]
pub enum EndFileReason {
    /// The end of file was reached.
    ///
    /// Sometimes this may also happen on incomplete or corrupted files, or if the network connection was interrupted when playing a remote file.
    /// It also happens if the playback range was restricted with `--end` or `--frames` or similar.
    Eof,
    /// Playback was stopped by an external action (e.g. playlist controls).
    Stop,
    /// Playback was stopped by the quit command or player shutdown.
    Quit,
    /// Some kind of error happened that lead to playback abort.
    ///
    /// Does not necessarily happen on incomplete or broken files (in these cases, both [`EndFileReason::Error`] or [`EndFileReason::Eof`] are possible).
    Error(Error),
    /// The file was a playlist or similar.
    ///
    /// When the playlist is read, its entries will be appended to the playlist after the entry of the current file, the entry of the current file is removed, and an [`Event::EndFile`] is sent with [`EndFile.reason`](field@EndFile::reason) set to [`EndFileReason::Redirect`].
    /// Then playback continues with the playlist contents.
    Redirect,
}

/// Events that may be received from [`Handle::wait_event()`].
///
/// Some are just informational, while some contain additional data and some are responses to mpv commands.
#[derive(Debug)]
pub enum Event {
    /// Nothing happened. Happens on timeouts or sporadic wakeups.
    None,
    /// Happens when the player quits. The player enters a state where it tries to disconnect all clients.
    ///
    /// Most requests to the player will fail, and the client should react to this accordingly;
    /// a [`Handle`] should return execution from whatever context it was passed its [`mpv_handle`],
    /// while a [`Client`] should call [`Client::destroy`] and quit as soon as possible.
    Shutdown,
    /// Happens when mpv receives a log message that matches the level filter set up with [`Handle::request_log_messages()`].
    LogMessage(LogMessage),
    /// Reply to a `mpv_get_property_async()` request.
    GetPropertyReply(GetPropertyReply),
    /// Reply to a `mpv_set_property_async()` request.
    SetPropertyReply(SetPropertyReply),
    /// Reply to a `mpv_command_async()` or `mpv_command_node_async()` request.
    CommandReply(CommandReply),
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
    /// (The command line player normally quits instead of entering idle mode, unless `--idle` was specified. If mpv was started with [`Handle::create()`], idle mode is enabled by default.)
    #[deprecated = "This is equivalent to using mpv_observe_property() on the `idle-active` property. The event is redundant, and might be removed in the far future. As a further warning, this event is not necessarily sent at the right point anymore (at the start of the program), while the property behaves correctly."]
    Idle,
    /// Sent every time after a video frame is displayed.
    ///
    /// Note that currently this will be sent in lower frequency if there is no video, or playback is paused - but that will be removed in the future, and it will be restricted to video frames only.
    #[deprecated = "Use `Handle::observe_property()` with relevant properties instead (such as `playback-time`)."]
    Tick,
    /// Triggered by the `script-message` input command.
    ///
    /// The command uses the first argument of the command as a client name (see [`Handle::client_name()`]) to dispatch the message and passes along all arguments starting from the second argument as strings.
    ClientMessage(ClientMessage),
    /// Happens after video changed in some way. This can happen on resolution changes, pixel format changes, or video filter changes. The event is sent after the video filters and the VO are reconfigured. Applications embedding a mpv window should listen to this event to resize the window if needed.
    ///
    /// Note that this event can happen sporadically, and you should check yourself whether the video parameters really changed before doing something expensive.
    VideoReconfig,
    /// Similar to [`Event::VideoReconfig`].
    ///
    /// This is relatively uninteresting because there is no such thing as audio output embedding.
    AudioReconfig,
    /// Happens when a seek was initiated. Playback stops.
    ///
    /// Usually it will resume with [`Event::PlaybackRestart`] as soon as the seek is finished.
    Seek,
    /// There was a discontinuity of some sort (like a seek), and playback was reinitialized.
    ///
    /// Usually happens at the start of playback and after seeking. The main purpose is allowing the client to detect when a seek request is finished.
    PlaybackRestart,
    /// Event sent when a property observed with [`Handle::observe_property()`] is changed.
    PropertyChange(PropertyChange),
    /// Happens if the internal per-[`mpv_handle`] ringbuffer overflows, and at least 1 event had to be dropped.
    ///
    /// This can happen if the client doesn't read the event queue quickly enough with [`Handle::wait_event()`], or if the client makes a very large number of asynchronous calls at once.
    ///
    /// Event delivery will continue normally once this event was returned (this forces the client to empty the queue completely).
    QueueOverflow,
    /// Triggered if a hook handler was registered with [`Handle::hook_add()`], and the hook is invoked.
    ///
    /// If you receive this, you **must** handle it and continue the hook with [`Handle::hook_continue()`].
    Hook(Hook),
}

/// Details provided to [`Event::LogMessage`].
#[derive(Debug)]
pub struct LogMessage {
    /// The level of this log message.
    pub level: LogLevel,
    /// The module prefix, identifies the sender of the message.
    ///
    /// As a special case, if the message buffer overflows, this will be set to the string "overflow" (which doesn't appear as a prefix otherwise), and the text field will contain an informative message.
    pub prefix: String,
    /// The log message. It consists of 1 line of text and is terminated with a newline character.
    pub text: String,
}

/// Details provided to [`Event::GetPropertyReply`].
#[derive(Debug)]
pub struct GetPropertyReply {
    /// Value of the property, or an error if one occurred.
    pub value: Result<PropertyValue>,
    /// Name of the property.
    pub name: String,
    /// `userdata` value passed to the mpv request which generated this event, if provided.
    // TODO: Update doc with link to async function when implemented.
    pub userdata: u64,
}

/// Details provided to [`Event::SetPropertyReply`].
#[derive(Debug)]
pub struct SetPropertyReply {
    /// The error setting the property, if any. mpv may also report a success code, which is retained in Ok(i32).
    pub error: Result<i32>,
    /// `userdata` value passed to the mpv request which generated this event, if provided.
    // TODO: Update doc with link to async function when implemented.
    pub userdata: u64,
}

/// Details provided to [`Event::CommandReply`].
#[derive(Debug)]
pub struct CommandReply {
    /// Result of a command (which may be `Node::None` even on success depending on the command), or an error if one occurred.
    pub result: Result<Node>,
    /// `userdata` value passed to the mpv request which generated this event, if provided.
    // TODO: Update doc with link to async function when implemented.
    pub userdata: u64,
}

/// Details provided to [`Event::StartFile`].
#[derive(Debug)]
pub struct StartFile {
    /// Playlist entry ID of the file being loaded now.
    pub playlist_entry_id: i64,
}

/// Details provided to [`Event::EndFile`].
#[derive(Debug)]
pub struct EndFile {
    /// The reason why the file ended.
    pub reason: EndFileReason,
    /// Playlist entry ID of the file that was being played or attempted to be played.
    ///
    /// This has the same value as the [`StartFile.playlist_entry_id`](field@StartFile::playlist_entry_id) field in the corresponding [`Event::StartFile`] event.
    pub playlist_entry_id: i64,
    /// If loading ended, because the playlist entry to be played was for example a playlist, and the current playlist entry is replaced with a number of other entries.
    ///
    /// This may happen at least with [`EndFileReason::Redirect`] (other event types may use this for similar but different purposes in the future).
    /// In this case, [`playlist_insert_id`] will be set to the playlist entry ID of the first inserted entry, and [`playlist_insert_num_entries`] to the total number of inserted playlist entries.
    /// Note this in this specific case, the ID of the last inserted entry is [`playlist_insert_id`] + [`playlist_insert_num_entries`] - 1.
    /// Beware that depending on circumstances, you may observe the new playlist entries before seeing the event (e.g. reading the `playlist` property or getting a property change notification before receiving the event).
    ///
    /// [`playlist_insert_id`]: field@EndFile::playlist_insert_id
    /// [`playlist_insert_num_entries`]: field@EndFile::playlist_insert_num_entries
    pub playlist_insert_id: i64,
    /// See [`playlist_insert_id`]. Only non-0 if [`playlist_insert_id`] is valid. Never negative.
    ///
    /// [`playlist_insert_id`]: field@EndFile::playlist_insert_id
    pub playlist_insert_num_entries: i32,
}

/// Details provided to [`Event::ClientMessage`].
///
/// Arbitrary arguments chosen by the sender of the message. What these arguments mean is up to the sender and receiver.
#[derive(Debug)]
pub struct ClientMessage(pub Vec<String>);

/// Details provided to [`Event::PropertyChange`].
#[derive(Debug)]
pub struct PropertyChange {
    /// Name of the property.
    pub name: String,
    /// New value of the property, or [`PropertyValue::None`] if an error occurred.
    ///
    /// Note that mpv does not propagate error details for the [`PropertyChange`] event.
    /// Any [`Err`] result will be a [`RustError`](error::RustError) created while attempting to parse the data.
    pub value: Result<PropertyValue>,
    /// `userdata` value passed to the mpv request which generated this event, if provided.
    // TODO: Update doc with link to async function when implemented.
    pub userdata: u64,
}

/// Details provided to [`Event::Hook`].
#[derive(Debug)]
pub struct Hook {
    /// The hook name as passed to [`Handle::hook_add()`].
    pub name: String,
    /// Internal ID which must be passed to [`Handle::hook_continue()`].
    pub id: u64,
    /// `userdata` value passed to the mpv request which generated this event, if provided.
    // TODO: Update doc with link to async function when implemented.
    pub userdata: u64,
}

impl Event {
    pub(crate) fn from_ptr(ptr: *const mpv_event) -> Result<Event> {
        check_null!(ptr);
        let event = unsafe { *ptr };

        match event.event_id {
            libmpv_client_sys::mpv_event_id_MPV_EVENT_NONE => Ok(Event::None),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_SHUTDOWN => Ok(Event::Shutdown),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_LOG_MESSAGE => Ok(Event::LogMessage(LogMessage::from_event(event)?)),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_GET_PROPERTY_REPLY => Ok(Event::GetPropertyReply(GetPropertyReply::from_event(event)?)),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_SET_PROPERTY_REPLY => Ok(Event::SetPropertyReply(SetPropertyReply::from_event(event)?)),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_COMMAND_REPLY => Ok(Event::CommandReply(CommandReply::from_event(event)?)),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_START_FILE => Ok(Event::StartFile(StartFile::from_event(event)?)),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_END_FILE => Ok(Event::EndFile(EndFile::from_event(event)?)),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_FILE_LOADED => Ok(Event::FileLoaded),
            #[allow(deprecated)]
            libmpv_client_sys::mpv_event_id_MPV_EVENT_IDLE => Ok(Event::Idle),
            #[allow(deprecated)]
            libmpv_client_sys::mpv_event_id_MPV_EVENT_TICK => Ok(Event::Tick),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_CLIENT_MESSAGE => Ok(Event::ClientMessage(ClientMessage::from_event(event)?)),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_VIDEO_RECONFIG => Ok(Event::VideoReconfig),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_AUDIO_RECONFIG => Ok(Event::AudioReconfig),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_SEEK => Ok(Event::Seek),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_PLAYBACK_RESTART => Ok(Event::PlaybackRestart),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_PROPERTY_CHANGE => Ok(Event::PropertyChange(PropertyChange::from_event(event)?)),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_QUEUE_OVERFLOW => Ok(Event::QueueOverflow),
            libmpv_client_sys::mpv_event_id_MPV_EVENT_HOOK => Ok(Event::Hook(Hook::from_event(event)?)),
            _ => unimplemented!(),
        }
    }
}

impl LogMessage {
    fn from_event(event: mpv_event) -> Result<Self> {
        check_null!(event.data);
        let event_log_message = unsafe { *(event.data as *const mpv_event_log_message) };

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

        check_null!(event_log_message.prefix);
        let prefix = unsafe { CStr::from_ptr(event_log_message.prefix) }.to_str()?.to_string();

        check_null!(event_log_message.text);
        let text = unsafe { CStr::from_ptr(event_log_message.text) }.to_str()?.to_string();

        Ok(Self { level, prefix, text })
    }
}

impl GetPropertyReply {
    fn from_event(event: mpv_event) -> Result<Self> {
        check_null!(event.data);
        let event_prop = unsafe { *(event.data as *const mpv_event_property) };

        check_null!(event_prop.name);
        let name = unsafe { CStr::from_ptr(event_prop.name).to_str()?.to_string() };

        let value = error_to_result_code(event.error)
            .and_then(|_| {
                unsafe { PropertyValue::from_mpv(event_prop.format, event_prop.data) }
            });

        let userdata = event.reply_userdata;

        Ok(Self { value, name, userdata })
    }
}

impl SetPropertyReply {
    fn from_event(event: mpv_event) -> Result<Self> {
        let error = error_to_result_code(event.error);

        let userdata = event.reply_userdata;

        Ok(Self { error, userdata })
    }
}

impl CommandReply {
    fn from_event(event: mpv_event) -> Result<Self> {
        check_null!(event.data);
        let event_command = unsafe { *(event.data as *const mpv_event_command) };

        let result = error_to_result_code(event.error)
            .and_then(|_| {
                unsafe { Node::from_node_ptr(&event_command.result) }
            });

        let userdata = event.reply_userdata;

        Ok(Self { result, userdata })
    }
}

impl ClientMessage {
    fn from_event(event: mpv_event) -> Result<Self> {
        check_null!(event.data);
        let event_client_message = unsafe { *(event.data as *const mpv_event_client_message) };

        let mut args = Vec::with_capacity(event_client_message.num_args as usize);

        check_null!(event_client_message.args);
        let event_args = unsafe { std::slice::from_raw_parts(event_client_message.args, event_client_message.num_args as usize) };

        for event_arg in event_args {
            check_null!(event_arg);
            args.push(unsafe { CStr::from_ptr(*event_arg).to_str()?.to_string() });
        }

        Ok(Self(args))
    }
}

impl StartFile {
    fn from_event(event: mpv_event) -> Result<Self> {
        check_null!(event.data);
        let event_start_file = unsafe { *(event.data as *const mpv_event_start_file) };

        Ok(Self { playlist_entry_id: event_start_file.playlist_entry_id })
    }
}

impl EndFile {
    fn from_event(event: mpv_event) -> Result<Self> {
        check_null!(event.data);
        let event_end_file = unsafe { *(event.data as *const mpv_event_end_file) };

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
            playlist_insert_num_entries: event_end_file.playlist_insert_num_entries,
        })
    }
}

impl PropertyChange {
    fn from_event(event: mpv_event) -> Result<Self> {
        check_null!(event.data);
        let event_prop = unsafe { *(event.data as *const mpv_event_property) };

        check_null!(event_prop.name);
        let name = unsafe { CStr::from_ptr(event_prop.name).to_str()?.to_string() };

        let value = unsafe { PropertyValue::from_mpv(event_prop.format, event_prop.data) };

        let userdata = event.reply_userdata;

        Ok(Self { value, name, userdata })
    }
}

impl Hook {
    fn from_event(event: mpv_event) -> Result<Self> {
        check_null!(event.data);
        let event_hook = unsafe { *(event.data as *const mpv_event_hook) };

        check_null!(event_hook.name);
        let name = unsafe { CStr::from_ptr(event_hook.name) }.to_str()?.to_string();

        let id = event_hook.id;

        let userdata = event.reply_userdata;

        Ok(Self { name, id, userdata })
    }
}

#[derive(Debug)]
/// An enum of the possible values returned in a [`GetPropertyReply`] or a [`PropertyChange`].
pub enum PropertyValue {
    /// Sometimes used for empty values or errors. See [`Format::NONE`].
    None,
    /// A raw property string. See [`Format::STRING`].
    String(String),
    /// An OSD property string. See [`Format::OSD_STRING`].
    OsdString(OsdString),
    /// A flag property. See [`Format::FLAG`].
    Flag(bool),
    /// An int64 property. See [`Format::INT64`].
    Int64(i64),
    /// A double property. See [`Format::DOUBLE`].
    Double(f64),
    /// A [`Node`] property. See [`Format::NODE`].
    Node(Node),
    /// A [`NodeArray`] property. See [`Format::NODE_ARRAY`].
    NodeArray(NodeArray),
    /// A [`NodeMap`] property. See [`Format::NODE_MAP`].
    NodeMap(NodeMap),
    /// A [`ByteArray`] property. See [`Format::BYTE_ARRAY`].
    ByteArray(ByteArray),
}

impl PropertyValue {
    pub(crate) unsafe fn from_mpv(format: mpv_format, data: *mut c_void) -> Result<Self> {
        match format {
            libmpv_client_sys::mpv_format_MPV_FORMAT_NONE => Ok(Self::None),
            libmpv_client_sys::mpv_format_MPV_FORMAT_STRING => Ok(Self::String(unsafe { String::from_ptr(data)? })),
            libmpv_client_sys::mpv_format_MPV_FORMAT_OSD_STRING => Ok(Self::OsdString(unsafe { OsdString::from_ptr(data)? })),
            libmpv_client_sys::mpv_format_MPV_FORMAT_FLAG => Ok(Self::Flag(unsafe { bool::from_ptr(data)? })),
            libmpv_client_sys::mpv_format_MPV_FORMAT_INT64 => Ok(Self::Int64(unsafe { i64::from_ptr(data)? })),
            libmpv_client_sys::mpv_format_MPV_FORMAT_DOUBLE => Ok(Self::Double(unsafe { f64::from_ptr(data)? })),
            libmpv_client_sys::mpv_format_MPV_FORMAT_NODE => Ok(Self::Node(unsafe { Node::from_ptr(data)? })),
            libmpv_client_sys::mpv_format_MPV_FORMAT_NODE_ARRAY => Ok(Self::NodeArray(unsafe { NodeArray::from_ptr(data)? })),
            libmpv_client_sys::mpv_format_MPV_FORMAT_NODE_MAP => Ok(Self::NodeMap(unsafe { NodeMap::from_ptr(data)? })),
            libmpv_client_sys::mpv_format_MPV_FORMAT_BYTE_ARRAY => Ok(Self::ByteArray(unsafe { ByteArray::from_ptr(data)? })),
            _ => unimplemented!()
        }
    }
}