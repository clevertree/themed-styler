import { jsx as _jsx } from "react/jsx-runtime";
import React, { useEffect } from 'react';
import unifiedBridge from '../unifiedBridge';
import styleManager from '../styleManager';
export function useThemedStyles(tag, className) {
    useEffect(() => {
        try {
            unifiedBridge.registerUsage(tag, { className });
            styleManager.requestRender();
        }
        catch (e) {
            // no-op
        }
    }, [tag, className]);
    return { style: undefined };
}
export function resolveThemedStyle(tag, className) {
    try {
        unifiedBridge.registerUsage(tag, { className });
        styleManager.requestRender();
    }
    catch (e) {
        // ignore
    }
    return undefined;
}
export const ThemedElement = React.forwardRef(({ component: Component, tag = 'div', className, style, children, ...rest }, ref) => {
    useThemedStyles(tag, className);
    return (_jsx(Component, { ref: ref, className: className, style: style, ...rest, children: children }));
});
export const TSDiv = React.forwardRef(({ component, tag = 'div', className, style, children, ...rest }, ref) => {
    useThemedStyles(tag, className);
    const ResolvedComponent = component || tag;
    // Special handling for void elements - they cannot have children
    const voidElements = ['img', 'input', 'br', 'hr', 'area', 'base', 'col', 'embed', 'link', 'meta', 'param', 'source', 'track', 'wbr'];
    if (voidElements.includes(tag)) {
        return _jsx(ResolvedComponent, { ref: ref, className: className, style: style, ...rest });
    }
    return (_jsx(ResolvedComponent, { ref: ref, className: className, style: style, ...rest, children: children }));
});
//# sourceMappingURL=TSDiv.js.map