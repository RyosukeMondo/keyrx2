// Test 1: Simple Key Remapping
// A → B, B → C, C → A (cycle)

device_start("*");
map("A", "VK_B");
map("B", "VK_C");
map("C", "VK_A");
device_end();
