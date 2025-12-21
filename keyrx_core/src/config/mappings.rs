use alloc::vec::Vec;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

use crate::config::conditions::Condition;
use crate::config::keys::KeyCode;
use crate::config::types::{Metadata, Version};

/// Base key mapping without conditional nesting
///
/// Used as the leaf mappings within conditional blocks.
/// This prevents infinite recursion in rkyv serialization.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
#[repr(C)]
pub enum BaseKeyMapping {
    /// Simple 1:1 key remapping
    Simple { from: KeyCode, to: KeyCode },
    /// Key acts as custom modifier (MD_00-MD_FE)
    Modifier { from: KeyCode, modifier_id: u8 },
    /// Key toggles custom lock (LK_00-LK_FE)
    Lock { from: KeyCode, lock_id: u8 },
    /// Dual tap/hold behavior
    TapHold {
        from: KeyCode,
        tap: KeyCode,
        hold_modifier: u8,
        threshold_ms: u16,
    },
    /// Output with physical modifiers
    ModifiedOutput {
        from: KeyCode,
        to: KeyCode,
        shift: bool,
        ctrl: bool,
        alt: bool,
        win: bool,
    },
}

/// Key mapping configuration
///
/// Defines all possible mapping types.
/// Conditional mappings contain base mappings to limit recursion depth to 1 level.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
#[repr(C)]
pub enum KeyMapping {
    /// Base mapping (non-conditional)
    Base(BaseKeyMapping),
    /// Conditional mappings (when/when_not)
    /// Contains only base mappings to avoid infinite recursion
    Conditional {
        condition: Condition,
        mappings: Vec<BaseKeyMapping>,
    },
}

/// Device identifier pattern for matching keyboards
///
/// The pattern string is used to match against device names/IDs.
/// Examples:
/// - "*" matches all devices
/// - "USB Keyboard" matches devices with that exact name
/// - Platform-specific patterns may be supported by the daemon
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
#[repr(C)]
pub struct DeviceIdentifier {
    /// Pattern string for matching device names/IDs
    pub pattern: alloc::string::String,
}

/// Device-specific configuration
///
/// Contains all key mappings for a specific device or device pattern.
/// Multiple devices can share the same configuration by using pattern matching.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
#[repr(C)]
pub struct DeviceConfig {
    /// Device identifier pattern
    pub identifier: DeviceIdentifier,
    /// List of key mappings for this device
    pub mappings: Vec<KeyMapping>,
}

/// Root configuration structure
///
/// This is the top-level structure that gets serialized to .krx binary format.
/// Contains all device configurations and metadata.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
#[repr(C)]
pub struct ConfigRoot {
    /// Binary format version
    pub version: Version,
    /// List of device-specific configurations
    pub devices: Vec<DeviceConfig>,
    /// Compilation metadata
    pub metadata: Metadata,
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use crate::config::conditions::ConditionItem;

    #[test]
    fn test_base_key_mapping_variants() {
        // Test Simple mapping
        let mapping1 = BaseKeyMapping::Simple {
            from: KeyCode::A,
            to: KeyCode::B,
        };
        assert_eq!(
            mapping1,
            BaseKeyMapping::Simple {
                from: KeyCode::A,
                to: KeyCode::B,
            }
        );

        // Test Modifier mapping
        let mapping2 = BaseKeyMapping::Modifier {
            from: KeyCode::CapsLock,
            modifier_id: 0x01,
        };
        if let BaseKeyMapping::Modifier { from, modifier_id } = mapping2 {
            assert_eq!(from, KeyCode::CapsLock);
            assert_eq!(modifier_id, 0x01);
        } else {
            panic!("Expected Modifier variant");
        }

        // Test Lock mapping
        let mapping3 = BaseKeyMapping::Lock {
            from: KeyCode::ScrollLock,
            lock_id: 0x02,
        };
        if let BaseKeyMapping::Lock { from, lock_id } = mapping3 {
            assert_eq!(from, KeyCode::ScrollLock);
            assert_eq!(lock_id, 0x02);
        } else {
            panic!("Expected Lock variant");
        }

        // Test TapHold mapping
        let mapping4 = BaseKeyMapping::TapHold {
            from: KeyCode::Space,
            tap: KeyCode::Space,
            hold_modifier: 0x00,
            threshold_ms: 200,
        };
        if let BaseKeyMapping::TapHold {
            from,
            tap,
            hold_modifier,
            threshold_ms,
        } = mapping4
        {
            assert_eq!(from, KeyCode::Space);
            assert_eq!(tap, KeyCode::Space);
            assert_eq!(hold_modifier, 0x00);
            assert_eq!(threshold_ms, 200);
        } else {
            panic!("Expected TapHold variant");
        }

        // Test ModifiedOutput mapping
        let mapping5 = BaseKeyMapping::ModifiedOutput {
            from: KeyCode::A,
            to: KeyCode::A,
            shift: true,
            ctrl: false,
            alt: false,
            win: false,
        };
        if let BaseKeyMapping::ModifiedOutput {
            shift,
            ctrl,
            alt,
            win,
            ..
        } = mapping5
        {
            assert!(shift);
            assert!(!ctrl);
            assert!(!alt);
            assert!(!win);
        } else {
            panic!("Expected ModifiedOutput variant");
        }
    }

    #[test]
    fn test_key_mapping_variants() {
        // Test Base mapping
        let mapping1 = KeyMapping::Base(BaseKeyMapping::Simple {
            from: KeyCode::A,
            to: KeyCode::B,
        });
        assert!(matches!(mapping1, KeyMapping::Base(_)));

        // Test Conditional mapping
        let mapping2 = KeyMapping::Conditional {
            condition: Condition::ModifierActive(0x01),
            mappings: alloc::vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            },],
        };
        if let KeyMapping::Conditional {
            condition,
            mappings,
        } = &mapping2
        {
            assert_eq!(*condition, Condition::ModifierActive(0x01));
            assert_eq!(mappings.len(), 1);
        } else {
            panic!("Expected Conditional variant");
        }
    }

    #[test]
    fn test_device_config_creation() {
        use alloc::string::String;

        let device_config = DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: String::from("*"),
            },
            mappings: alloc::vec![
                KeyMapping::Base(BaseKeyMapping::Simple {
                    from: KeyCode::A,
                    to: KeyCode::B,
                }),
                KeyMapping::Base(BaseKeyMapping::Modifier {
                    from: KeyCode::CapsLock,
                    modifier_id: 0x01,
                }),
            ],
        };

        assert_eq!(device_config.identifier.pattern, "*");
        assert_eq!(device_config.mappings.len(), 2);
    }

    #[test]
    fn test_config_root_serialization_round_trip() {
        use alloc::string::String;

        // Create a complete ConfigRoot
        let config = ConfigRoot {
            version: Version::current(),
            devices: alloc::vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: String::from("*"),
                },
                mappings: alloc::vec![KeyMapping::Base(BaseKeyMapping::Simple {
                    from: KeyCode::A,
                    to: KeyCode::B,
                }),],
            },],
            metadata: Metadata {
                compilation_timestamp: 1234567890,
                compiler_version: String::from("1.0.0"),
                source_hash: String::from("abc123"),
            },
        };

        // Serialize
        let bytes = rkyv::to_bytes::<_, 1024>(&config).expect("Serialization failed");

        // Deserialize
        let archived = unsafe { rkyv::archived_root::<ConfigRoot>(&bytes[..]) };

        // Verify round-trip
        assert_eq!(archived.version.major, 1);
        assert_eq!(archived.version.minor, 0);
        assert_eq!(archived.version.patch, 0);
        assert_eq!(archived.devices.len(), 1);
        assert_eq!(archived.metadata.compilation_timestamp, 1234567890);
    }

    #[test]
    fn test_deterministic_serialization() {
        use alloc::string::String;

        // Create the same config twice
        let create_config = || ConfigRoot {
            version: Version::current(),
            devices: alloc::vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: String::from("USB Keyboard"),
                },
                mappings: alloc::vec![
                    KeyMapping::Base(BaseKeyMapping::Simple {
                        from: KeyCode::A,
                        to: KeyCode::B,
                    }),
                    KeyMapping::Conditional {
                        condition: Condition::ModifierActive(0x01),
                        mappings: alloc::vec![BaseKeyMapping::Simple {
                            from: KeyCode::H,
                            to: KeyCode::Left,
                        },],
                    },
                ],
            },],
            metadata: Metadata {
                compilation_timestamp: 9999999999,
                compiler_version: String::from("1.0.0"),
                source_hash: String::from("test_hash_123"),
            },
        };

        let config1 = create_config();
        let config2 = create_config();

        // Serialize both
        let bytes1 = rkyv::to_bytes::<_, 2048>(&config1).expect("Serialization 1 failed");
        let bytes2 = rkyv::to_bytes::<_, 2048>(&config2).expect("Serialization 2 failed");

        // Verify deterministic output (same struct → same bytes)
        assert_eq!(bytes1.len(), bytes2.len());
        assert_eq!(&bytes1[..], &bytes2[..]);
    }

    #[test]
    fn test_complex_config_serialization() {
        use alloc::string::String;

        // Create a complex configuration with all mapping types
        let config = ConfigRoot {
            version: Version::current(),
            devices: alloc::vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: String::from("*"),
                },
                mappings: alloc::vec![
                    // Simple mapping
                    KeyMapping::Base(BaseKeyMapping::Simple {
                        from: KeyCode::A,
                        to: KeyCode::B,
                    }),
                    // Modifier mapping
                    KeyMapping::Base(BaseKeyMapping::Modifier {
                        from: KeyCode::CapsLock,
                        modifier_id: 0x01,
                    }),
                    // Lock mapping
                    KeyMapping::Base(BaseKeyMapping::Lock {
                        from: KeyCode::ScrollLock,
                        lock_id: 0x02,
                    }),
                    // TapHold mapping
                    KeyMapping::Base(BaseKeyMapping::TapHold {
                        from: KeyCode::Space,
                        tap: KeyCode::Space,
                        hold_modifier: 0x00,
                        threshold_ms: 200,
                    }),
                    // ModifiedOutput mapping
                    KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
                        from: KeyCode::A,
                        to: KeyCode::A,
                        shift: true,
                        ctrl: false,
                        alt: false,
                        win: false,
                    }),
                    // Conditional mapping with single condition
                    KeyMapping::Conditional {
                        condition: Condition::ModifierActive(0x01),
                        mappings: alloc::vec![BaseKeyMapping::Simple {
                            from: KeyCode::H,
                            to: KeyCode::Left,
                        },],
                    },
                    // Conditional mapping with AllActive
                    KeyMapping::Conditional {
                        condition: Condition::AllActive(alloc::vec![
                            ConditionItem::ModifierActive(0x01),
                            ConditionItem::LockActive(0x02),
                        ]),
                        mappings: alloc::vec![BaseKeyMapping::Simple {
                            from: KeyCode::J,
                            to: KeyCode::Down,
                        },],
                    },
                    // Conditional mapping with NotActive
                    KeyMapping::Conditional {
                        condition: Condition::NotActive(alloc::vec![
                            ConditionItem::ModifierActive(0x01),
                        ]),
                        mappings: alloc::vec![BaseKeyMapping::Simple {
                            from: KeyCode::K,
                            to: KeyCode::Up,
                        },],
                    },
                ],
            },],
            metadata: Metadata {
                compilation_timestamp: 1234567890,
                compiler_version: String::from("1.0.0"),
                source_hash: String::from("complex_test_hash"),
            },
        };

        // Serialize
        let bytes = rkyv::to_bytes::<_, 4096>(&config).expect("Serialization failed");

        // Deserialize
        let archived = unsafe { rkyv::archived_root::<ConfigRoot>(&bytes[..]) };

        // Verify all mappings are preserved
        assert_eq!(archived.devices.len(), 1);
        assert_eq!(archived.devices[0].mappings.len(), 8);
        assert_eq!(archived.metadata.compilation_timestamp, 1234567890);
    }
}
