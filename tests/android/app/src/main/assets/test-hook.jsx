import React, { useState } from 'react';
import { setCurrentTheme, getThemes } from '@clevertree/themed-styler';

export default function ThemedDemo() {
  // Initialize from themed-styler bridge state, then maintain local state for re-renders
  const [theme, setTheme] = useState(() => getThemes().currentTheme || 'light');

  const toggleTheme = () => {
    const themes = getThemes().themes;
    const themeList = Object.keys(themes);
    if (themeList.length === 0) return;

    const currentIndex = themeList.indexOf(theme);
    const nextIndex = (currentIndex + 1) % themeList.length;
    const nextTheme = themeList[nextIndex];

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
        <span className="text-2xl font-bold">Themed Demo</span>
        <button onClick={toggleTheme} className="px-4 py-2 rounded">
          Next Theme
        </button>
      </div>

      <div className="p-4 rounded mb-6 bg-surface">
        <span className="text-lg font-semibold mb-2">Current Theme</span>
        <span>Theme: <span style={{ fontWeight: 'bold' }}>{theme.toUpperCase()}</span></span>
      </div>

      <div className="mb-6">
        <span className="text-lg font-semibold mb-2">Color Palette</span>
        <div className="flex flex-col gap-2">
          <div className="p-4 rounded bg-primary">
            <span>Primary Color (theme-aware)</span>
          </div>
          <div className="p-4 rounded bg-secondary">
            <span>Secondary Color (theme-aware)</span>
          </div>
          <div className="p-4 rounded border-themed bg-surface">
            <span className="text-themed">Surface with Border (theme-aware)</span>
          </div>
        </div>
      </div>

      <div className="mb-6">
        <span className="text-lg font-semibold mb-2">Flex Layout</span>
        <div className="flex flex-row p-2 rounded bg-surface">
          <div className="p-3 rounded m-1 bg-primary flex-1">
            <span>Box 1</span>
          </div>
          <div className="p-3 rounded m-1 bg-secondary flex-1">
            <span>Box 2</span>
          </div>
          <div className="p-3 rounded m-1 bg-primary flex-1">
            <span>Box 3</span>
          </div>
        </div>
      </div>

      <div className="p-4 rounded bg-surface">
        <span className="text-lg font-semibold mb-2">Theme Switching</span>
        <span>Click the button above to cycle through all available themes. The UI will re-render with the appropriate colors and styles.</span>
      </div>
    </body>
  );
}
