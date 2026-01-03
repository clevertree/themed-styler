import React, { useState } from 'react';
import { setCurrentTheme, getThemes } from '@clevertree/themed-styler';

export default function ThemedDemo() {
  // Initialize from themed-styler bridge state, then maintain local state for re-renders
  const [theme, setTheme] = useState(() => getThemes().currentTheme || 'light');

  const toggleTheme = () => {
    const nextTheme = theme === 'light' ? 'dark' : 'light';
    setCurrentTheme(nextTheme);  // Update bridge state
    setTheme(nextTheme);          // Update local state to trigger re-render
  };

  // Auto-switch theme every 3 seconds for debugging
  React.useEffect(() => {
    const timer = setInterval(toggleTheme, 3000);
    return () => clearInterval(timer);
  }, [theme]);

  return (
    <body>
      <div className="mb-6 flex flex-row items-center justify-between">
        <text className="text-2xl font-bold">Themed Demo</text>
        <button onClick={toggleTheme} className="px-4 py-2 rounded">
          {theme === 'light' ? 'Switch to Dark' : 'Switch to Light'}
        </button>
      </div>

      <div className="p-4 rounded mb-6 bg-surface">
        <text className="text-lg font-semibold mb-2">Current Theme</text>
        <text>Theme: <text style={{ fontWeight: 'bold' }}>{theme.toUpperCase()}</text></text>
      </div>

      <div className="mb-6">
        <text className="text-lg font-semibold mb-2">Color Palette</text>
        <div className="flex flex-col gap-2">
          <div className="p-4 rounded bg-primary">
            <text style={{ color: 'white' }}>Primary Color (theme-aware)</text>
          </div>
          <div className="p-4 rounded bg-secondary">
            <text style={{ color: 'white' }}>Secondary Color (theme-aware)</text>
          </div>
          <div className="p-4 rounded border-themed bg-surface">
            <text className="text-themed">Surface with Border (theme-aware)</text>
          </div>
        </div>
      </div>

      <div className="mb-6">
        <text className="text-lg font-semibold mb-2">Flex Layout</text>
        <div className="flex flex-row p-2 rounded bg-surface">
          <div className="p-3 rounded m-1 bg-primary flex-1">
            <text style={{ color: 'white' }}>Box 1</text>
          </div>
          <div className="p-3 rounded m-1 bg-secondary flex-1">
            <text style={{ color: 'white' }}>Box 2</text>
          </div>
          <div className="p-3 rounded m-1 bg-primary flex-1">
            <text style={{ color: 'white' }}>Box 3</text>
          </div>
        </div>
      </div>

      <div className="p-4 rounded bg-surface">
        <text className="text-lg font-semibold mb-2">Theme Switching</text>
        <text>Click the button above to toggle between light and dark themes. The UI will re-render with the appropriate colors and styles.</text>
      </div>
    </body>
  );
}
