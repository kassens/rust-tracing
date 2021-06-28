use serde::Serialize;

// This file contains types which specify the speedscope file format.
// https://raw.githubusercontent.com/jlfwong/speedscope/main/src/lib/file-format-spec.ts

pub struct SpeedscopeSchemaUrl;

impl Serialize for SpeedscopeSchemaUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("https://www.speedscope.app/file-format-schema.json")
    }
}

pub struct ProfileTypeEvented;
impl Serialize for ProfileTypeEvented {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("evented")
    }
}

/// Data shared between profiles
#[derive(Serialize)]
pub struct FileShared {
    pub frames: Vec<Frame>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    #[serde(rename = "$schema")]
    pub schema: SpeedscopeSchemaUrl,

    /// Data shared between profiles
    pub shared: FileShared,

    /// List of profile definitions
    pub profiles: Vec<EventedProfile>,

    /// The name of the contained profile group. If omitted, will use the name of
    /// the file itself.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The index into the `profiles` array that should be displayed upon file
    /// load. If omitted, will default to displaying the first profile in the
    /// file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_profile_index: Option<u32>,

    /// The name of the the program which exported this profile. This isn't
    /// consumed but can be helpful for debugging generated data by seeing what
    /// was generating it! Recommended format is "name@version". e.g. when the
    /// file was exported by speedscope v0.6.0 itself, it will be
    /// "speedscope@0.6.0"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exporter: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub col: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventedProfile {
    #[serde(rename = "type")]
    pub type_: ProfileTypeEvented,

    /// Name of the profile. Typically a filename for the source of the profile.
    pub name: String,

    /// Unit which all value are specified using in the profile.
    pub unit: ValueUnit,

    /// The starting value of the profile. This will typically be a timestamp.
    /// All event values will be relative to this startValue.
    pub start_value: Value,

    /// The final value of the profile. This will typically be a timestamp. This
    /// must be greater than or equal to the startValue. This is useful in
    /// situations where the recorded profile extends past the end of the recorded
    /// events, which may happen if nothing was happening at the end of the
    /// profile.
    pub end_value: Value,

    /// List of events that occured as part of this profile.
    /// The "at" field of every event must be in non-decreasing order.
    pub events: Vec<Event>,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ValueUnit {
    None,
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
    Bytes,
}

/// This will typically be a timestamp.
#[derive(Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Value(u64);

impl Value {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

/// An index into the frames array in the shared data within the profile
#[derive(Serialize, Clone, Copy)]
pub struct FrameIndex(u32);
impl FrameIndex {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Event {
    /// Indicates a stack frame opened. Every opened stack frame must have a
    /// corresponding close frame event, and the ordering must be balanced.
    #[serde(rename = "O", rename_all = "camelCase")]
    OpenFrame {
        at: Value,
        /// An index into the frames array in the shared data within the profile
        frame: FrameIndex,
    },

    #[serde(rename = "C", rename_all = "camelCase")]
    CloseFrame {
        at: Value,
        /// An index into the frames array in the shared data within the profile
        frame: FrameIndex,
    },
}

impl Event {
    pub fn frame(&self) -> FrameIndex {
        match self {
            Event::OpenFrame { frame, .. } | Event::CloseFrame { frame, .. } => *frame,
        }
    }
}
