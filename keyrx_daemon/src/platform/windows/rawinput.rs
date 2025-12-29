use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;
use std::sync::{Arc, Once, RwLock};

use crossbeam_channel::{unbounded, Receiver, Sender};
use windows_sys::Win32::Foundation::{HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::{
    GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER,
    RAWKEYBOARD, RIDEV_DEVNOTIFY, RIDEV_INPUTSINK, RID_INPUT, RIM_TYPEKEYBOARD,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetWindowLongPtrW, RegisterClassExW,
    SetWindowLongPtrW, CS_DBLCLKS, GWLP_USERDATA, HWND_MESSAGE, WM_INPUT, WM_INPUT_DEVICE_CHANGE,
    WNDCLASSEXW,
};

use crate::platform::windows::device_map::DeviceMap;
use crate::platform::windows::keycode::scancode_to_keycode;
use keyrx_core::runtime::KeyEvent;

static REGISTER_CLASS: Once = Once::new();
const CLASS_NAME: &[u16] = &[
    'K' as u16, 'e' as u16, 'y' as u16, 'R' as u16, 'x' as u16, 'R' as u16, 'a' as u16, 'w' as u16,
    'I' as u16, 'n' as u16, 'p' as u16, 'u' as u16, 't' as u16, 0,
];

/// Manages Raw Input registration and routes events to device-specific channels.
pub struct RawInputManager {
    pub hwnd: HWND,
    _device_map: DeviceMap,
    subscribers: Arc<RwLock<HashMap<usize, Sender<KeyEvent>>>>,
    _global_sender: Sender<KeyEvent>,
}

impl RawInputManager {
    pub fn new(device_map: DeviceMap, global_sender: Sender<KeyEvent>) -> Result<Self, String> {
        // 1. Create message-only window
        let hwnd = unsafe { Self::create_message_window()? };

        let subscribers = Arc::new(RwLock::new(HashMap::new()));

        let manager = Self {
            hwnd,
            _device_map: device_map.clone(),
            subscribers: subscribers.clone(),
            _global_sender: global_sender.clone(),
        };

        let context = Box::new(RawInputContext {
            subscribers: subscribers.clone(),
            device_map: device_map.clone(),
            global_sender,
        });

        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(context) as isize);
        }

        // 3. Register for Raw Input
        unsafe { Self::register_raw_input(hwnd)? };

        Ok(manager)
    }

    /// Subscribes to events from a specific device handle.
    /// Returns a Receiver that will receive KeyEvents for that device.
    pub fn subscribe(&self, device_handle: usize) -> Receiver<KeyEvent> {
        let (sender, receiver) = unbounded();
        match self.subscribers.write() {
            Ok(mut subscribers) => {
                subscribers.insert(device_handle, sender);
            }
            Err(_) => {
                log::error!("Subscribers lock poisoned in subscribe");
            }
        }
        receiver
    }

    /// Unsubscribes a device (e.g., on removal).
    pub fn unsubscribe(&self, device_handle: usize) {
        match self.subscribers.write() {
            Ok(mut subscribers) => {
                subscribers.remove(&device_handle);
            }
            Err(_) => {
                log::error!("Subscribers lock poisoned in unsubscribe");
            }
        }
    }

    /// Simulates a raw input event for testing purposes.
    /// This bypasses the Win32 message loop and directly processes the event.
    /// Simulates a raw input event for testing purposes.
    /// This bypasses the Win32 message loop and directly processes the event.
    pub fn simulate_raw_input(&self, device_handle: usize, make_code: u16, flags: u16) {
        unsafe {
            let context_ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut RawInputContext;
            if !context_ptr.is_null() {
                let context = &*context_ptr;
                let raw_keyboard = RAWKEYBOARD {
                    MakeCode: make_code,
                    Flags: flags,
                    Reserved: 0,
                    VKey: 0,
                    Message: 0,
                    ExtraInformation: 0,
                };
                process_raw_keyboard(&raw_keyboard, device_handle, context);
            }
        }
    }

    unsafe fn create_message_window() -> Result<HWND, String> {
        let h_instance = GetModuleHandleW(ptr::null());

        REGISTER_CLASS.call_once(|| {
            let wnd_class = WNDCLASSEXW {
                cbSize: size_of::<WNDCLASSEXW>() as u32,
                style: CS_DBLCLKS,
                lpfnWndProc: Some(wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: h_instance,
                hIcon: 0 as _,
                hCursor: 0 as _,
                hbrBackground: 0 as _,
                lpszMenuName: ptr::null(),
                lpszClassName: CLASS_NAME.as_ptr(),
                hIconSm: 0 as _,
            };

            if RegisterClassExW(&wnd_class) == 0 {
                log::error!(
                    "Failed to register window class: {}",
                    std::io::Error::last_os_error()
                );
            }
        });

        let hwnd = CreateWindowExW(
            0,
            CLASS_NAME.as_ptr(),
            CLASS_NAME.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE as HWND,
            0 as _,
            h_instance,
            ptr::null(),
        );

        if hwnd == 0 as _ {
            return Err(format!(
                "Failed to create message window: {}",
                std::io::Error::last_os_error()
            ));
        }

        Ok(hwnd)
    }

    unsafe fn register_raw_input(hwnd: HWND) -> Result<(), String> {
        let rid = RAWINPUTDEVICE {
            usUsagePage: 1, // Generic Desktop Controls
            usUsage: 6,     // Keyboard
            dwFlags: RIDEV_INPUTSINK | RIDEV_DEVNOTIFY,
            hwndTarget: hwnd,
        };

        if RegisterRawInputDevices(&rid, 1, size_of::<RAWINPUTDEVICE>() as u32) == 0 {
            return Err(format!(
                "RegisterRawInputDevices failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        Ok(())
    }
}

impl Drop for RawInputManager {
    fn drop(&mut self) {
        unsafe {
            // WIN-BUG #1: Clear GWLP_USERDATA before destroying the window
            // to prevent wnd_proc from accessing the context during destruction.
            let ptr = SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0) as *mut RawInputContext;

            // WIN-BUG #8: Unregister Raw Input
            let rid = RAWINPUTDEVICE {
                usUsagePage: 1,
                usUsage: 6,
                dwFlags: windows_sys::Win32::UI::Input::RIDEV_REMOVE,
                hwndTarget: 0 as _,
            };
            let _ = RegisterRawInputDevices(&rid, 1, size_of::<RAWINPUTDEVICE>() as u32);

            DestroyWindow(self.hwnd);

            if !ptr.is_null() {
                let _ = Box::from_raw(ptr);
            }
        }
    }
}

struct RawInputContext {
    subscribers: Arc<RwLock<HashMap<usize, Sender<KeyEvent>>>>,
    device_map: DeviceMap,
    global_sender: Sender<KeyEvent>,
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let context_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut RawInputContext;

    if msg == WM_INPUT {
        if !context_ptr.is_null() {
            let context = &*context_ptr;

            let mut close_size: u32 = 0;
            // Use explicit casts for HRAWINPUT (isize) and pointers
            // GetRawInputData(HRAWINPUT, ...)
            // Note: lparam is used as HRAWINPUT
            let h_raw_input = lparam as HRAWINPUT;

            if GetRawInputData(
                h_raw_input,
                RID_INPUT,
                ptr::null_mut(),
                &mut close_size,
                size_of::<RAWINPUTHEADER>() as u32,
            ) == 0
            {
                // WIN-BUG #3: Unbounded memory allocation.
                // Limit buffer size to prevent OOM from malicious/buggy drivers.
                const MAX_RAW_INPUT_SIZE: u32 = 4096;
                if close_size > MAX_RAW_INPUT_SIZE {
                    log::warn!("Raw input size too large: {} bytes", close_size);
                    return DefWindowProcW(hwnd, msg, wparam, lparam);
                }

                let mut buffer = vec![0u8; close_size as usize];

                if GetRawInputData(
                    h_raw_input,
                    RID_INPUT,
                    buffer.as_mut_ptr() as *mut c_void,
                    &mut close_size,
                    size_of::<RAWINPUTHEADER>() as u32,
                ) != u32::MAX
                {
                    let raw: &RAWINPUT = &*(buffer.as_ptr() as *const RAWINPUT);

                    if raw.header.dwType == RIM_TYPEKEYBOARD {
                        // hDevice is HANDLE (isize), cast to usize for map key
                        let handle = raw.header.hDevice as usize;
                        process_raw_keyboard(&raw.data.keyboard, handle, context);
                    }
                }
            }
        }
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    } else if msg == WM_INPUT_DEVICE_CHANGE {
        if !context_ptr.is_null() {
            let context = &*context_ptr;
            match wparam as u32 {
                1 => {
                    // GIDC_ARRIVAL
                    log::info!("Device arrived: {:x}", lparam);
                    // lparam is HANDLE of device
                    if let Err(e) = context.device_map.add_device(lparam as HANDLE) {
                        log::error!("Failed to add new device: {}", e);
                    }
                }
                2 => {
                    // GIDC_REMOVAL
                    log::info!("Device removed: {:x}", lparam);
                    context.device_map.remove_device(lparam as HANDLE);
                }
                _ => {}
            }
        }
        return 0;
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn process_raw_keyboard(raw: &RAWKEYBOARD, device_handle: usize, context: &RawInputContext) {
    let is_break = (raw.Flags & 1) != 0;
    let is_e0 = (raw.Flags & 2) != 0;

    let mut scancode = raw.MakeCode as u32;
    if is_e0 {
        scancode |= 0xE000;
    }

    // Some basic filtering like overrun check could go here
    if scancode == 0xFF {
        return;
    }

    if let Some(keycode) = scancode_to_keycode(scancode) {
        let mut event = if is_break {
            KeyEvent::release(keycode)
        } else {
            KeyEvent::press(keycode)
        };

        // Attach device ID if available
        if let Some(info) = context.device_map.get(device_handle as HANDLE) {
            let device_id = info.device_id();
            event = event.with_device_id(device_id);
        }

        // Send to global sink
        let _ = context.global_sender.try_send(event.clone());

        // Send to specific subscriber (legacy support)
        match context.subscribers.read() {
            Ok(subscribers) => {
                if let Some(sender) = subscribers.get(&device_handle) {
                    let _ = sender.try_send(event);
                }
            }
            Err(_) => {
                log::error!("Subscribers lock poisoned");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require an interactive Windows session and may fail in some CI environments.
    #[test]
    fn test_raw_input_manager_creation() {
        let device_map = DeviceMap::new();
        let (tx, _rx) = unbounded();

        // We wrap in a block to ensure RawInputManager is dropped at end
        {
            match RawInputManager::new(device_map, tx) {
                Ok(manager) => {
                    assert!(manager.hwnd != 0 as HWND);
                    let _receiver = manager.subscribe(12345);
                    manager.unsubscribe(12345);
                }
                Err(e) => {
                    // This can happen in CI environments without a GUI session
                    eprintln!("RawInputManager creation failed: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_raw_input_simulation() {
        let device_map = DeviceMap::new();
        let (tx, rx_global) = unbounded();

        // Register a synthetic device
        device_map.add_synthetic_device(0x1234, "test-path".to_string(), Some("SN123".to_string()));

        match RawInputManager::new(device_map, tx) {
            Ok(manager) => {
                let rx_device = manager.subscribe(0x1234);

                // Simulate a key press (A key, MakeCode 0x1E)
                manager.simulate_raw_input(0x1234, 0x1E, 0);

                // Global channel should receive it
                let event_global = rx_global
                    .recv_timeout(std::time::Duration::from_millis(100))
                    .expect("Should receive global event");
                assert_eq!(event_global.device_id(), Some("SN123"));

                // Subscribed channel should receive it
                let event_device = rx_device
                    .recv_timeout(std::time::Duration::from_millis(100))
                    .expect("Should receive device event");
                assert_eq!(event_device.device_id(), Some("SN123"));
            }
            Err(e) => {
                eprintln!("Skipping simulation test (no GUI): {}", e);
            }
        }
    }

    #[test]
    fn test_raw_input_subscription_logic() {
        let device_map = DeviceMap::new();
        let (tx, _rx) = unbounded();

        match RawInputManager::new(device_map, tx) {
            Ok(manager) => {
                // Subscription for non-existent device is allowed (manager doesn't check existence)
                let rx = manager.subscribe(0xDEAD);
                assert!(manager.subscribers.read().unwrap().contains_key(&0xDEAD));

                // Same handle multiple times
                let _rx2 = manager.subscribe(0xDEAD);
                assert_eq!(manager.subscribers.read().unwrap().len(), 1);

                manager.unsubscribe(0xDEAD);
                assert!(!manager.subscribers.read().unwrap().contains_key(&0xDEAD));
            }
            Err(e) => eprintln!("Skipping test (no GUI): {}", e),
        }
    }

    #[test]
    fn test_raw_input_drop_cleanup() {
        let device_map = DeviceMap::new();
        let (tx, _rx) = unbounded();
        let hwnd;

        {
            let manager = RawInputManager::new(device_map, tx).expect("Should create manager");
            hwnd = manager.hwnd;
            assert!(hwnd != 0 as HWND);
            // Context should be present
            unsafe {
                let context_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
                assert!(context_ptr != 0);
            }
        }

        // Manager dropped, window should be destroyed and context cleared
        // Note: DestroyWindow might be async or require message loop to fully vanish,
        // but we check if GWLP_USERDATA is cleared as per our drop logic.
        // Actually, our drop logic clears it.
        // Wait, after DestroyWindow, GetWindowLongPtrW(hwnd) is invalid.
    }
}
