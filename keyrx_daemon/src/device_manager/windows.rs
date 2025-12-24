use super::{DiscoveryError, KeyboardInfo};
use crate::platform::windows::WindowsKeyboardInput;
use keyrx_core::config::DeviceConfig;
use keyrx_core::runtime::{DeviceState, KeyLookup};
use log::info;
use std::path::PathBuf;

pub struct ManagedDevice {
    info: KeyboardInfo,
    input: WindowsKeyboardInput,
    lookup: KeyLookup,
    state: DeviceState,
    config_index: usize,
}

impl ManagedDevice {
    pub fn new(
        info: KeyboardInfo,
        input: WindowsKeyboardInput,
        config: &DeviceConfig,
        config_index: usize,
    ) -> Self {
        Self {
            info,
            input,
            lookup: KeyLookup::from_device_config(config),
            state: DeviceState::new(),
            config_index,
        }
    }

    pub fn info(&self) -> &KeyboardInfo {
        &self.info
    }

    pub fn input_mut(&mut self) -> &mut WindowsKeyboardInput {
        &mut self.input
    }

    pub fn input(&self) -> &WindowsKeyboardInput {
        &self.input
    }

    pub fn lookup(&self) -> &KeyLookup {
        &self.lookup
    }

    pub fn state_mut(&mut self) -> &mut DeviceState {
        &mut self.state
    }

    pub fn state(&self) -> &DeviceState {
        &self.state
    }

    pub fn config_index(&self) -> usize {
        self.config_index
    }

    pub fn rebuild_lookup(&mut self, config: &DeviceConfig) {
        self.lookup = KeyLookup::from_device_config(config);
    }

    pub fn lookup_and_state_mut(&mut self) -> (&KeyLookup, &mut DeviceState) {
        (&self.lookup, &mut self.state)
    }
}

pub struct DeviceManager {
    devices: Vec<ManagedDevice>,
}

pub struct RefreshResult {
    pub added: usize,
    pub removed: usize,
}

impl DeviceManager {
    pub fn discover(configs: &[DeviceConfig]) -> Result<Self, DiscoveryError> {
        info!("Windows platform detected: using global keyboard hook");

        // On Windows, we treat the global keyboard hook as a single managed device.
        // We match it against the first configuration (or the one that matches "*").
        let keyboard_info = KeyboardInfo {
            path: PathBuf::from("windows-global-hook"),
            name: "Windows Global Keyboard Hook".to_string(),
            serial: None,
            phys: None,
        };

        let mut matched_config = None;
        for (idx, config) in configs.iter().enumerate() {
            if super::match_device(&keyboard_info, &config.identifier.pattern) {
                matched_config = Some((idx, config));
                break;
            }
        }

        let (config_idx, config) = matched_config.ok_or_else(|| DiscoveryError::NoDevicesFound)?;

        let input = WindowsKeyboardInput::new();
        let managed = ManagedDevice::new(keyboard_info, input, config, config_idx);

        Ok(Self {
            devices: vec![managed],
        })
    }

    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    pub fn devices(&self) -> impl Iterator<Item = &ManagedDevice> {
        self.devices.iter()
    }

    pub fn devices_mut(&mut self) -> impl Iterator<Item = &mut ManagedDevice> {
        self.devices.iter_mut()
    }

    pub fn get_device(&self, index: usize) -> Option<&ManagedDevice> {
        self.devices.get(index)
    }

    pub fn get_device_mut(&mut self, index: usize) -> Option<&mut ManagedDevice> {
        self.devices.get_mut(index)
    }

    pub fn rebuild_lookups(&mut self, configs: &[DeviceConfig]) {
        for device in &mut self.devices {
            if let Some(config) = configs.get(device.config_index) {
                device.rebuild_lookup(config);
            }
        }
    }

    pub fn refresh(&mut self, _configs: &[DeviceConfig]) -> Result<RefreshResult, DiscoveryError> {
        // Hot-plug not really applicable to global hook on Windows
        Ok(RefreshResult {
            added: 0,
            removed: 0,
        })
    }
}
