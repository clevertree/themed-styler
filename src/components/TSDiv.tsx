import React, { useEffect } from 'react'
import unifiedBridge from '../unifiedBridge'
import styleManager from '../styleManager'

export function useThemedStyles(tag: string, className?: string) {
    useEffect(() => {
        try {
            unifiedBridge.registerUsage(tag, { className })
            styleManager.requestRender()
        } catch (e) {
            // no-op
        }
    }, [tag, className])

    return { style: undefined }
}

export function resolveThemedStyle(tag: string, className?: string) {
    try {
        unifiedBridge.registerUsage(tag, { className })
        styleManager.requestRender()
    } catch (e) {
        // ignore
    }
    return undefined
}

export const ThemedElement = React.forwardRef<any, any>(
    ({ component: Component, tag = 'div', className, style, children, ...rest }, ref) => {
        useThemedStyles(tag, className)
        return (
            <Component ref={ref} className={className} style={style} {...rest}>
                {children}
            </Component>
        )
    },
)

export const TSDiv = React.forwardRef<any, any>(
    ({ component, tag = 'div', className, style, children, ...rest }, ref) => {
        useThemedStyles(tag, className)
        const ResolvedComponent = component || tag
        
        // Special handling for void elements - they cannot have children
        const voidElements = ['img', 'input', 'br', 'hr', 'area', 'base', 'col', 'embed', 'link', 'meta', 'param', 'source', 'track', 'wbr']
        if (voidElements.includes(tag)) {
            return <ResolvedComponent ref={ref} className={className} style={style} {...rest} />
        }

        return (
            <ResolvedComponent ref={ref} className={className} style={style} {...rest}>
                {children}
            </ResolvedComponent>
        )
    },
)
