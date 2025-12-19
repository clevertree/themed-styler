export * from './themedPrimitives';
export * from './themedRuntime';
export { TSDiv, ThemedElement, resolveThemedStyle } from './components/TSDiv';
export { styled } from './utils/themedStyled';
export { default as themedStylerBridge, ensureDefaultsLoaded } from './themedStylerBridge';
export { default as styleManager } from './styleManager';
export { default as unifiedBridge } from './unifiedBridge';
// @ts-ignore
import wasmPath from '../wasm/themed_styler_bg.wasm';
export async function initWasmThemedStyler() {
    const g = globalThis;
    if (g.__themedStylerRenderCss && g.__themedStylerGetRn) {
        return;
    }
    try {
        // @ts-ignore
        const stylerMod = await import('../wasm/themed_styler.js');
        const init = stylerMod.default;
        await init({ module_or_path: new URL(wasmPath, import.meta.url) });
        g.__themedStylerRenderCss = (snap, themes) => {
            const state = {
                ...themes,
                used_classes: snap.classes,
                used_tags: snap.tags,
                used_tag_classes: snap.tagClasses,
                used_selectors: snap.selectors
            };
            return stylerMod.render_css_for_web(JSON.stringify(state));
        };
        g.__themedStylerGetRn = (selector, classes, themes) => {
            const res = stylerMod.get_rn_styles(JSON.stringify(themes), selector, JSON.stringify(classes));
            return res ? JSON.parse(res) : {};
        };
        console.log('[themed-styler] WASM styler ready');
    }
    catch (e) {
        console.warn('[themed-styler] Failed to initialize WASM styler', e);
    }
}
export async function initThemedStyler() {
    return initWasmThemedStyler();
}
//# sourceMappingURL=index.js.map