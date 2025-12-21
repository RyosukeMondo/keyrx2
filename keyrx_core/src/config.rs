//! Configuration loading and validation module
//!
//! This module handles loading and validating .krx binary configuration files
//! using rkyv for zero-copy deserialization.

use core::fmt;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

/// Version information for binary compatibility checking.
///
/// Uses semantic versioning with major.minor.patch components.
/// All fields are u8 to keep the size small (3 bytes total).
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct Version {
    /// Major version number (incompatible API changes)
    pub major: u8,
    /// Minor version number (backwards-compatible functionality)
    pub minor: u8,
    /// Patch version number (backwards-compatible bug fixes)
    pub patch: u8,
}

impl Version {
    /// Returns the current version of the configuration format.
    ///
    /// # Returns
    /// Version 1.0.0 - the initial stable version
    pub fn current() -> Self {
        Self {
            major: 1,
            minor: 0,
            patch: 0,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Keyboard key codes enumeration.
///
/// Represents all standard keyboard keys with explicit numeric values
/// to ensure binary stability across versions. Keys are organized by category.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u16)]
pub enum KeyCode {
    // Letters A-Z (0x00-0x19)
    A = 0x00,
    B = 0x01,
    C = 0x02,
    D = 0x03,
    E = 0x04,
    F = 0x05,
    G = 0x06,
    H = 0x07,
    I = 0x08,
    J = 0x09,
    K = 0x0A,
    L = 0x0B,
    M = 0x0C,
    N = 0x0D,
    O = 0x0E,
    P = 0x0F,
    Q = 0x10,
    R = 0x11,
    S = 0x12,
    T = 0x13,
    U = 0x14,
    V = 0x15,
    W = 0x16,
    X = 0x17,
    Y = 0x18,
    Z = 0x19,

    // Numbers 0-9 (0x20-0x29)
    Key0 = 0x20,
    Key1 = 0x21,
    Key2 = 0x22,
    Key3 = 0x23,
    Key4 = 0x24,
    Key5 = 0x25,
    Key6 = 0x26,
    Key7 = 0x27,
    Key8 = 0x28,
    Key9 = 0x29,

    // Function keys F1-F12 (0x30-0x3B)
    F1 = 0x30,
    F2 = 0x31,
    F3 = 0x32,
    F4 = 0x33,
    F5 = 0x34,
    F6 = 0x35,
    F7 = 0x36,
    F8 = 0x37,
    F9 = 0x38,
    F10 = 0x39,
    F11 = 0x3A,
    F12 = 0x3B,

    // Modifier keys (0x100-0x107)
    LShift = 0x100,
    RShift = 0x101,
    LCtrl = 0x102,
    RCtrl = 0x103,
    LAlt = 0x104,
    RAlt = 0x105,
    LMeta = 0x106,
    RMeta = 0x107,

    // Arrow keys (0x210-0x213)
    Left = 0x210,
    Right = 0x211,
    Up = 0x212,
    Down = 0x213,

    // Special keys (0x200+)
    Escape = 0x200,
    Enter = 0x201,
    Backspace = 0x202,
    Tab = 0x203,
    Space = 0x204,
    CapsLock = 0x205,
    Insert = 0x206,
    Delete = 0x207,
    Home = 0x208,
    End = 0x209,
    PageUp = 0x20A,
    PageDown = 0x20B,
    PrintScreen = 0x20C,
    ScrollLock = 0x20D,
    Pause = 0x20E,

    // Punctuation and symbols (0x40-0x5F)
    Minus = 0x40,
    Equal = 0x41,
    LeftBracket = 0x42,
    RightBracket = 0x43,
    Backslash = 0x44,
    Semicolon = 0x45,
    Apostrophe = 0x46,
    Grave = 0x47,
    Comma = 0x48,
    Period = 0x49,
    Slash = 0x4A,

    // Numpad keys (0x60-0x7F)
    NumLock = 0x60,
    NumpadDivide = 0x61,
    NumpadMultiply = 0x62,
    NumpadMinus = 0x63,
    NumpadPlus = 0x64,
    NumpadEnter = 0x65,
    Numpad1 = 0x66,
    Numpad2 = 0x67,
    Numpad3 = 0x68,
    Numpad4 = 0x69,
    Numpad5 = 0x6A,
    Numpad6 = 0x6B,
    Numpad7 = 0x6C,
    Numpad8 = 0x6D,
    Numpad9 = 0x6E,
    Numpad0 = 0x6F,
    NumpadPeriod = 0x70,

    // Additional function keys (0x80-0x8F)
    F13 = 0x80,
    F14 = 0x81,
    F15 = 0x82,
    F16 = 0x83,
    F17 = 0x84,
    F18 = 0x85,
    F19 = 0x86,
    F20 = 0x87,
    F21 = 0x88,
    F22 = 0x89,
    F23 = 0x8A,
    F24 = 0x8B,

    // Media keys (0x300-0x30F)
    Mute = 0x300,
    VolumeDown = 0x301,
    VolumeUp = 0x302,
    MediaPlayPause = 0x303,
    MediaStop = 0x304,
    MediaPrevious = 0x305,
    MediaNext = 0x306,
}

/// Key mapping types enumeration.
///
/// Defines the different types of key mappings supported by the remapping engine.
/// Currently only Simple mapping is implemented, with placeholders for future features.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub enum KeyMapping {
    /// Simple 1:1 key mapping (from -> to).
    ///
    /// Maps a single key press directly to another key.
    /// Example: CapsLock -> Escape
    Simple {
        /// The source key to remap from
        from: KeyCode,
        /// The target key to remap to
        to: KeyCode,
    },

    // Future variants (placeholders):
    // TapHold {
    //     key: KeyCode,
    //     tap_action: KeyCode,
    //     hold_action: KeyCode,
    //     hold_timeout_ms: u16,
    // },
    //
    // Layer {
    //     key: KeyCode,
    //     layer_id: u8,
    //     mappings: Vec<KeyMapping>,
    // },
}
