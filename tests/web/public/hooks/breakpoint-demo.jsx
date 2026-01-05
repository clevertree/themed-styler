import React from 'react';

export default function BreakpointDemo() {
    return (
        <div className="p-4">
            <span className="text-2xl">Breakpoints Demo</span>
            <div className="p-2 bg-blue-500 rounded">
                <span className="text-white">sm: 640px</span>
            </div>
            <div className="p-2 bg-blue-500 rounded">
                <span className="text-white">md: 768px</span>
            </div>
            <div className="p-2 bg-blue-500 rounded">
                <span className="text-white">lg: 1024px</span>
            </div>
            <div className="p-2 bg-blue-500 rounded">
                <span className="text-white">xl: 1280px</span>
            </div>
            <div className="p-2 bg-blue-500 rounded">
                <span className="text-white">2xl: 1536px</span>
            </div>
        </div>
    );
}
