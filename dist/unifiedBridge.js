import themedStylerBridge, { loadThemesFromYamlUrl, loadThemesFromYamlText } from './themedStylerBridge';
// Unified bridge that exposes both hook-transpiler transpile API and the themed-styler runtime bridge
const unifiedBridge = {
    // Transpile API: wraps global hook-transpiler function if present
    async transpile(code, filename) {
        const g = typeof globalThis !== 'undefined' ? globalThis : {};
        const fn = g.__hook_transpile_jsx;
        if (typeof fn === 'function') {
            return await fn(code, filename || 'module.tsx');
        }
        throw new Error('hook-transpiler not initialized');
    },
    getTranspilerVersion() {
        const g = typeof globalThis !== 'undefined' ? globalThis : {};
        return g.__hook_transpiler_version || null;
    },
    // Themed-styler delegation
    registerUsage: themedStylerBridge.registerUsage,
    clearUsage: themedStylerBridge.clearUsage,
    getUsageSnapshot: themedStylerBridge.getUsageSnapshot,
    registerTheme: themedStylerBridge.registerTheme,
    setCurrentTheme: themedStylerBridge.setCurrentTheme,
    getThemes: themedStylerBridge.getThemes,
    getThemeList: themedStylerBridge.getThemeList,
    getCssForWeb: themedStylerBridge.getCssForWeb,
    getRnStyles: themedStylerBridge.getRnStyles,
    // Theme YAML helpers
    loadThemesFromYamlUrl,
    loadThemesFromYamlText,
};
export default unifiedBridge;
//# sourceMappingURL=unifiedBridge.js.map