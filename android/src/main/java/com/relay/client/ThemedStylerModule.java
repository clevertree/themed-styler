package com.relay.client;

import org.yaml.snakeyaml.Yaml;
import com.google.gson.Gson;
import java.util.Map;

public class ThemedStylerModule {
    static {
        System.loadLibrary("themed_styler");
    }

    private static final Gson gson = new Gson();
    private static final StyleCache styleCache = new StyleCache(gson);

    public static native void nativeInitialize(String themesJson, float displayDensity, float scaledDensity);

    public static native String nativeGetAndroidStyles(String selector, String className);

    public static native String nativeProcessStyles(String stylesJson);

    public static native String nativeGetVersion();

    /**
     * Set the current theme for the cached styler
     */
    public static void setTheme(String themeJson, float density, float scaledDensity) {
        nativeInitialize(themeJson, density, scaledDensity);
        styleCache.clear();
    }

    /**
     * Get styles for a selector and classes combination (cached)
     */
    public static Map<String, Object> getStyles(String selector, String className) {
        return styleCache.getStyles(selector, className);
    }

    /**
     * Process inline styles (expand shorthands, convert units)
     */
    public static Map<String, Object> processStyles(Map<String, Object> styles) {
        return styleCache.processStyles(styles);
    }

    /**
     * Clear the style cache
     */
    public static void clearCache() {
        styleCache.clear();
    }

    /**
     * Parse YAML theme file and convert to JSON for the native styler
     */
    public static String parseThemeYaml(String yamlText) {
        try {
            Yaml yaml = new Yaml();
            Object parsed = yaml.load(yamlText);
            Gson gson = new Gson();
            return gson.toJson(parsed);
        } catch (Exception e) {
            android.util.Log.e("ThemedStylerModule", "Failed to parse YAML", e);
            return "{}";
        }
    }
}