// Test 3: Lock Mappings
// Gaming mode toggle that disables Windows key

device_start("*");

// ScrollLock toggles gaming mode (LK_00)
map("ScrollLock", "LK_00");

// Normal mode: Meta key works
when_not_start("LK_00");
map("LMeta", "VK_LMeta");
when_end();

device_end();
