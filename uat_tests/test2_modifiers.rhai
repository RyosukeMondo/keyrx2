// Test 2: Modifier Mappings
// CapsLock as layer modifier for Vim-style navigation

device_start("*");

// CapsLock becomes a layer modifier (MD_00)
map("CapsLock", "MD_00");

// When CapsLock held: HJKL → Arrow keys
when_start("MD_00");
map("H", "VK_Left");
map("J", "VK_Down");
map("K", "VK_Up");
map("L", "VK_Right");
when_end();

device_end();
