import { jsx as _jsx } from "react/jsx-runtime";
import React, { useMemo, useEffect } from 'react';
import { SafeAreaView, ScrollView, Text, TouchableOpacity, View } from 'react-native';
import unifiedBridge from '../unifiedBridge';
function parseClassName(className) {
    if (!className || typeof className !== 'string')
        return [];
    return className.trim().split(/\s+/).filter(Boolean);
}
export function useThemedStyles(tag, className) {
    const classes = useMemo(() => parseClassName(className), [className]);
    useEffect(() => {
        try {
            unifiedBridge.registerUsage(tag, { className });
        }
        catch (e) {
            console.warn('[TSDiv] registerUsage failed', e);
        }
    }, [tag, className]);
    const style = classes.length ? unifiedBridge.getRnStyles(tag, classes) : undefined;
    return { style };
}
export const ThemedElement = React.forwardRef(({ component: Component, tag = 'div', className, style, children, ...rest }, ref) => {
    const { style: themedStyle } = useThemedStyles(tag, className);
    const mergedStyle = themedStyle ? [themedStyle, style] : style;
    return (_jsx(Component, { ref: ref, style: mergedStyle, ...rest, children: children }));
});
export function resolveThemedStyle(tag, className) {
    const classes = parseClassName(className);
    try {
        unifiedBridge.registerUsage(tag, { className });
    }
    catch (e) {
        // ignore
    }
    return classes.length ? unifiedBridge.getRnStyles(tag, classes) : undefined;
}
const overflowClassRegex = /\boverflow(?:-[xy])?-[a-z0-9-]+\b/i;
const buttonComponent = TouchableOpacity ?? View;
const tagComponentMap = {
    span: Text,
    button: buttonComponent,
    main: SafeAreaView,
    'safe-area': SafeAreaView,
    'safe-area-view': SafeAreaView,
    scroll: ScrollView,
};
function hasOverflowClass(className) {
    return Boolean(className && overflowClassRegex.test(className));
}
function prefersScrollView(rest, className) {
    if (hasOverflowClass(className))
        return true;
    const scrollProps = [
        'horizontal',
        'contentContainerStyle',
        'showsHorizontalScrollIndicator',
        'showsVerticalScrollIndicator',
        'onScroll',
        'refreshControl',
        'nestedScrollEnabled',
        'scrollEnabled',
    ];
    return scrollProps.some((prop) => Object.prototype.hasOwnProperty.call(rest, prop));
}
export const TSDiv = React.forwardRef(({ component, tag = 'div', className, style, children, ...rest }, ref) => {
    const normalizedTag = tag?.toLowerCase?.() ?? 'div';
    const ResolvedComponent = component ||
        tagComponentMap[normalizedTag] ||
        (prefersScrollView(rest, className) ? ScrollView : View);
    return (_jsx(ThemedElement, { component: ResolvedComponent, tag: tag, className: className, style: style, ref: ref, ...rest, children: children }));
});
//# sourceMappingURL=TSDiv.native.js.map