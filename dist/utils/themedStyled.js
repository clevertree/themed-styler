import React from 'react';
import { resolveThemedStyle } from '../components/TSDiv';
export function styled(Component, tagName = 'div') {
    const Wrapped = React.forwardRef((props, ref) => {
        const forwardedProps = props;
        const { className, style, ...rest } = forwardedProps;
        const computed = resolveThemedStyle(tagName, className);
        const mergedStyle = computed ? [computed, style] : style;
        return React.createElement(Component, { ...rest, style: mergedStyle, ref });
    });
    Wrapped.displayName = `Styled(${Component.displayName || Component.name || 'Component'})`;
    return Wrapped;
}
//# sourceMappingURL=themedStyled.js.map