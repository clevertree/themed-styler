# themed-styler Copilot Instructions

## Project Overview
Rust WASM/native library for CSS-like styling with theming support.
Provides both web (WASM) and Android (JNI) bindings.

## Architecture

### Key Components
1. **Core Styling Engine** - Parses CSS-like rules, applies themes, handles unit conversions
2. **WASM Bindings** - Web API via `wasm-bindgen` for browser use
3. **Android JNI** - Native bindings for Android with density-aware unit conversion
4. **Theme System** - YAML-based themes with light/dark mode support

### Build System
- Cargo crate: `name = "themed-styler"`
- Features: `wasm` (for web builds)
- Android: cargo-ndk for cross-compilation to all architectures
- Scripts: `scripts/build-android.sh` builds native libs, `android/` subproject packages AAR

## Implementation Details

### State Management
```rust
pub struct State {
    pub display_density: f32,      // Android display density (e.g., 2.0 for xhdpi)
    pub scaled_density: f32,       // Android scaled density for text
    // ...
}
```

### Android JNI Exports
- `android_styles_for(css, themes_json, density)` - Returns Android styles JSON with dp→px conversion
- `version()` - Returns crate version string

### WASM Exports
- `render_css(css, themes_json)` - Returns web-compatible CSS

## Android Test App Architecture

### Test App Location
`/home/ari/dev/themed-styler/tests/android/` - Primary Android test application for themed-styler

### Key Test Components
1. **JSCManager** - Extends base `com.clevertree.jscbridge.JSCManager`
   - Overrides `setupModules(context: JSContext)` to install test-specific modules
   - Manages theme JSON loading with light/dark variants
   - Installs native transpiler and Android bridge modules
   
2. **HookRenderer** - Renders JSX hooks to native Android views
   - Creates native ViewGroup hierarchy
   - Manages React-like component lifecycle
   - Triggers events and state updates
   
3. **AndroidRenderer** - Converts style objects to native Android properties
   - Applies themes from CSS classes (.bg-primary, .text-themed, etc.)
   - Converts density-aware pixel values
   - Calls ThemedStylerModule.nativeGetAndroidStyles()

### Theme Definition
```kotlin
// Themes include selectors with class-based styling:
".bg-primary": { "backgroundColor": "#3b82f6" },
".text-themed": { "color": "#1f2937" }
```

**Selector Format (CRITICAL)**: Class selectors MUST include the dot prefix (`.bg-primary`, not `bg-primary`)
- Android renderer adds dot prefix to class names before calling nativeGetAndroidStyles()
- Hook-transpiler's StyleCache also converts `bg-primary` → `.bg-primary`
- Themed-styler Rust code expects CSS selector format

**Known Issue**: Class selectors in themed-styler Rust code aren't being matched properly even with correct selector format (`.bg-primary`). The Rust matcher falls back to default theme colors. This appears to be a bug in the CSS selector matching algorithm in the themed-styler Rust implementation.

### Building Test App
```bash
cd /home/ari/dev/themed-styler/tests/android
./gradlew clean assembleDebug
adb install -r app/build/outputs/apk/debug/app-debug.apk
```

### Test Verification
```bash
# Check for successful renders
adb logcat com.relay.test:I -e "RENDER_SUCCESS|RENDER_VERIFIED"

# Expected output:
# [RENDER_SUCCESS] Renderer: Android, Views: 29, Bridge calls: 75
# [RENDER_VERIFIED] Correct renderer used: Android
```

## Android Live Reload System

### Architecture
The test app supports live reloading of JSX/TSX files from a local dev server.
- **Detection**: On initialization, `HookRenderer` checks if the app is debuggable and pings `http://127.0.0.1:8081/status`.
- **WebSocket**: Listens for `reload` messages on `ws://127.0.0.1:8081`.
- **Interception**: Intercepts asset reads and redirects to the dev server.

### Usage (CRITICAL)
To use live reload, you MUST set up an ADB reverse proxy:
```bash
adb reverse tcp:8081 tcp:8081
```
Then start the dev server in `tests/android/scripts/start-dev.sh` (which runs `dev-server.cjs`).

## Code Architecture: jscbridge Integration

### JSCManager Consolidation
The test app's JSCManager extends `com.clevertree.jscbridge.JSCManager` base class to eliminate duplication:
- **Inherited from base**: CommonJS module system, console shim, JSC engine lifecycle
- **Test-specific override**: `setupModules()` to register theme and transpiler modules
- **Removed duplication**: ~400 lines of boilerplate code now come from jscbridge base class

### Building After jscbridge Changes
If you modify jscbridge, the test app must be rebuilt with dependency refresh:
```bash
# After changing jscbridge:
cd /home/ari/dev/jscbridge && ./gradlew clean publishToMavenLocal
cd /home/ari/dev/themed-styler/tests/android
./gradlew --refresh-dependencies clean assembleDebug  # Note: --refresh-dependencies is CRITICAL
```

**Common Error**: "This type is final, so it cannot be inherited from"
- **Cause**: jscbridge JSCManager not marked `open` or methods not marked `open`
- **Solution**: Ensure all methods in jscbridge JSCManager that subclasses override have `open` keyword
- **Verification**: Check `/home/ari/dev/jscbridge/src/main/kotlin/com/clevertree/jscbridge/JSCManager.kt`

## Dependency Management (CRITICAL)

### Publishing to mavenLocal
The Android test app in hook-transpiler depends on `com.clevertree:themed-styler-android:1.0.0` from mavenLocal.

**After ANY changes to themed-styler:**
1. **Rebuild native libraries:**
   ```bash
   cd /home/ari/dev/themed-styler
   bash scripts/build-android.sh  # Builds all architectures
   ```

2. **Publish AAR to mavenLocal:**
   ```bash
   cd android
   ./gradlew clean publishToMavenLocal
   ```

3. **Verify published version:**
   ```bash
   unzip -p ~/.m2/repository/com/clevertree/themed-styler-android/1.0.0/themed-styler-android-1.0.0.aar \
     jni/arm64-v8a/libthemed_styler.so | strings | grep "1\.2\.[0-9]" | head -2
   ```

4. **Update dependent projects:**
   ```bash
   cd /home/ari/dev/hook-transpiler/tests/android
   ./gradlew --refresh-dependencies clean assembleDebug
   ```

### Version Verification
Check that APK contains the correct version:
```bash
strings app/build/outputs/apk/debug/app-debug.apk | grep "1\.2\.[0-9]" | head -5
```

**NEVER** manually copy `libthemed_styler.so` files to hook-transpiler or other consumer projects!
They should always come from the published AAR via Gradle dependency resolution.

## Build & Test Playbook

### Web (WASM)
```bash
npm run build  # Calls wasm-pack build --release --target web --features wasm
```

### Android Native
```bash
bash scripts/build-android.sh  # Builds for all architectures
cd android && ./gradlew publishToMavenLocal  # Publishes AAR
```

### Testing
- Rust unit tests: `cargo test`
- Android integration: Use hook-transpiler test app
- Web integration: Use themed-styler web test or hook-transpiler web tests

## Common Issues

### Old version in APK despite rebuild
- Cause: Manually copied libs in consumer project's jniLibs folders
- Solution: Remove any `libthemed_styler.so` from consumer projects, rely on mavenLocal dependency
- Check: `find /path/to/consumer -path "*/jniLibs/*" -name "libthemed_styler.so"`

### AAR not found by Gradle
- Verify mavenLocal has the AAR: `ls ~/.m2/repository/com/clevertree/themed-styler-android/1.0.0/`
- Ensure consumer's `build.gradle` has `mavenLocal()` in repositories
- Run with `--refresh-dependencies` flag

### Version mismatch between Cargo.toml and native lib
- Rebuild native libs: `bash scripts/build-android.sh`
- Republish: `cd android && ./gradlew publishToMavenLocal`
- Version is embedded in binary via Rust's `env!("CARGO_PKG_VERSION")`
