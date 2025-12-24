#[cfg(test)]
mod tests {
    use crate::platform::windows::keycode::{keycode_to_vk, vk_to_keycode};
    use keyrx_core::config::KeyCode;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

    #[test]
    fn test_vk_mapping_completeness() {
        // Test a few common keys
        assert_eq!(vk_to_keycode(VK_A as u16), Some(KeyCode::A));
        assert_eq!(vk_to_keycode(VK_SPACE as u16), Some(KeyCode::Space));
        assert_eq!(vk_to_keycode(VK_LSHIFT as u16), Some(KeyCode::LShift));

        assert_eq!(keycode_to_vk(KeyCode::A), Some(VK_A as u16));
        assert_eq!(keycode_to_vk(KeyCode::Space), Some(VK_SPACE as u16));
        assert_eq!(keycode_to_vk(KeyCode::LShift), Some(VK_LSHIFT as u16));
    }

    #[test]
    fn test_unmapped_vk() {
        assert_eq!(vk_to_keycode(0x07), None); // Undefined VK code
    }

    #[test]
    fn test_extended_keys() {
        use crate::platform::windows::inject::is_extended_key;
        assert!(is_extended_key(VK_RMENU as u16));
        assert!(is_extended_key(VK_RCONTROL as u16));
        assert!(is_extended_key(VK_LEFT as u16));
        assert!(!is_extended_key(VK_A as u16));
    }
}
