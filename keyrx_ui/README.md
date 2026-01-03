# KeyRx UI

Web-based configuration interface for KeyRx keyboard remapper. Built with React, TypeScript, and Vite.

## Features

- **Visual Configuration Editor**: QMK-style drag-and-drop interface for keyboard remapping
- **Real-time Simulator**: Test key mappings with WASM-powered simulation
- **Profile Management**: Create, edit, and activate profiles
- **Device Management**: Monitor connected devices and configure device-specific mappings
- **Metrics Dashboard**: View keystroke statistics and latency metrics
- **Accessibility**: WCAG 2.2 Level AA compliant with full keyboard navigation support

## Drag-and-Drop Configuration

The visual configuration editor provides an intuitive interface for creating key mappings without writing code.

### How to Use

1. **Navigate to Configuration Page**: Click "Config" in the navigation menu
2. **Select Profile**: Choose a profile from the dropdown in the header
3. **Select Layer**: Choose which layer to edit (base, nav, num, fn)
4. **Drag Keys**: Drag keys from the palette on the left onto the keyboard visualizer
5. **Configure Mapping**: Click a mapped key to open the configuration dialog
6. **Save**: Changes are auto-saved to the daemon

### Keyboard Accessibility

The drag-and-drop interface is fully keyboard-accessible following the Salesforce Lightning pattern:

- **Tab**: Move focus between draggable items and drop zones
- **Space**: Grab a focused item (press again to drop)
- **Arrow Keys**: Navigate between drop zones while dragging
- **Escape**: Cancel the current drag operation
- **Enter**: Open configuration dialog for a mapped key

Screen readers will announce drag state and provide instructions for keyboard users.

### Mapping Types

The configuration editor supports four types of key mappings:

#### 1. Simple Mapping
Map a physical key directly to a virtual key, modifier, or lock.

**Example**: Remap CapsLock to Escape
```typescript
{
  keyCode: "CapsLock",
  type: "simple",
  simple: "VK_ESCAPE"
}
```

#### 2. Tap-Hold (Dual Function)
Different actions for tap vs. hold, with configurable timeout.

**Example**: CapsLock as Escape on tap, Ctrl on hold
```typescript
{
  keyCode: "CapsLock",
  type: "tap_hold",
  tapHold: {
    tap: "VK_ESCAPE",
    hold: "MD_CTRL",
    timeoutMs: 200
  }
}
```

#### 3. Macro Sequence
Execute a sequence of key presses with one key.

**Example**: Type "hello" with a single key
```typescript
{
  keyCode: "F13",
  type: "macro",
  macro: ["VK_H", "VK_E", "VK_L", "VK_L", "VK_O"]
}
```

#### 4. Layer Switch
Switch to a different layer when key is pressed.

**Example**: Access navigation layer while holding key
```typescript
{
  keyCode: "Space",
  type: "layer_switch",
  layer: "nav"
}
```

### Component Documentation

#### DragKeyPalette

Displays a palette of draggable keys organized by category (Virtual Keys, Modifiers, Locks, Layers).

```tsx
import { DragKeyPalette } from '@/components/config/DragKeyPalette';

<DragKeyPalette
  onDragStart={(key) => console.log('Started dragging', key.id)}
  onDragEnd={() => console.log('Drag ended')}
  filterCategory="vk"  // Optional: filter by category
/>
```

**Props:**
- `onDragStart?: (key: AssignableKey) => void` - Callback when drag starts
- `onDragEnd?: () => void` - Callback when drag ends
- `filterCategory?: string` - Filter by category (vk, modifier, lock, layer, macro)
- `className?: string` - Additional CSS classes

#### KeyMappingDialog

Modal dialog for configuring individual key mappings with form validation.

```tsx
import { KeyMappingDialog } from '@/components/config/KeyMappingDialog';

<KeyMappingDialog
  open={isOpen}
  onClose={() => setIsOpen(false)}
  keyCode="CapsLock"
  currentMapping={existingMapping}
  onSave={async (mapping) => {
    await api.saveMapping(mapping);
    setIsOpen(false);
  }}
/>
```

**Props:**
- `open: boolean` - Whether dialog is visible
- `onClose: () => void` - Callback to close dialog
- `keyCode: string` - Physical key being configured
- `currentMapping?: KeyMapping` - Existing mapping (for editing)
- `onSave: (mapping: KeyMapping) => Promise<void>` - Callback with new mapping

#### ProfileHeader

Displays profile context in the configuration page header.

```tsx
import { ProfileHeader } from '@/components/config/ProfileHeader';

<ProfileHeader
  profileName="my-profile"
  isActive={true}
  lastModified={new Date()}
  onProfileChange={(name) => navigate(`/config?profile=${name}`)}
  availableProfiles={['default', 'my-profile', 'gaming']}
/>
```

**Props:**
- `profileName: string` - Name of current profile
- `isActive?: boolean` - Whether profile is active in daemon
- `lastModified?: Date` - Last modification timestamp
- `onProfileChange?: (name: string) => void` - Callback to switch profiles
- `availableProfiles?: string[]` - List of available profiles

#### useDragAndDrop Hook

Custom hook for managing drag-and-drop state and operations.

```tsx
import { useDragAndDrop } from '@/hooks/useDragAndDrop';

function ConfigPage() {
  const { activeDragKey, handleDragStart, handleDragEnd, handleKeyDrop } =
    useDragAndDrop({ profileName: 'default', selectedLayer: 'base' });

  return (
    <DndContext onDragStart={handleDragStart} onDragEnd={handleDragEnd}>
      <DragKeyPalette />
      <KeyboardVisualizer onKeyDrop={handleKeyDrop} />
    </DndContext>
  );
}
```

**Parameters:**
- `profileName: string` - Profile to save mappings to
- `selectedLayer: string` - Active layer for mappings

**Returns:**
- `activeDragKey: AssignableKey | null` - Currently dragged key
- `handleDragStart: (event: DragStartEvent) => void` - Drag start handler
- `handleDragEnd: (event: DragEndEvent) => void` - Drag end handler
- `handleKeyDrop: (keyCode: string, key: AssignableKey) => Promise<void>` - Drop handler
- `isSaving: boolean` - Whether save is in progress

### Troubleshooting

#### Drag-and-drop not working

- **Check browser compatibility**: @dnd-kit requires a modern browser with Pointer Events support
- **Verify daemon connection**: Drag-and-drop saves require WebSocket connection to daemon
- **Check console for errors**: Open browser DevTools and check for error messages

#### Changes not saving

- **Verify WebSocket connection**: Check the connection indicator in the top-right corner
- **Check network requests**: Look for failed PUT /api/config requests in DevTools Network tab
- **Verify profile permissions**: Ensure you have write access to the profile directory

#### Keyboard navigation not working

- **Focus not visible**: Ensure your browser/OS allows focus indicators (check system accessibility settings)
- **Tab order incorrect**: Report as bug - keyboard navigation should follow visual layout
- **Space key not working**: Ensure focus is on a draggable item before pressing Space

## Development

### Prerequisites

- Node.js 18+
- npm 9+

### Setup

```bash
cd keyrx_ui
npm install
```

### Available Scripts

- `npm run dev` - Start development server with HMR
- `npm run build` - Build for production
- `npm run preview` - Preview production build
- `npm test` - Run unit tests
- `npm run test:coverage` - Generate coverage report
- `npm run test:a11y` - Run accessibility tests
- `npm run lint` - Run ESLint
- `npm run type-check` - Run TypeScript compiler check

### Project Structure

```
keyrx_ui/
├── src/
│   ├── api/              # API client functions
│   ├── components/       # Reusable UI components
│   │   └── config/       # Configuration editor components
│   ├── contexts/         # React contexts (API, theme)
│   ├── hooks/            # Custom React hooks
│   ├── pages/            # Top-level page components
│   ├── services/         # Business logic services
│   ├── types/            # TypeScript type definitions
│   ├── utils/            # Utility functions
│   └── App.tsx           # Root component
├── tests/                # Test utilities
├── e2e/                  # Playwright E2E tests
└── public/               # Static assets
```

### Testing

The project uses Vitest for unit tests and Playwright for E2E tests.

**Run unit tests:**
```bash
npm test
```

**Run E2E tests:**
```bash
npm run test:e2e
```

**Run accessibility tests:**
```bash
npm run test:a11y
```

All tests must pass before merging. Coverage threshold is 80% for new code.

## Technology Stack

- **React 18**: UI framework
- **TypeScript 5**: Type safety
- **Vite**: Build tool and dev server
- **@dnd-kit**: Drag-and-drop library
- **@tanstack/react-query**: Data fetching and caching
- **React Router**: Client-side routing
- **Tailwind CSS**: Utility-first styling
- **Vitest**: Unit testing
- **Playwright**: E2E testing
- **axe-core**: Accessibility testing

## React Compiler

The React Compiler is not enabled on this template because of its impact on dev & build performances. To add it, see [this documentation](https://react.dev/learn/react-compiler/installation).

## Expanding the ESLint configuration

If you are developing a production application, we recommend updating the configuration to enable type-aware lint rules:

```js
export default defineConfig([
  globalIgnores(['dist']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      // Other configs...

      // Remove tseslint.configs.recommended and replace with this
      tseslint.configs.recommendedTypeChecked,
      // Alternatively, use this for stricter rules
      tseslint.configs.strictTypeChecked,
      // Optionally, add this for stylistic rules
      tseslint.configs.stylisticTypeChecked,

      // Other configs...
    ],
    languageOptions: {
      parserOptions: {
        project: ['./tsconfig.node.json', './tsconfig.app.json'],
        tsconfigRootDir: import.meta.dirname,
      },
      // other options...
    },
  },
])
```

You can also install [eslint-plugin-react-x](https://github.com/Rel1cx/eslint-react/tree/main/packages/plugins/eslint-plugin-react-x) and [eslint-plugin-react-dom](https://github.com/Rel1cx/eslint-react/tree/main/packages/plugins/eslint-plugin-react-dom) for React-specific lint rules:

```js
// eslint.config.js
import reactX from 'eslint-plugin-react-x'
import reactDom from 'eslint-plugin-react-dom'

export default defineConfig([
  globalIgnores(['dist']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      // Other configs...
      // Enable lint rules for React
      reactX.configs['recommended-typescript'],
      // Enable lint rules for React DOM
      reactDom.configs.recommended,
    ],
    languageOptions: {
      parserOptions: {
        project: ['./tsconfig.node.json', './tsconfig.app.json'],
        tsconfigRootDir: import.meta.dirname,
      },
      // other options...
    },
  },
])
```
