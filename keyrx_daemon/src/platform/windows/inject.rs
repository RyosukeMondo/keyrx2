use crate::platform::windows::keycode::keycode_to_vk;
use keyrx_core::runtime::event::KeyEvent;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

pub struct EventInjector;

impl EventInjector {
    #[allow(dead_code)]
    pub fn inject(&self, event: KeyEvent) -> Result<(), String> {
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
                wScan: 0,
                dwFlags: if is_release { KEYEVENTF_KEYUP } else { 0 },
                time: 0,
                dwExtraInfo: 0,
            };

            // Set extended key flag for certain keys
            if is_extended_key(vk) {
                input.Anonymous.ki.dwFlags |= KEYEVENTF_EXTENDEDKEY;
            }

            if SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) == 0 {
                return Err("SendInput failed".to_string());
            }
        }

        Ok(())
    }
}

#[allow(dead_code)]
fn is_extended_key(vk: u16) -> bool {
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
            | VK_LEFT
            | VK_RIGHT
            | VK_UP
            | VK_DOWN
            | VK_NUMLOCK
            | VK_SNAPSHOT
            | VK_DIVIDE
    )
}
