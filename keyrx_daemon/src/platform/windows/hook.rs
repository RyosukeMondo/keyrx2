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

unsafe extern "system" fn low_level_keyboard_proc(
    code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if code == HC_ACTION as i32 {
        let kbd_struct = *(l_param as *const KBDLLHOOKSTRUCT);

        // Ignore injected events to avoid infinite loops
        if kbd_struct.flags & LLKHF_INJECTED != 0 {
            return CallNextHookEx(null_mut(), code, w_param, l_param);
        }

        if let Some(keycode) = vk_to_keycode(kbd_struct.vkCode as u16) {
            let event = match w_param as u32 {
                WM_KEYDOWN | WM_SYSKEYDOWN => Some(KeyEvent::press(keycode)),
                WM_KEYUP | WM_SYSKEYUP => Some(KeyEvent::release(keycode)),
                _ => None,
            };

            if let Some(event) = event {
                if let Some(sender) = EVENT_SENDER.get() {
                    // Try to send the event. If the channel is full, we might have lag,
                    // but we must return quickly.
                    let _ = sender.try_send(event);

                    // Always suppress the original event when grabbing is active.
                    // This will be managed by the InputDevice trait implementation.
                    return 1;
                }
            }
        }
    }

    CallNextHookEx(null_mut(), code, w_param, l_param)
}
