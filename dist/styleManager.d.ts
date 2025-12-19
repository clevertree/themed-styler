export declare function ensureStyleElement(): HTMLStyleElement;
export declare function renderCssIntoDom(): void;
export declare function requestRender(): void;
export declare function wrapCreateElement(reactModule: any): any;
export declare function useStyleManager(cb?: (ev?: Event) => void): {
    requestRender: typeof requestRender;
    renderCssIntoDom: typeof renderCssIntoDom;
};
export declare function tearDownStyleElement(): void;
export declare function startAutoSync(pollInterval?: number): void;
export declare function stopAutoSync(): void;
export declare function onChange(cb: (ev?: Event) => void): () => void;
declare const _default: {
    ensureStyleElement: typeof ensureStyleElement;
    renderCssIntoDom: typeof renderCssIntoDom;
    tearDownStyleElement: typeof tearDownStyleElement;
    startAutoSync: typeof startAutoSync;
    stopAutoSync: typeof stopAutoSync;
    requestRender: typeof requestRender;
    onChange: typeof onChange;
    wrapCreateElement: typeof wrapCreateElement;
    useStyleManager: typeof useStyleManager;
};
export default _default;
