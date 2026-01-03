import React from 'react';

/**
 * React Native Parity Test Component
 * Tests consistency between themed-styler on Android and other platforms
 */
export default function NativeParityTest() {
    return (
        <div style={{ padding: 16, backgroundColor: '#ffffff' }}>
            <text style={{ fontSize: 24, fontWeight: 'bold', marginBottom: 16 }}>
                Android/iOS Native Parity Test
            </text>
            <text style={{ fontSize: 16, marginBottom: 8 }}>
                Using bridge-based components
            </text>
        </div>
    );
}
