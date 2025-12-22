// Test 5: Complex Multi-Layer Configuration
// Vim-style navigation with multiple layers

device_start("*");

// Layer modifiers
map("CapsLock", "MD_00");  // Primary layer
map("Space", "MD_01");     // Secondary layer

// CapsLock layer: Vim navigation
when_start("MD_00");
map("H", "VK_Left");
map("J", "VK_Down");
map("K", "VK_Up");
map("L", "VK_Right");
map("U", "VK_PageUp");
map("D", "VK_PageDown");
map("Num0", "VK_Home");
map("Num4", "VK_End");
when_end();

// Space layer: Number row
when_start("MD_01");
map("Q", "VK_Num1");
map("W", "VK_Num2");
map("E", "VK_Num3");
map("R", "VK_Num4");
map("T", "VK_Num5");
map("Y", "VK_Num6");
map("U", "VK_Num7");
map("I", "VK_Num8");
map("O", "VK_Num9");
map("P", "VK_Num0");
when_end();

// Both layers: Special functions
when_start(["MD_00", "MD_01"]);
map("Esc", "VK_F12");
map("Enter", "VK_F11");
when_end();

device_end();
