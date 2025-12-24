export * from './nativeThemedStyler.native'
export * from './themedPrimitives.native'
export * from './themedRuntime'
export { TSDiv, ThemedElement, resolveThemedStyle } from './components/TSDiv.native'
export { styled } from './utils/themedStyled'

export { default as themedStylerBridge, ensureDefaultsLoaded } from '../shared/themedStylerBridge'
export { default as unifiedBridge } from '../shared/unifiedBridge'

import { initNativeThemedStyler } from './nativeThemedStyler.native'

export async function initThemedStyler(): Promise<void> {
  return initNativeThemedStyler()
}
