# Themed-Styler Android Test App

Test application for the `@clevertree/themed-styler` library demonstrating theme management, dynamic styling, and responsive layouts on Android.

## Features

### 1. **Multi-Theme Support**
The app supports dark and light themes defined in `app-theme.yaml`:
- **Light Theme** - Clean, bright colors suitable for daytime use
- **Dark Theme** - Dark backgrounds with light text for low-light environments

### 2. **Dynamic Theme Switching**
Users can switch between themes at runtime:
- The UI re-renders with the new theme's colors and styles
- All components automatically update their appearance
- Switching is seamless without disrupting the app state

### 3. **Responsive Breakpoints**
The theme system includes responsive breakpoints for adaptive layouts:
- `sm`: 640px (mobile)
- `md`: 768px (tablet)
- `lg`: 1024px (desktop)
- `xl`: 1280px (wide desktop)
- `2xl`: 1536px (ultra-wide)

### 4. **Color Palette System**
Each theme defines a comprehensive color palette:
- **Primary** - Main accent color
- **Secondary** - Alternative accent
- **Success** - Positive actions/states
- **Warning** - Cautionary information
- **Danger** - Error or destructive actions
- **Background** - Main background
- **Surface** - Component backgrounds
- **Text** - Main text color
- **Text Muted** - Secondary/disabled text
- **Border** - Border and divider colors

## File Structure

```
tests/android/
├── app/
│   ├── src/
│   │   ├── main/
│   │   │   ├── assets/
│   │   │   │   ├── app-theme.yaml          # Theme definitions (light & dark)
│   │   │   │   ├── test-hook.jsx           # Main demo hook with theme switching
│   │   │   │   ├── breakpoint-demo.jsx     # Responsive breakpoints demo
│   │   │   │   ├── rn-parity.jsx           # React Native parity tests
│   │   │   │   └── string-keywords-test.jsx # String handling tests
│   │   │   ├── java/com/relay/test/
│   │   │   │   ├── MainActivity.kt         # App entry point with ViewPager
│   │   │   │   ├── HookFragmentAdapter.kt  # Fragment adapter for tabs
│   │   │   │   ├── LocalHookFragment.kt    # Hook rendering fragment
│   │   │   │   ├── JSCManager.kt           # JS runtime management
│   │   │   │   ├── AndroidRenderer.kt      # Native view creation
│   │   │   │   └── AndroidThemeManager.kt  # Theme management (optional)
│   │   │   ├── res/
│   │   │   │   ├── layout/
│   │   │   │   │   ├── activity_main.xml
│   │   │   │   │   └── fragment_local_hook.xml
│   │   │   │   └── values/
│   │   │   │       └── strings.xml
│   │   │   └── AndroidManifest.xml
│   │   └── androidTest/
│   │       └── java/com/relay/test/
│   │           ├── ActInstrumentationTest.kt
│   │           ├── TranspilerJNITest.kt
│   │           └── JSCEs6Test.kt
│   ├── build.gradle
│   └── proguard-rules.pro
├── gradle.properties
└── README.md (this file)
```

## Usage

### Building the App

```bash
# From /home/ari/dev/themed-styler/tests/android/
./gradlew clean assembleDebug
```

### Installing on Device

```bash
adb install -r app/build/outputs/apk/debug/app-debug.apk
```

### Running the App

```bash
adb shell am start -n com.relay.test/.MainActivity
```

## Test Hooks

### 1. **test-hook.jsx** (Default)
The main demo hook demonstrating:
- Stateful theme switching (light/dark)
- Conditional styling based on theme state
- Responsive color palettes
- Flex layout with theme colors
- UI re-rendering on theme change

**How to use:**
```jsx
const [currentTheme, setCurrentTheme] = useState('light');

// Conditionally apply theme-specific classes
<div className={`p-4 ${theme.bgClass} ${theme.textClass}`}>
  {/* Content */}
</div>
```

### 2. **breakpoint-demo.jsx** (Tab 2)
Demonstrates responsive design using breakpoints:
- Shows all 5 breakpoints with pixel widths
- Color-coded visual representation
- Responsive layout that adapts to screen size

### 3. **Other Hooks**
- `rn-parity.jsx` - Tests React Native compatibility
- `string-keywords-test.jsx` - Tests string handling and keywords

## Tab Navigation

The app uses ViewPager2 with TabLayout for multi-hook navigation:
- **Tab 1: "Test Hook"** - Main themed demo with theme switching
- **Tab 2: "Breakpoints"** - Responsive breakpoints demonstration

Swipe horizontally to switch between tabs.

## Theme Configuration (app-theme.yaml)

The `app-theme.yaml` file defines two complete themes:

```yaml
themes:
  light:
    colors:
      primary: "#3b82f6"
      background: "#ffffff"
      text: "#1f2937"
      # ... more colors
    selectors:
      ".bg-primary":
        background-color: "#3b82f6"
      # ... CSS-like selectors for styling
  
  dark:
    colors:
      primary: "#60a5fa"
      background: "#0f172a"
      text: "#f1f5f9"
      # ... more colors
    selectors:
      # ... dark theme selectors
```

### Adding New Colors to Themes

To add a new color to both themes:

1. Open `app-theme.yaml`
2. Add the color to both `light.colors` and `dark.colors`
3. Add corresponding selectors for using the color (e.g., `.bg-newColor`, `.text-newColor`)
4. Update `test-hook.jsx` to use the new color class

Example:
```yaml
light:
  colors:
    accent: "#ff6b6b"
  selectors:
    ".bg-accent":
      background-color: "#ff6b6b"

dark:
  colors:
    accent: "#ff8787"
  selectors:
    ".bg-accent":
      background-color: "#ff8787"
```

## Instrumentation Tests

The app includes comprehensive tests in `androidTest`:

```bash
# Run all tests
./gradlew connectedAndroidTest

# Run specific test class
./gradlew connectedAndroidTest -Pandroid.testInstrumentationRunnerArguments.class=com.relay.test.ActInstrumentationTest
```

### Test Classes
- **ActInstrumentationTest** - Tests ACT renderer with themed hooks
- **TranspilerJNITest** - Tests JSX transpilation and asset loading
- **JSCEs6Test** - Tests ES6 features in JavaScript runtime

## Architecture

```
┌─────────────────────────────────────────┐
│ User Interface (Android Views)           │
├─────────────────────────────────────────┤
│ HookRenderer (Orchestrates execution)    │
├──────────────────┬──────────────────────┤
│                  │                      │
│ Theme System     │ JSX Transpiler       │
│ - light/dark     │ - Parses JSX         │
│ - Colors         │ - Compiles to JS     │
│ - Selectors      │                      │
│                  │                      │
├─────────────────┴──────────────────────┤
│ JavaScriptCore Runtime                   │
│ - Executes transpiled code               │
│ - Manages state (useState, etc.)         │
│ - Bridges to native Android              │
└─────────────────────────────────────────┘
```

## Dependencies

### Android Libraries
- `androidx.appcompat:appcompat:1.6.1`
- `com.google.android.material:material:1.9.0`
- `androidx.viewpager2:viewpager2:1.0.0`
- `androidx.constraintlayout:constraintlayout:2.1.4`

### Native Libraries
- `jscbridge` - JavaScriptCore bridge
- `hook-transpiler-android` - JSX transpiler
- `themed-styler-android` - Styling engine

## Troubleshooting

### App crashes on startup
- Check that all native libraries are properly loaded
- Verify `MainActivity.kt` has correct library loading order
- Check logcat for JNI errors: `adb logcat | grep "HookRenderer\|JNI"`

### Theme changes not reflected
- Ensure `useState` is imported from React
- Verify state updates trigger re-renders
- Check that className bindings are correct

### Transpilation errors
- Check `breakpoint-demo.jsx` for syntax errors (no stray JSX)
- Ensure component names match exported functions
- Verify proper JSX syntax with closing tags

## Further Development

### To add a new test hook:

1. Create `src/main/assets/my-hook.jsx`:
```jsx
import React, { useState } from 'react';

export default function MyHook() {
  const [state, setState] = useState(initialValue);
  
  return (
    <div className="p-4">
      {/* Hook content */}
    </div>
  );
}
```

2. Add to `HookFragmentAdapter.kt`:
```kotlin
override fun getItemCount(): Int = 3  // Increase count

override fun createFragment(position: Int): Fragment = when(position) {
    0 -> LocalHookFragment.newInstance("test-hook.jsx")
    1 -> LocalHookFragment.newInstance("breakpoint-demo.jsx")
    2 -> LocalHookFragment.newInstance("my-hook.jsx")  // New hook
    else -> LocalHookFragment.newInstance("test-hook.jsx")
}
```

3. Add tab label in `MainActivity.kt`:
```kotlin
tab.text = when (position) {
    0 -> "Test Hook"
    1 -> "Breakpoints"
    2 -> "My Hook"  // New label
    else -> "Tab"
}
```

## Documentation References

- [Relay Hook Transpiler](https://github.com/clevertree/hook-transpiler)
- [@clevertree/themed-styler](https://github.com/clevertree/themed-styler)
- [Android App Bundle Documentation](https://developer.android.com/guide/app-bundle)

## License

MIT OR Apache-2.0
