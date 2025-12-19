import { loadThemesFromYamlUrl, loadThemesFromYamlText } from './themedStylerBridge';
declare const unifiedBridge: {
    transpile(code: string, filename?: string): Promise<any>;
    getTranspilerVersion(): any;
    registerUsage: typeof import("./themedStylerBridge").registerUsage;
    clearUsage: typeof import("./themedStylerBridge").clearUsage;
    getUsageSnapshot: typeof import("./themedStylerBridge").getUsageSnapshot;
    registerTheme: typeof import("./themedStylerBridge").registerTheme;
    setCurrentTheme: typeof import("./themedStylerBridge").setCurrentTheme;
    getThemes: typeof import("./themedStylerBridge").getThemes;
    getThemeList: typeof import("./themedStylerBridge").getThemeList;
    getCssForWeb: typeof import("./themedStylerBridge").getCssForWeb;
    getRnStyles: typeof import("./themedStylerBridge").getRnStyles;
    loadThemesFromYamlUrl: typeof loadThemesFromYamlUrl;
    loadThemesFromYamlText: typeof loadThemesFromYamlText;
};
export default unifiedBridge;
