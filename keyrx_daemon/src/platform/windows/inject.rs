use crate::platform::windows::keycode::keycode_to_vk;
use std::mem::size_of;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

// Marker for events injected by the daemon
const DAEMON_OUTPUT_MARKER: usize = 0x4441454D; // "DAEM"

pub struct EventInjector;

impl EventInjector {
    #[allow(dead_code)]
    pub fn inject(&self, event: &keyrx_core::runtime::KeyEvent) -> Result<(), String> {
        let keycode = event.keycode();
        let is_release = event.is_release();

        let vk =
            keycode_to_vk(keycode).ok_or_else(|| format!("Unmapped keycode: {:?}", keycode))?;

        unsafe {
            let mut input = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: std::mem::zeroed(),
            };

            input.Anonymous.ki = KEYBDINPUT {
                wVk: vk,
                wScan: MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) as u16,
                dwFlags: (if is_release { KEYEVENTF_KEYUP } else { 0 }) | KEYEVENTF_SCANCODE,
                time: 0,
                dwExtraInfo: DAEMON_OUTPUT_MARKER,
            };

            // Set extended key flag for certain keys
            if is_extended_key(vk) {
                input.Anonymous.ki.dwFlags |= KEYEVENTF_EXTENDEDKEY;
            }

            if SendInput(1, &input, size_of::<INPUT>() as i32) == 0 {
                log::error!("SendInput failed: {}", std::io::Error::last_os_error());
                return Err("SendInput failed".to_string());
            }
        }

        Ok(())
    }
}

#[allow(dead_code)]
pub(crate) fn is_extended_key(vk: u16) -> bool {
    matches!(
        vk,
        VK_RMENU
            | VK_RCONTROL
            | VK_INSERT
            | VK_DELETE
            | VK_HOME
            | VK_END
            | VK_PRIOR
            | VK_NEXT
            | VK_UP
            | VK_DOWN
            | VK_LEFT
            | VK_RIGHT
    )
}
