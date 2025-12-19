import unifiedBridge from '../unifiedBridge';
export function registerThemeStyles(themeName, definitions) {
    if (!themeName || !definitions)
        return;
    try {
        unifiedBridge.registerTheme(themeName, definitions);
    }
    catch (e) {
        console.debug('Failed to register theme:', e);
    }
}
export function setThemedStylerDebug(enabled) {
    try {
        global.__THEMED_STYLER_DEBUG__ = !!enabled;
    }
    catch (e) { }
}
//# sourceMappingURL=themedMapper.js.map