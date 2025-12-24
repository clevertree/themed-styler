/**
 * Android/QuickJS entry point for themed-styler
 * Provides native-backed styling without WASM, using Rust FFI via JNI
 */

declare global {
    var __themedStylerNative: any
    var __themedStylerRenderCss: ((usage: any, themes: any) => string) | undefined
    var __themedStylerGetRn: ((themeName: string) => any) | undefined
    var __themedStylerGetVersion: (() => string) | undefined
}

// Android-specific exports only - exclude any modules with dependencies
// Do NOT export themedPrimitives, themedRuntime, styled, TSDiv, etc.
// These have circular dependencies and pull in YAML via themedStylerBridge
// Android clients should ONLY use the native-backed init function below

/**
 * Initialize ThemedStyler for Android/QuickJS runtime
 * Binds native Rust functions for style rendering without WASM
 */
export async function initAndroidThemedStyler(opts?: {
    onRenderCss?: (usage: any, themes: any) => string
    onGetRnStyles?: (themeName: string) => any
}): Promise<void> {
    const g = globalThis as any

    // Check if already initialized
    if (g.__themedStylerRenderCss && g.__themedStylerGetRn) {
        console.debug('[themed-styler-android] Already initialized')
        return
    }

    // Verify native binding is available
    if (!g.__themedStylerNative) {
        console.warn('[themed-styler-android] Native binding not available - using stubs')
        // Provide stub implementations that fall back gracefully
        g.__themedStylerRenderCss = opts?.onRenderCss || ((usage: any, themes: any) => '')
        g.__themedStylerGetRn = opts?.onGetRnStyles || ((themeName: string) => ({}))
        g.__themedStylerGetVersion = () => 'native-stub'
        return
    }

    // Bind native functions
    g.__themedStylerRenderCss = (usage: any, themes: any) => {
        try {
            // Call native Rust function via JNI
            const result = g.__themedStylerNative.renderCss(JSON.stringify(usage), JSON.stringify(themes))
            return result || ''
        } catch (e) {
            console.error('[themed-styler-android] renderCss failed:', e)
            return opts?.onRenderCss ? opts.onRenderCss(usage, themes) : ''
        }
    }

    g.__themedStylerGetRn = (themeName: string) => {
        try {
            // Call native Rust function via JNI
            const result = g.__themedStylerNative.getRnStyles(themeName)
            return JSON.parse(result || '{}')
        } catch (e) {
            console.error('[themed-styler-android] getRnStyles failed:', e)
            return opts?.onGetRnStyles ? opts.onGetRnStyles(themeName) : {}
        }
    }

    g.__themedStylerGetVersion = () => {
        try {
            return g.__themedStylerNative.getVersion?.() || 'native'
        } catch (e) {
            return 'native-error'
        }
    }

    console.log('[themed-styler-android] Initialized with native binding:', g.__themedStylerGetVersion?.())
}

/**
 * Create a themed style object for Android components
 * Converts theme definitions to native-compatible style objects
 */
export function createAndroidTheme(definitions: Record<string, any>) {
    const theme = { ...definitions }

    return {
        getStyle: (name: string) => theme[name] || {},
        getColor: (name: string) => theme[name],
        getRenderCss: () => globalThis.__themedStylerRenderCss?.(theme, {}),
    }
}

/**
 * Apply themed styles to Android view properties
 * Maps style objects to native view props
 */
export function applyAndroidThemeStyle(
    baseProps: any,
    themeName: string,
    fallbackStyle?: any,
): any {
    const g = globalThis as any
    const themeStyles = g?.__themedStylerGetRn?.(themeName) || {}

    return {
        ...baseProps,
        ...(themeStyles || fallbackStyle || {}),
    }
}
