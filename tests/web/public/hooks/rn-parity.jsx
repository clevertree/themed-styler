import React from 'react';

/**
 * React Native Parity Test Component
 * Tests consistency between themed-styler on Android and other platforms
 */
export default function NativeParityTest() {
    return (
        <div style={{ padding: 16, backgroundColor: '#ffffff' }}>
            <span style={{ fontSize: 24, fontWeight: 'bold', marginBottom: 16 }}>
                Android/iOS Native Parity Test
            </span>
            <span style={{ fontSize: 16, marginBottom: 8 }}>
                Using bridge-based components
            </span>
        </div>
    );
}
