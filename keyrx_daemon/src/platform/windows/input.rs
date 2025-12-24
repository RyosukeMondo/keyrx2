use crate::platform::windows::hook::WindowsKeyboardHook;
use crate::platform::{DeviceError, InputDevice};
use crossbeam_channel::{Receiver, Sender};
use keyrx_core::runtime::event::KeyEvent;

pub struct WindowsKeyboardInput {
    #[allow(dead_code)]
    receiver: Receiver<KeyEvent>,
    _hook: Option<WindowsKeyboardHook>,
    _sender: Sender<KeyEvent>,
}

impl WindowsKeyboardInput {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::bounded(1024);
        Self {
            receiver,
            _hook: None,
            _sender: sender,
        }
    }

    #[allow(dead_code)]
    pub fn is_grabbed(&self) -> bool {
        self._hook.is_some()
    }
}

impl InputDevice for WindowsKeyboardInput {
    fn next_event(&mut self) -> Result<KeyEvent, DeviceError> {
        self.receiver.try_recv().map_err(|e| match e {
            crossbeam_channel::TryRecvError::Empty => DeviceError::EndOfStream,
            crossbeam_channel::TryRecvError::Disconnected => DeviceError::EndOfStream,
        })
    }

    fn grab(&mut self) -> Result<(), DeviceError> {
        if self._hook.is_some() {
            return Ok(());
        }

        let hook = WindowsKeyboardHook::install(self._sender.clone())
            .map_err(|e| DeviceError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        self._hook = Some(hook);
        Ok(())
    }

    fn release(&mut self) -> Result<(), DeviceError> {
        self._hook = None;
        Ok(())
    }
}

impl Default for WindowsKeyboardInput {
    fn default() -> Self {
        Self::new()
    }
}
