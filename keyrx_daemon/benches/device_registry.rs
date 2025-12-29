use criterion::{black_box, criterion_group, criterion_main, Criterion};
use keyrx_daemon::config::device_registry::{DeviceEntry, DeviceRegistry, DeviceScope};
use tempfile::TempDir;

/// Benchmark device registry save operation (atomic write).
/// Target: <50ms for atomic write
fn benchmark_device_registry_save(c: &mut Criterion) {
    // Setup: Create temp directory with populated registry
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let registry_path = temp_dir.path().join("devices.json");

    let mut registry = DeviceRegistry::new(registry_path.clone());

    // Add 10 devices to simulate realistic workload
    for i in 0..10 {
        let entry = DeviceEntry {
            id: format!("device_{}", i),
            name: format!("Keyboard {}", i),
            serial: Some(format!("SN{:04}", i)),
            scope: DeviceScope::DeviceSpecific,
            layout: Some("ansi_104".to_string()),
            last_seen: 1700000000 + i,
        };

        registry.register(entry).expect("Failed to register device");
    }

    c.bench_function("device_registry_save_10_devices", |b| {
        b.iter(|| {
            // Benchmark the atomic write operation
            let result = registry.save();
            assert!(result.is_ok(), "Registry save failed");
            black_box(result);
        });
    });
}

/// Benchmark device registry load operation.
fn benchmark_device_registry_load(c: &mut Criterion) {
    // Setup: Create and persist a registry
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let registry_path = temp_dir.path().join("devices.json");

    let mut registry = DeviceRegistry::new(registry_path.clone());

    // Add 50 devices for more realistic load testing
    for i in 0..50 {
        let entry = DeviceEntry {
            id: format!("device_{}", i),
            name: format!("Keyboard {}", i),
            serial: Some(format!("SN{:04}", i)),
            scope: if i % 2 == 0 {
                DeviceScope::DeviceSpecific
            } else {
                DeviceScope::Global
            },
            layout: Some("ansi_104".to_string()),
            last_seen: 1700000000 + i,
        };

        registry.register(entry).expect("Failed to register device");
    }

    // Save to disk
    registry.save().expect("Failed to save registry");

    c.bench_function("device_registry_load_50_devices", |b| {
        b.iter(|| {
            // Benchmark loading from disk
            let loaded = DeviceRegistry::load(black_box(&registry_path));
            assert!(loaded.is_ok(), "Registry load failed");
            black_box(loaded);
        });
    });
}

/// Benchmark device registry lookup operations.
fn benchmark_device_registry_get(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let registry_path = temp_dir.path().join("devices.json");

    let mut registry = DeviceRegistry::new(registry_path);

    // Add 100 devices
    for i in 0..100 {
        let entry = DeviceEntry {
            id: format!("device_{}", i),
            name: format!("Keyboard {}", i),
            serial: Some(format!("SN{:04}", i)),
            scope: DeviceScope::DeviceSpecific,
            layout: Some("ansi_104".to_string()),
            last_seen: 1700000000 + i,
        };

        registry.register(entry).expect("Failed to register device");
    }

    c.bench_function("device_registry_get_lookup", |b| {
        b.iter(|| {
            // Benchmark device lookup (HashMap access)
            let device = registry.get(black_box("device_50"));
            black_box(device);
        });
    });
}

/// Benchmark device registry listing operation.
fn benchmark_device_registry_list(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let registry_path = temp_dir.path().join("devices.json");

    let mut registry = DeviceRegistry::new(registry_path);

    // Add 100 devices
    for i in 0..100 {
        let entry = DeviceEntry {
            id: format!("device_{}", i),
            name: format!("Keyboard {}", i),
            serial: Some(format!("SN{:04}", i)),
            scope: DeviceScope::DeviceSpecific,
            layout: Some("ansi_104".to_string()),
            last_seen: 1700000000 + i,
        };

        registry.register(entry).expect("Failed to register device");
    }

    c.bench_function("device_registry_list_100_devices", |b| {
        b.iter(|| {
            let devices = registry.list();
            black_box(devices);
        });
    });
}

/// Benchmark device registry rename operation.
fn benchmark_device_registry_rename(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let registry_path = temp_dir.path().join("devices.json");

    let mut registry = DeviceRegistry::new(registry_path);

    let entry = DeviceEntry {
        id: "device_test".to_string(),
        name: "Original Name".to_string(),
        serial: Some("SN0001".to_string()),
        scope: DeviceScope::DeviceSpecific,
        layout: Some("ansi_104".to_string()),
        last_seen: 1700000000,
    };

    registry.register(entry).expect("Failed to register device");

    c.bench_function("device_registry_rename", |b| {
        let mut counter = 0;
        b.iter(|| {
            // Benchmark rename with validation
            let new_name = format!("Keyboard {}", counter);
            let result = registry.rename(black_box("device_test"), black_box(&new_name));
            assert!(result.is_ok(), "Rename failed");
            black_box(result);
            counter += 1;
        });
    });
}

criterion_group!(
    benches,
    benchmark_device_registry_save,
    benchmark_device_registry_load,
    benchmark_device_registry_get,
    benchmark_device_registry_list,
    benchmark_device_registry_rename
);
criterion_main!(benches);
