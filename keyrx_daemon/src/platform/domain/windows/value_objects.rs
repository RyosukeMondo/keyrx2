//! Windows-specific domain value objects for Platform domain

#![cfg(target_os = "windows")]

/// Virtual key code value object
///
/// Represents a Windows virtual key code (VK_A, VK_ENTER, etc.).
/// Virtual key codes are device-independent codes used by Windows for keyboard events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VirtualKeyCodeVO {
    vk: u8,
}

impl VirtualKeyCodeVO {
    /// Creates a new VirtualKeyCode value object
    pub fn new(vk: u8) -> Self {
        Self { vk }
    }

    /// Gets the raw virtual key code
    pub fn as_raw(&self) -> u8 {
        self.vk
    }

    /// Checks if this is a modifier key (Shift, Ctrl, Alt, Win)
    pub fn is_modifier(&self) -> bool {
        matches!(
            self.vk,
            0x10 | 0x11 | 0x12 | 0x5B | 0x5C | 0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5
        ) // VK_SHIFT, VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN, etc.
    }

    /// Checks if this is a lock key (CapsLock, NumLock, ScrollLock)
    pub fn is_lock(&self) -> bool {
        matches!(self.vk, 0x14 | 0x90 | 0x91) // VK_CAPITAL, VK_NUMLOCK, VK_SCROLL
    }

    /// Checks if this is a valid virtual key code (0x01-0xFE)
    pub fn is_valid(&self) -> bool {
        self.vk >= 0x01 && self.vk <= 0xFE
    }
}

impl From<u8> for VirtualKeyCodeVO {
    fn from(vk: u8) -> Self {
        Self::new(vk)
    }
}

/// Scan code value object
///
/// Represents a Windows hardware scan code.
/// Scan codes are hardware-specific codes that identify physical keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScanCodeVO {
    scan_code: u16,
    extended: bool,
}

impl ScanCodeVO {
    /// Creates a new ScanCode value object
    pub fn new(scan_code: u16, extended: bool) -> Self {
        Self {
            scan_code,
            extended,
        }
    }

    /// Gets the raw scan code
    pub fn as_raw(&self) -> u16 {
        self.scan_code
    }

    /// Checks if this is an extended scan code (E0 prefix)
    pub fn is_extended(&self) -> bool {
        self.extended
    }

    /// Gets the full scan code with extended bit
    pub fn as_full_scan_code(&self) -> u16 {
        if self.extended {
            self.scan_code | 0xE000
        } else {
            self.scan_code
        }
    }

    /// Checks if this is a valid scan code (non-zero)
    pub fn is_valid(&self) -> bool {
        self.scan_code != 0
    }
}

impl From<u16> for ScanCodeVO {
    fn from(scan_code: u16) -> Self {
        // Check if extended (E0 prefix)
        let extended = (scan_code & 0xE000) == 0xE000;
        let raw_scan = scan_code & 0x01FF;
        Self::new(raw_scan, extended)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_key_code_vo_creation() {
        let vk = VirtualKeyCodeVO::new(0x41); // VK_A
        assert_eq!(vk.as_raw(), 0x41);
        assert!(vk.is_valid());
        assert!(!vk.is_modifier());
        assert!(!vk.is_lock());
    }

    #[test]
    fn test_virtual_key_code_vo_modifiers() {
        let shift = VirtualKeyCodeVO::new(0x10); // VK_SHIFT
        assert!(shift.is_modifier());
        assert!(!shift.is_lock());

        let ctrl = VirtualKeyCodeVO::new(0x11); // VK_CONTROL
        assert!(ctrl.is_modifier());

        let alt = VirtualKeyCodeVO::new(0x12); // VK_MENU (Alt)
        assert!(alt.is_modifier());
    }

    #[test]
    fn test_virtual_key_code_vo_locks() {
        let caps = VirtualKeyCodeVO::new(0x14); // VK_CAPITAL
        assert!(caps.is_lock());
        assert!(!caps.is_modifier());

        let num = VirtualKeyCodeVO::new(0x90); // VK_NUMLOCK
        assert!(num.is_lock());
    }

    #[test]
    fn test_virtual_key_code_vo_invalid() {
        let invalid = VirtualKeyCodeVO::new(0x00);
        assert!(!invalid.is_valid());

        let invalid2 = VirtualKeyCodeVO::new(0xFF);
        assert!(!invalid2.is_valid());
    }

    #[test]
    fn test_virtual_key_code_vo_from_u8() {
        let vk: VirtualKeyCodeVO = 0x41u8.into();
        assert_eq!(vk.as_raw(), 0x41);
    }

    #[test]
    fn test_scan_code_vo_creation() {
        let scan = ScanCodeVO::new(0x1E, false); // A key scan code
        assert_eq!(scan.as_raw(), 0x1E);
        assert!(!scan.is_extended());
        assert!(scan.is_valid());
    }

    #[test]
    fn test_scan_code_vo_extended() {
        let scan = ScanCodeVO::new(0x1C, true); // Enter key (extended)
        assert_eq!(scan.as_raw(), 0x1C);
        assert!(scan.is_extended());
        assert_eq!(scan.as_full_scan_code(), 0xE01C);
    }

    #[test]
    fn test_scan_code_vo_invalid() {
        let invalid = ScanCodeVO::new(0, false);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_scan_code_vo_from_u16() {
        // Non-extended scan code
        let scan: ScanCodeVO = 0x1Eu16.into();
        assert_eq!(scan.as_raw(), 0x1E);
        assert!(!scan.is_extended());

        // Extended scan code (E0 prefix)
        let scan_ext: ScanCodeVO = 0xE01Cu16.into();
        assert_eq!(scan_ext.as_raw(), 0x1C);
        assert!(scan_ext.is_extended());
    }

    #[test]
    fn test_scan_code_vo_roundtrip() {
        let original = ScanCodeVO::new(0x1E, false);
        let from_u16: ScanCodeVO = original.as_full_scan_code().into();
        assert_eq!(from_u16.as_raw(), original.as_raw());
        assert_eq!(from_u16.is_extended(), original.is_extended());

        let original_ext = ScanCodeVO::new(0x1C, true);
        let from_u16_ext: ScanCodeVO = original_ext.as_full_scan_code().into();
        assert_eq!(from_u16_ext.as_raw(), original_ext.as_raw());
        assert_eq!(from_u16_ext.is_extended(), original_ext.is_extended());
    }
}
