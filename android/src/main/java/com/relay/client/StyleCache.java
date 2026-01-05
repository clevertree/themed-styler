package com.relay.client;

import android.util.Log;
import android.util.LruCache;
import com.google.gson.Gson;
import java.util.Map;

/**
 * Unified style caching system for Android
 * Caches results from nativeGetAndroidStyles to avoid JNI overhead
 */
public class StyleCache {
    private static final String TAG = "StyleCache";
    private static final int MAX_CACHE_SIZE = 1000; // Max number of style entries
    private final Gson gson;

    // Computed style cache: key = selector|className, value = style properties map
    // Using LruCache to prevent memory leaks
    private final LruCache<String, Map<String, Object>> styleCache = new LruCache<>(MAX_CACHE_SIZE);

    public StyleCache(Gson gson) {
        this.gson = gson;
    }

    /**
     * Get styles for a selector and classes combination
     * Lazily computes and caches styles on first access
     */
    public Map<String, Object> getStyles(String selector, String className) {
        String cacheKey = selector + "|" + className;
        Map<String, Object> cachedStyles;

        synchronized (styleCache) {
            cachedStyles = styleCache.get(cacheKey);
        }

        if (cachedStyles != null) {
            return cachedStyles;
        }

        Log.d(TAG, "[Cache] Cache MISS for " + cacheKey + ", computing on-demand...");
        Map<String, Object> styles = computeStyleForElement(selector, className);

        synchronized (styleCache) {
            styleCache.put(cacheKey, styles);
        }

        return styles;
    }

    /**
     * Compute styles for a single element (calls native code)
     */
    private Map<String, Object> computeStyleForElement(String selector, String className) {
        try {
            String stylesJson = ThemedStylerModule.nativeGetAndroidStyles(selector, className);
            Log.d(TAG, "[Compute] Result for " + selector + "." + className + ": " + stylesJson);

            if (stylesJson != null && !stylesJson.isEmpty() && !stylesJson.equals("{}")) {
                return gson.fromJson(stylesJson, Map.class);
            } else {
                return new java.util.HashMap<>();
            }
        } catch (Exception e) {
            Log.e(TAG, "[Compute] Error computing styles for " + selector + "." + className, e);
            return new java.util.HashMap<>();
        }
    }

    /**
     * Process inline styles (expand shorthands, convert units)
     */
    public Map<String, Object> processStyles(Map<String, Object> styles) {
        try {
            String stylesJson = gson.toJson(styles);
            String processedJson = ThemedStylerModule.nativeProcessStyles(stylesJson);

            if (processedJson != null && !processedJson.isEmpty() && !processedJson.equals("{}")) {
                return gson.fromJson(processedJson, Map.class);
            } else {
                return styles;
            }
        } catch (Exception e) {
            Log.e(TAG, "[Process] Error processing inline styles", e);
            return styles;
        }
    }

    public void clear() {
        synchronized (styleCache) {
            styleCache.evictAll();
        }
    }
}
