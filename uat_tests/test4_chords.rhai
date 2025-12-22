// Test 4: Modified Output (Chords)
// Number row produces shifted/modified output

device_start("*");

// Press 1 → Shift+1 (!)
map("Num1", with_shift("VK_Num1"));

// Press 2 → Ctrl+C (copy)
map("Num2", with_ctrl("VK_C"));

// Press 3 → Ctrl+Shift+Esc (Task Manager)
map("Num3", with_mods("VK_Esc", true, true, false, false));

// Press 4 → Alt+F4 (close window)
map("Num4", with_alt("VK_F4"));

device_end();
