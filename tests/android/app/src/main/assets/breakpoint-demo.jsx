import React from 'react';

export default function BreakpointDemo() {
    return (
        <div className="p-4">
            <text className="text-2xl">Breakpoints Demo</text>
            <div className="p-2 bg-blue-500 rounded">
                <text className="text-white">sm: 640px</text>
            </div>
            <div className="p-2 bg-blue-500 rounded">
                <text className="text-white">md: 768px</text>
            </div>
            <div className="p-2 bg-blue-500 rounded">
                <text className="text-white">lg: 1024px</text>
            </div>
            <div className="p-2 bg-blue-500 rounded">
                <text className="text-white">xl: 1280px</text>
            </div>
            <div className="p-2 bg-blue-500 rounded">
                <text className="text-white">2xl: 1536px</text>
            </div>
        </div>
    );
}
