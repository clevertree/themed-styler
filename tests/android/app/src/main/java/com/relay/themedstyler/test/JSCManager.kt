package com.relay.themedstyler.test

import android.content.Context
import android.util.Log
import com.clevertree.jscbridge.JSCManager as BaseJSCManager
import com.clevertree.jscbridge.JSContext
import com.relay.client.RustTranspilerModule
import com.relay.client.ThemedStylerModule
import com.google.gson.Gson

/**
 * JSC Manager for themed-styler tests
 * Extends base JSCManager with theme support and hook transpilation
 */
class JSCManager(context: Context) : BaseJSCManager(context) {
    companion object {
        private const val TAG = "JSCManager"
        var activeManager: JSCManager? = null
    }

    private var lastAsset: String? = "test-hook.jsx"
    private var lastProps: Map<String, Any> = emptyMap()
    private var themesJson: String = "{}"

    fun getThemesJson(): String = themesJson

    init {
        loadThemes()
        // Call parent's initialize() to set up the engine
        // This will call setupModules() which installs our custom modules
        super.initialize()
    }

    private fun loadThemes() {
        try {
            themesJson = """
                {
                  "themes": {
                    "light": {
                      "variables": { 
                        "primary": "#3b82f6",
                        "secondary": "#8b5cf6",
                        "surface": "#f3f4f6",
                        "text": "#1f2937"
                      },
                      "selectors": {
                        ".bg-surface": { "backgroundColor": "#f3f4f6" },
                        ".text-themed": { "color": "#1f2937" }
                      }
                    },
                    "dark": {
                      "variables": {
                        "primary": "#60a5fa",
                        "secondary": "#a78bfa",
                        "surface": "#374151",
                        "text": "#f9fafb"
                      },
                      "selectors": {
                        ".bg-surface": { "backgroundColor": "#374151" },
                        ".text-themed": { "color": "#f9fafb" }
                      }
                    },
                    "default": {
                      "variables": { "colors": { "primary": "#3b82f6", "text": "#1f2937" } },
                      "selectors": {
                        "div": { "padding": 16 },
                        "span": { "color": "#1f2937", "fontSize": 16 },
                        ".text-blue-500": { "color": "#3b82f6" },
                        ".font-bold": { "fontWeight": "bold" },
                        ".bg-gray-100": { "backgroundColor": "#f3f4f6" },
                        ".rounded": { "borderRadius": 8 }
                      }
                    }
                  },
                  "current_theme": "light",
                  "default_theme": "default"
                }
            """.trimIndent()

            val metrics = context.resources.displayMetrics
            ThemedStylerModule.setTheme(themesJson, metrics.density, metrics.scaledDensity)
            Log.i(TAG, "Native themed-styler initialized with density=${metrics.density}")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to load themes", e)
        }
    }

    override fun setupModules(context: JSContext) {
        super.setupModules(context)
        activeManager = this
        
        // Install native transpilation function
        installNativeFunctions(context)
        // Install Android bridge (view creation, event handling)
        installAndroidBridge(context)
        // Install themed-styler virtual module
        installThemedStylerBridge(context)
        // Initialize native themed-styler hook
        initializeNativeThemedStyler(context)
        // Load the runtime
        loadRuntime(context)
        // Inject versioning information
        injectVersions(context)
    }

    private fun installNativeFunctions(context: JSContext) {
        try {
            context.evaluateScript("""
                globalThis.__native_transpile = function(source, filename) {
                    // Transpilation will happen via Android callback
                    // For now, return as-is
                    return source;
                };
            """.trimIndent(), "native_transpile.js")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to install native functions", e)
        }
    }

    private fun installAndroidBridge(context: JSContext) {
        try {
            context.evaluateScript("""
                globalThis.__android_create_view = function(propsJson) {
                    var props = JSON.parse(propsJson);
                    return 0;  // Will be implemented via native
                };
                globalThis.__android_update_view = function(viewId, propsJson) {
                    return true;
                };
                globalThis.__android_add_child = function(parentId, childId) {
                    return true;
                };
                globalThis.__android_remove_child = function(parentId, childId) {
                    return true;
                };
                globalThis.__android_on_event = function(tag, eventType) {
                    return true;
                };
                globalThis.__android_log = function(msg) {
                    console.log(msg);
                };
                globalThis.__native_get_android_styles = function(selector, classesJson, themesJson) {
                    // Native binding - will be overridden below
                    console.warn('[__native_get_android_styles] Not yet initialized');
                    return '{}';
                };
            """.trimIndent(), "android_bridge.js")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to install Android bridge", e)
        }
    }

    private fun installThemedStylerBridge(context: JSContext) {
        try {
            context.evaluateScript("""
                (function() {
                    if (!globalThis.__clevertree_packages) {
                        globalThis.__clevertree_packages = {};
                    }
                    
                    // Set up the native hook for getting Android styles
                    // This will be called by themedStylerBridge.getAndroidStyles()
                    globalThis.__themedStylerGetAndroidStyles = function(selector, classes, themesState) {
                        console.log('[__themedStylerGetAndroidStyles] Called with selector=' + selector + ' classes=' + JSON.stringify(classes));
                        // This function will be replaced by the native init
                        return {};
                    };
                    
                    if (!globalThis.__themed_styler_state) {
                        globalThis.__themed_styler_state = {
                            themes: {},
                            currentTheme: 'light',
                            usage: { tags: new Set(), classes: new Set() }
                        };
                    }
                    
                    var state = globalThis.__themed_styler_state;
                    
                    globalThis.__clevertree_packages['@clevertree/themed-styler'] = {
                        setCurrentTheme: function(name) {
                            console.log('[themed-styler] setCurrentTheme: ' + name);
                            state.currentTheme = name;
                            return true;
                        },
                        getThemes: function() {
                            console.log('[themed-styler] getThemes');
                            return {
                                themes: state.themes,
                                currentTheme: state.currentTheme,
                                current_theme: state.currentTheme
                            };
                        },
                        getThemeList: function() {
                            var list = [];
                            for (var key in state.themes) list.push({ key: key, name: key });
                            return list;
                        },
                        registerTheme: function(name, defs) {
                            state.themes[name] = defs || {};
                            if (!state.currentTheme) state.currentTheme = name;
                            return true;
                        },
                        clearUsage: function() {
                            state.usage.tags.clear();
                            state.usage.classes.clear();
                            return true;
                        },
                        getUsageSnapshot: function() {
                            return {
                                tags: Array.from(state.usage.tags),
                                classes: Array.from(state.usage.classes)
                            };
                        }
                    };
                    
                    console.log('[INIT] @clevertree/themed-styler registered');
                })();
            """.trimIndent(), "register_themed_styler.js")
            
            // Now load themes from Kotlin's loadThemes() function
            // Must be done as a separate script after the bridge is set up
            val themesJson = getThemesJson()
            context.evaluateScript("""
                (function() {
                    try {
                        var themesData = $themesJson;
                        console.log('[INIT] Loading themes from Kotlin, keys: ' + Object.keys(themesData.themes || {}).join(', '));
                        
                        // Register each theme
                        if (themesData.themes) {
                            for (var themeName in themesData.themes) {
                                var themeDef = themesData.themes[themeName];
                                globalThis.__clevertree_packages['@clevertree/themed-styler'].registerTheme(themeName, themeDef);
                                console.log('[INIT] Registered theme: ' + themeName);
                            }
                        }
                        
                        // Set current theme
                        if (themesData.current_theme) {
                            globalThis.__clevertree_packages['@clevertree/themed-styler'].setCurrentTheme(themesData.current_theme);
                            console.log('[INIT] Set current theme to: ' + themesData.current_theme);
                        }
                    } catch (e) {
                        console.error('[INIT] Failed to load themes: ' + e);
                    }
                })();
            """.trimIndent(), "load_themes.js")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to install themed-styler bridge", e)
        }
    }

    private fun initializeNativeThemedStyler(context: JSContext) {
        try {
            // Replace the __themedStylerGetAndroidStyles placeholder with actual native call
            context.evaluateScript("""
                (function() {
                    globalThis.__themedStylerGetAndroidStyles = function(selector, classes, themesState) {
                        var classesJson = JSON.stringify(classes);
                        var themesJson = JSON.stringify(themesState);
                        console.log('[__themedStylerGetAndroidStyles] Calling native: selector=' + selector);
                        
                        // Call native function - will be bridged via JNI
                        try {
                            var result = __native_get_android_styles(selector, classesJson, themesJson);
                            return JSON.parse(result);
                        } catch (e) {
                            console.error('[__themedStylerGetAndroidStyles] Error: ' + e);
                            return {};
                        }
                    };
                    console.log('[INIT] __themedStylerGetAndroidStyles initialized');
                })();
            """.trimIndent(), "init_native_styler.js")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to initialize native themed-styler", e)
        }
    }

    private fun loadRuntime(context: JSContext) {
        try {
            context.evaluateScript("""
                globalThis.__require_module = function(id) {
                    if (!id) return {};
                    console.log('[__require_module] Checking __clevertree_packages for: ' + id);
                    if (globalThis.__clevertree_packages && id in globalThis.__clevertree_packages) {
                        console.log('[__require_module] Found in __clevertree_packages: ' + id);
                        var pkg = globalThis.__clevertree_packages[id];
                        return { default: pkg, ...pkg };
                    }
                    throw new Error('Module not found: ' + id);
                };
            """.trimIndent(), "require_module.js")
            Log.d(TAG, "Runtime loaded")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to load runtime", e)
        }
    }

    private fun injectVersions(context: JSContext) {
        try {
            val transpilerVersion = RustTranspilerModule.nativeGetVersion()
            val stylerVersion = ThemedStylerModule.nativeGetVersion()
            
            context.evaluateScript("""
                globalThis.__versions = {
                    transpiler: '${transpilerVersion}',
                    styler: '${stylerVersion}',
                    engine: 'jsc'
                };
            """.trimIndent(), "inject_versions.js")
            
            Log.i(TAG, "Injected versions: transpiler=$transpilerVersion, styler=$stylerVersion, engine=jsc")
        } catch (e: Exception) {
            Log.w(TAG, "Failed to inject versions", e)
        }
    }

    fun renderHook(asset: String, props: Map<String, Any> = emptyMap()) {
        lastAsset = asset
        lastProps = props

        val jsCtx = getContext() ?: run {
            Log.w(TAG, "JSC not initialized")
            userMessageHandler?.invoke("Engine is not ready yet. Try restarting.", true)
            return
        }

        AndroidRenderer.clearAll()
        jsCtx.evaluateScript("globalThis.__last_module__ = null; delete globalThis.__hook_props;", "reset_globals.js")

        val propsJson = gson.toJson(props)
        jsCtx.evaluateScript("globalThis.__hook_props = ${propsJson};", "props_${asset}.js")

        val source = readAssetFile(asset)
        if (source.isEmpty()) {
            Log.w(TAG, "Asset $asset is empty; skipping render")
            userMessageHandler?.invoke("Could not load $asset from app assets.", true)
            return
        }

        try {
            val transpiled = RustTranspilerModule.nativeTranspile(source, asset, true)
            Log.d(TAG, "Transpiled $asset (${source.length} → ${transpiled.length} bytes)")
            
            val wrappedCode = """
                globalThis.__last_module__ = $transpiled;
            """.trimIndent()
            
            try {
                jsCtx.evaluateScript(wrappedCode, asset)
                Log.i(TAG, "Hook executed: $asset")
            } catch (e: Exception) {
                Log.e(TAG, "Error executing transpiled code: ${e.message}", e)
                userMessageHandler?.invoke("Transpilation error: ${e.message}", true)
                throw e
            }

            val checkExport = jsCtx.evaluateScript("typeof globalThis.__last_module__", "check_export.js")
            Log.d(TAG, "Module export type: $checkExport")

            val hasDefault = jsCtx.evaluateScript("globalThis.__last_module__ && typeof globalThis.__last_module__.default", "check_default.js")
            Log.d(TAG, "Has default export: $hasDefault")

            drainMessageQueue()

            val defaultExportName = jsCtx.evaluateScript("globalThis.__last_module__.default?.name", "get_export_name.js")
            Log.i(TAG, "Default export name: $defaultExportName")

            // Call the component
            try {
                jsCtx.evaluateScript("""
                    globalThis.__render_result = {
                        tag: globalThis.__tag_counter || 0,
                        type: 'div',
                        props: globalThis.__hook_props || {},
                        children: []
                    };
                    
                    try {
                        const component = globalThis.__last_module__.default;
                        if (typeof component === 'function') {
                            const result = component(globalThis.__hook_props || {});
                            globalThis.__render_result = result;
                        }
                    } catch(e) {
                        console.error('Component render error: ' + e.message);
                        throw e;
                    }
                """.trimIndent(), "call_component.js")
                
                drainMessageQueue()
            } catch (e: Exception) {
                Log.e(TAG, "Component execution failed: ${e.message}", e)
                userMessageHandler?.invoke("Component error: ${e.message}", true)
            }
        } catch (e: Exception) {
            Log.e(TAG, "Hook loading failed", e)
            userMessageHandler?.invoke("Hook loading failed: ${e.message}", true)
        }
    }

    private fun readAssetFile(filename: String): String {
        return try {
            context.assets.open(filename).bufferedReader().use { it.readText() }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to read asset: $filename", e)
            ""
        }
    }

    override fun cleanup() {
        super.cleanup()
    }

    fun drainMessageQueue() {
        // Drain console messages
        try {
            val logsJson = getContext()?.evaluateScript("JSON.stringify(globalThis.__console_logs)", "drain_logs.js") ?: return
            if (logsJson.isNotEmpty() && logsJson != "[]") {
                Log.d(TAG, "Console: $logsJson")
            }
        } catch (e: Exception) {
            Log.w(TAG, "Failed to drain message queue", e)
        }
    }
}
