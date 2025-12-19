/**
 * In-memory bridge for themed-styler usage.
 * Collected at runtime by web and RN HookRenderers via shared imports.
 */
type Props = Record<string, any>;
type HierNode = {
    tag: string;
    classes?: string[];
};
export declare function registerUsage(tag: string, props?: Props, hierarchy?: HierNode[]): void;
export declare function clearUsage(): void;
export declare function getUsageSnapshot(): {
    tags: string[];
    classes: string[];
    tagClasses: string[];
    selectors: string[];
};
export declare function registerTheme(name: string, defs?: Record<string, unknown>): void;
export declare function setCurrentTheme(name: string): void;
export declare function getThemes(): {
    themes: {
        [x: string]: Record<string, any>;
    };
    currentTheme: string;
    current_theme: string;
    default_theme: string;
    variables: {};
    breakpoints: {};
};
export declare function getThemeList(): Array<{
    key: string;
    name: string;
}>;
export declare function loadThemesFromYamlText(yamlText: string): Promise<void>;
export declare function loadThemesFromYamlUrl(url: string): Promise<void>;
export declare function ensureDefaultsLoaded(): Promise<void>;
export declare function getCssForWeb(): string;
export declare function getRnStyles(selector: string, classes?: string[]): any;
declare const _default: {
    registerUsage: typeof registerUsage;
    clearUsage: typeof clearUsage;
    getUsageSnapshot: typeof getUsageSnapshot;
    registerTheme: typeof registerTheme;
    setCurrentTheme: typeof setCurrentTheme;
    getThemes: typeof getThemes;
    getThemeList: typeof getThemeList;
    getCssForWeb: typeof getCssForWeb;
    getRnStyles: typeof getRnStyles;
    loadThemesFromYamlText: typeof loadThemesFromYamlText;
    loadThemesFromYamlUrl: typeof loadThemesFromYamlUrl;
};
export default _default;
