# <img src="icon.png" width="32" height="32" align="center" /> @clevertree/themed-styler

A high-performance styling engine for Relay that supports dynamic theme switching and consistent rendering across Web and Native platforms.

## Vision

As part of the **Relay project vision**, we believe that design systems should be platform-agnostic and runtime-efficient. `@clevertree/themed-styler` allows developers to define themes once in YAML or JSON and have them applied consistently across Web (generating CSS) and Android/iOS Native (generating StyleSheets).

By offloading the style computation to a shared Rust core, we achieve:
1. **Consistency**: Exact same styling logic on all platforms.
2. **Performance**: Minimal overhead for style resolution, especially critical for complex themes in Android/iOS Native.
3. **Dynamic Themes**: Real-time theme switching without full re-renders or CSS flashes.

## Installation

### NPM/Yarn

Install from npm:

```bash
npm install @clevertree/themed-styler
# or
yarn add @clevertree/themed-styler
```

The package includes prebuilt WASM files in the `wasm/` directory, so **no additional build step is required during installation**.

### Automatic WASM Inclusion

When you run `npm install`, the WASM files are automatically included:
- `node_modules/@clevertree/themed-styler/wasm/themed_styler.js` (WASM wrapper)
- `node_modules/@clevertree/themed-styler/wasm/themed_styler_bg.wasm` (WASM binary)

These are bundled and published with every version update. **When you upgrade the package, the WASM files are updated automatically—no manual copy needed.**

## Usage

### Web (WASM)

Initialize the WASM module in your app startup:

```typescript
import { initThemedStyler, unifiedBridge, styleManager } from '@clevertree/themed-styler';

async function startApp() {
  // Initialize the WASM styler
  await initThemedStyler();
  
  // Ensure default themes are loaded
  await unifiedBridge.ensureDefaultsLoaded();
  
  // Start auto-sync for style updates
  styleManager.startAutoSync();
}
```

Then use themed components in your precompiled React code:

```typescript
import { TSDiv, Text } from '@clevertree/themed-styler';

const MyComponent = () => (
  <TSDiv tag="div" className="p-4 bg-surface text-primary">
    <Text>Hello Relay!</Text>
  </TSDiv>
);
```

#### WASM Loading & Bundler Configuration

The WASM files are loaded dynamically by `initThemedStyler()`. Here are key points for different bundlers:

**For Vite:**
- WASM files must be served as static assets. Standard Vite configuration handles this automatically.
- If using custom vite.config.ts, ensure `.wasm` files are not excluded from assets:
  
  ```javascript
  export default {
    build: {
      rollupOptions: {
        output: {
          assetFileNames: 'assets/[name].[hash][extname]'
        }
      }
    }
  };
  ```

**For esbuild:**
- Use `--loader:.wasm=file` to emit WASM as external files.

**For webpack:**
- Ensure `file-loader` or `asset/resource` is configured for `.wasm` files.

#### Avoiding 404 Errors on WASM Files

If you see console errors like "Failed to load WASM" or network 404 errors for `themed_styler_bg.wasm`:

**Step 1: Verify WASM files exist**
```bash
ls node_modules/@clevertree/themed-styler/wasm/
# Should output:
#   themed_styler.js
#   themed_styler_bg.wasm
```

**Step 2: Clear and reinstall if missing**
```bash
rm -rf node_modules
npm install
```

**Step 3: Rebuild your bundle**
```bash
npm run build
```

**Step 4: Check bundler output**
Verify that your bundle includes the WASM files in the dist directory:
```bash
ls dist/  # or dist/assets/ depending on your bundler
# Should see themed_styler*.wasm files
```

**Step 5: For custom HTTP servers**
If using a non-Vite HTTP server, ensure it serves WASM with correct MIME type:

```javascript
// Express.js example
app.use('/node_modules/@clevertree/themed-styler/wasm', express.static(
  path.join(__dirname, 'node_modules/@clevertree/themed-styler/wasm'),
  {
    setHeaders: (res, path) => {
      if (path.endsWith('.wasm')) {
        res.setHeader('Content-Type', 'application/wasm');
      }
    }
  }
));
```

**Step 6: Check browser DevTools**
- Open DevTools → Network tab
- Look for `themed_styler_bg.wasm` request
- Check the response status and content-type header
- If 404, verify your server is serving from correct path

**Step 7: After upgrading the package**
- Clear browser cache (Ctrl+Shift+Delete / Cmd+Shift+Delete)
- Delete old dist directory: `rm -rf dist`
- Rebuild: `npm run build`
- The WASM files update automatically with npm—no manual copy needed

### Android/iOS Native (Android & iOS)

Initialize the native themed styler binding in your app:

```typescript
import { initThemedStyler, ensureDefaultsLoaded } from '@clevertree/themed-styler/android';
import { TSDiv, View, Text } from '@clevertree/themed-styler/android';

async function startApp() {
  // Initializes native Rust FFI binding via TurboModule
  // Android: Uses JNI to call Rust native functions
  // iOS: (Coming soon) Uses C-FFI to call Rust native functions
  await initThemedStyler();
  
  // Load default theme definitions from YAML
  await ensureDefaultsLoaded();
}

// In your component
const MyComponent = () => (
  <TSDiv tag="view" className="p-4 bg-surface text-primary">
    <Text>Hello Relay!</Text>
  </TSDiv>
);
```

#### Android Native Setup

For Android, ensure your app's native code has the Relay JNI module registered:

1. Verify TurboModule is linked in your Gradle build
2. Ensure the native library exports `ThemedStyler` module for JSI access
3. If you see "Native binding not available" in logs, the fallback stub is active (styles will be empty until native module is properly linked)

#### Platform-Specific Exports

Use platform-specific entry points to avoid bundling unnecessary code:

```typescript
// Web
import { initThemedStyler, TSDiv } from '@clevertree/themed-styler';

// Android/Android/iOS Native
import { initThemedStyler, TSDiv } from '@clevertree/themed-styler/android';
```

## Key Components

- **`TSDiv`**: A versatile themed component that maps to appropriate native views (View, ScrollView, SafeAreaView, etc.) based on the `tag` and `className`.
- **`styled`**: A utility for creating themed components, similar to styled-components but powered by the Relay Rust core.
- **`styleManager`** (Web only): Manages CSS rendering, auto-sync, and DOM updates.
- **`unifiedBridge`**: Provides unified theme registration and CSS generation API across platforms.
- **Themed Primitives**: Re-exports of `View`, `Text`, `TouchableOpacity`, `SafeAreaView`, etc., with theme awareness.

## Platform Support

- **Web**: Compiles to WASM. Generates optimized CSS at runtime.
- **Android**: JNI binding to Rust core via TurboModule.
- **iOS**: (Coming soon) C-FFI binding to Rust core via TurboModule.

## Development

### Build from Source

If modifying the Rust core or TypeScript sources:

```bash
# Install dependencies
npm install

# Compile TypeScript and build WASM
npm run build

# The build script automatically:
# 1. Runs `wasm-pack build --release --target web --features wasm`
# 2. Copies WASM files to `wasm/` directory
# 3. Compiles TypeScript to `dist/web/`, `dist/android/`, and `dist/shared/`
```

### Testing

Run the test suite:

```bash
# Web tests (Cypress)
cd tests/web
npm install
npm run test:e2e
```

## Requirements

- **Node.js**: >= 18 (required for ESM and tooling)
- **React**: >= 18.0.0
- **Android/iOS Native**: >= 0.70 (for Android/iOS platforms)

## Troubleshooting

### WASM Not Loading

**Symptom**: "Failed to initialize WASM styler" or console shows WASM module undefined.

**Solution**:
1. Verify files exist: `ls node_modules/@clevertree/themed-styler/wasm/`
2. Check browser network tab for 404 on `themed_styler_bg.wasm` (see **Avoiding 404 Errors** section above)
3. Ensure bundler outputs WASM as static assets
4. Clear cache and rebuild: `rm -rf dist && npm run build`
5. If upgrading from older version: `rm -rf node_modules && npm install`

### Styles Not Applying (Web)

**Symptom**: Components render but styles don't apply.

**Solution**:
1. Ensure `initThemedStyler()` is called before rendering components
2. Ensure `styleManager.startAutoSync()` is called to enable live style updates
3. Check that `unifiedBridge.ensureDefaultsLoaded()` completes successfully
4. Verify themes are registered via `registerTheme(name, definitions)`

### Styles Not Applying (Android)

**Symptom**: Components render but styles are empty.

**Solution**:
1. Check that native TurboModule is linked in Gradle
2. Verify `initThemedStyler()` completes without errors
3. If you see "Native binding not available", ensure the Relay native library is properly compiled and linked
4. Check logcat for JNI errors: `adb logcat | grep ThemedStyler`

## Contributing

Contributions are welcome! Please ensure:

1. WASM changes are built with `npm run build`
2. TypeScript changes compile cleanly: `tsc --noEmit`
3. Tests pass: `npm run test:e2e` (web)

## License

See LICENSE file in the repository.
