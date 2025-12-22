# <img src="icon.png" width="32" height="32" align="center" /> @clevertree/themed-styler

A high-performance styling engine for Relay that supports dynamic theme switching and consistent rendering across Web and Native platforms.

## Vision

As part of the **Relay project vision**, we believe that design systems should be platform-agnostic and runtime-efficient. `@clevertree/themed-styler` allows developers to define themes once in YAML or JSON and have them applied consistently across Web (generating CSS) and React Native (generating StyleSheets).

By offloading the style computation to a shared Rust core, we achieve:
1. **Consistency**: Exact same styling logic on all platforms.
2. **Performance**: Minimal overhead for style resolution, especially critical for complex themes in React Native.
3. **Dynamic Themes**: Real-time theme switching without full re-renders or CSS flashes.

## Usage

### Web (WASM)

Initializes the WASM module and sets up global hooks for the shared style manager.

```typescript
import { initThemedStyler } from '@clevertree/themed-styler';

async function startApp() {
  await initThemedStyler();
  // StyleManager in @clevertree/client-shared will now use the WASM core
}
```

Ensure the WASM is built with:

```bash
wasm-pack build --release --target web --features wasm
```

This produces `pkg/themed_styler.js` and `pkg/themed_styler_bg.wasm` which are copied into the `wasm/` directory during `npm run build`.

### React Native

Provides themed versions of standard components and a runtime for style resolution.

```typescript
import { initThemedStyler, TSDiv, View, Text } from '@clevertree/themed-styler';

// In your App initialization
useEffect(() => {
  initThemedStyler();
}, []);

// Usage in components
const MyComponent = () => (
  <TSDiv tag="div" className="p-4 bg-surface text-primary">
    <Text>Hello Relay!</Text>
  </TSDiv>
);
```

## Key Components

- **`TSDiv`**: A versatile themed component that maps to appropriate native views (View, ScrollView, SafeAreaView, etc.) based on the `tag` and `className`.
- **`styled`**: A utility for creating themed components, similar to styled-components but powered by the Relay Rust core.
- **Themed Primitives**: Re-exports of `View`, `Text`, `TouchableOpacity`, etc., that are aware of the Relay theme system.

## Platform Support

- **Web**: Compiles to WASM. Generates optimized CSS at runtime.
- **Android**: JNI binding to Rust core via TurboModule.
- **iOS**: (Coming soon) C-FFI binding to Rust core via TurboModule.

## Verify Locally

Build and dry-run pack to confirm publish contents:

```bash
npm run build
npm pack --dry-run
```

Node.js >= 18 is required for ESM and tooling.
