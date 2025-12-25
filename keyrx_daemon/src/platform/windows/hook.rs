use crate::platform::windows::keycode::vk_to_keycode;
use crossbeam_channel::Sender;
use keyrx_core::runtime::event::KeyEvent;
use std::ptr::null_mut;
use std::sync::OnceLock;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

static EVENT_SENDER: OnceLock<Sender<KeyEvent>> = OnceLock::new();

pub struct WindowsKeyboardHook {
    hook_id: HHOOK,
}

impl WindowsKeyboardHook {
    pub fn install(sender: Sender<KeyEvent>) -> Result<Self, String> {
        if EVENT_SENDER.set(sender).is_err() {
            return Err("Event sender already initialized".to_string());
        }

        unsafe {
            let hook_id =
                SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), null_mut(), 0);

            if hook_id == null_mut() {
                return Err("Failed to install keyboard hook".to_string());
            }

            Ok(Self { hook_id })
        }
    }
}

impl Drop for WindowsKeyboardHook {
    fn drop(&mut self) {
        unsafe {
            if self.hook_id != null_mut() {
                UnhookWindowsHookEx(self.hook_id);
            }
        }
    }
}

// Markers for events during E2E testing
const TEST_SIMULATED_PHYSICAL_MARKER: usize = 0x54455354; // "TEST"
                                                          // DAEMON_OUTPUT_MARKER removed as it's unused in this file

unsafe extern "system" fn low_level_keyboard_proc(
    code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if code == HC_ACTION as i32 {
        let kbd_struct = *(l_param as *const KBDLLHOOKSTRUCT);

        // Ignore injected events to avoid infinite loops,
        // UNLESS they have our special test marker (meaning they represent physical input for tests).
        if kbd_struct.flags & LLKHF_INJECTED != 0
            && kbd_struct.dwExtraInfo != TEST_SIMULATED_PHYSICAL_MARKER
        {
            return CallNextHookEx(null_mut(), code, w_param, l_param);
        }

        if let Some(keycode) = vk_to_keycode(kbd_struct.vkCode as u16) {
            log::debug!(
                "Hook received keycode: {:?}, extra: {:x}, flags: {:x}",
                keycode,
                kbd_struct.dwExtraInfo,
                kbd_struct.flags
            );
            let event = match w_param as u32 {
                WM_KEYDOWN | WM_SYSKEYDOWN => Some(KeyEvent::press(keycode)),
                WM_KEYUP | WM_SYSKEYUP => Some(KeyEvent::release(keycode)),
                _ => None,
            };

            if let Some(event) = event {
                if let Some(sender) = EVENT_SENDER.get() {
                    log::debug!("Hook sending event to daemon: {:?}", event);
                    let _ = sender.try_send(event);
                    return 1;
                }
            }
        }
    }

    CallNextHookEx(null_mut(), code, w_param, l_param)
}
