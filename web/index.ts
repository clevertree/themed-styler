export * from './themedPrimitives'
export * from './themedRuntime'
export { TSDiv, ThemedElement, resolveThemedStyle } from './components/TSDiv'
export { styled } from './utils/themedStyled'

export { default as themedStylerBridge, ensureDefaultsLoaded } from '../shared/themedStylerBridge'
export { default as styleManager } from './styleManager'
export { default as unifiedBridge } from '../shared/unifiedBridge'

// WASM-based styler for web - RN uses native JSI binding instead
export async function initWasmThemedStyler(): Promise<void> {
  const g = globalThis as any

  // Check if already initialized
  if (g.__themedStylerRenderCss && g.__themedStylerGetRn) {
    console.debug('[themed-styler] Already initialized')
    return
  }

  // In Android/iOS Native environments, skip WASM loading entirely
  // Native apps should use the JSI module or runtime checks instead
  const isNativeApp = typeof navigator !== 'undefined' && (navigator.product === 'AndroidNative' || navigator.product === 'iOSNative')
  const isNode = typeof process !== 'undefined' && process.versions && process.versions.node
  if (isNativeApp || (typeof window === 'undefined' && !isNode)) {
    console.debug('[themed-styler] Skipping WASM init in non-web/non-node environment')
    return
  }

  // Web-only WASM loading
  try {
    // @ts-ignore
    const { default: init, render_css_for_web, get_rn_styles, get_version } = await import('../wasm/themed_styler.js')

    // Call init without parameters to use default import.meta.url resolution
    // The WASM module will automatically find themed_styler_bg.wasm
    // relative to its own location using import.meta.url
    await init()

    const version = get_version ? get_version() : 'wasm'
    console.log('[themed-styler] WASM initialized:', version)

    // Expose styling functions to the bridge
    g.__themedStylerRenderCss = (usage: any, themes: any) => {
      const state = {
        ...themes,
        used_tags: usage.tags,
        used_classes: usage.classes,
        used_tag_classes: usage.tagClasses,
      }
      return render_css_for_web(JSON.stringify(state))
    }

    g.__themedStylerGetRn = (selector: string, classes: string[], themes: any) => {
      const state = { ...themes }
      return JSON.parse(get_rn_styles(JSON.stringify(state), selector, JSON.stringify(classes)))
    }

    g.__themedStylerVersion = version
  } catch (e) {
    console.warn('[themed-styler] Failed to initialize WASM styler', e)
  }
}

export async function initThemedStyler(): Promise<void> {
  return initWasmThemedStyler()
}
