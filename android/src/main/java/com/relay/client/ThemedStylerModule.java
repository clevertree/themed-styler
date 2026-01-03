package com.relay.client;

import org.yaml.snakeyaml.Yaml;
import com.google.gson.Gson;

public class ThemedStylerModule {
    static {
        System.loadLibrary("themed_styler");
    }

    public static native String nativeRenderCss(String usageJson, String themesJson);
    public static native String nativeGetAndroidStyles(String selector, String classesJson, String themesJson);
    public static native String nativeGetDefaultState();
    public static native String nativeGetVersion();
    
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