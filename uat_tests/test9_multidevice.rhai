// Test 9: Multiple Device Configuration
// Different mappings for different keyboards

// Laptop keyboard - missing right Ctrl
device_start("AT Translated Set 2 keyboard");
map("RAlt", "VK_RCtrl");  // Right Alt → Right Ctrl
map("CapsLock", "VK_Esc");  // CapsLock → Esc
device_end();

// External mechanical keyboard
device_start("Keychron K2");
// Minimal changes for good keyboard
map("CapsLock", "VK_LCtrl");  // CapsLock → Ctrl
device_end();

// Fallback for all other devices
device_start("*");
map("CapsLock", "VK_Esc");
map("Esc", "VK_CapsLock");
device_end();
