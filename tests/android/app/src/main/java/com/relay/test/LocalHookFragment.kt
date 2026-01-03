package com.relay.test

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.FrameLayout
import android.widget.TextView
import androidx.fragment.app.Fragment
import com.clevertree.hooktranspiler.render.HookRenderer
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import java.io.InputStreamReader
import java.text.SimpleDateFormat
import java.util.*

/**
 * Local Hook Fragment
 * Uses HookRenderer to transpile local asset, then JSCManager to execute
 */
class LocalHookFragment : Fragment() {
    private lateinit var hookRenderer: HookRenderer
    private lateinit var outputView: TextView
    private lateinit var containerView: FrameLayout
    private lateinit var loadButton: Button
    private var rendererMode = com.clevertree.hooktranspiler.model.RendererMode.ACT
    private val scope = CoroutineScope(Dispatchers.Main + Job())
    
    companion object {
        private const val ARG_FILENAME = "filename"
        
        fun newInstance(filename: String = "test-hook.jsx"): LocalHookFragment {
            return LocalHookFragment().apply {
                arguments = Bundle().apply {
                    putString(ARG_FILENAME, filename)
                }
            }
        }
    }
    
    private val assetFilename: String
        get() = arguments?.getString(ARG_FILENAME) ?: "test-hook.jsx"

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        return inflater.inflate(R.layout.fragment_local_hook, container, false)
    }

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        outputView = view.findViewById(R.id.tv_output)
        containerView = view.findViewById(R.id.js_container)
        loadButton = view.findViewById(R.id.btn_load_local)
        
        // val btnRendererAct = view.findViewById<Button>(R.id.btn_renderer_act)
        // val btnRendererReact = view.findViewById<Button>(R.id.btn_renderer_react)

        // Initialize HookRenderer
        hookRenderer = HookRenderer(requireContext())
        containerView.addView(hookRenderer, FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.MATCH_PARENT,
            FrameLayout.LayoutParams.MATCH_PARENT
        ))

        hookRenderer.onLoading = { logOutput("Loading hook...") }
        hookRenderer.onReady = { viewCount -> logOutput("Hook ready and rendered ($viewCount native views)") }
        hookRenderer.onError = { error -> logOutput("Error: ${error.message}") }

        loadButton.setOnClickListener {
            loadLocalHook()
        }
        
        // btnRendererAct.setOnClickListener {
        //     rendererMode = com.clevertree.hooktranspiler.model.RendererMode.ACT
        //     hookRenderer.setRendererMode(rendererMode)
        //     logOutput("Switched to Act renderer")
        //     updateRendererButtons(btnRendererAct, btnRendererReact)
        // }
        
        // btnRendererReact.setOnClickListener {
        //     rendererMode = com.clevertree.hooktranspiler.model.RendererMode.REACT_NATIVE
        //     hookRenderer.setRendererMode(rendererMode)
        //     logOutput("Switched to React Native renderer (for testing)")
        //     updateRendererButtons(btnRendererAct, btnRendererReact)
        // }
        
        // updateRendererButtons(btnRendererAct, btnRendererReact)

        // Auto-load on start
        view.postDelayed({ loadLocalHook() }, 300)
    }
    
    private fun updateRendererButtons(btnAct: Button, btnReact: Button) {
        btnAct.isEnabled = rendererMode != com.clevertree.hooktranspiler.model.RendererMode.ACT
        btnReact.isEnabled = rendererMode != com.clevertree.hooktranspiler.model.RendererMode.REACT_NATIVE
    }

    private fun loadLocalHook() {
        logOutput("Loading $assetFilename from assets...")
        
        // Load theme.yaml from assets if it exists
        val themes = try {
            val yaml = requireContext().assets.open("theme.yaml").bufferedReader().use { it.readText() }
            // For now, we'll just pass a simple map since we don't have a YAML parser here
            // In a real app, you'd parse the YAML or use JSON
            null 
        } catch (e: Exception) {
            null
        }

        // The themed-styler test app has a hardcoded theme in JSCManager, 
        // but HookRenderer is independent. Let's provide a basic theme in props
        // so we can verify theme switching works.
        val props = mapOf(
            "env" to mapOf("theme" to "light"),
            "themes" to mapOf(
                "themes" to mapOf(
                    "light" to mapOf(
                        "variables" to mapOf(
                            "colors" to mapOf(
                                "primary" to "#3b82f6",
                                "secondary" to "#10b981",
                                "text" to "#1f2937",
                                "background" to "#ffffff",
                                "surface" to "#f3f4f6",
                                "border" to "#d1d5db"
                            )
                        ),
                        "selectors" to mapOf(
                            "body" to mapOf("backgroundColor" to "#ffffff", "flex" to 1, "padding" to 16),
                            "text" to mapOf("color" to "#1f2937"),
                            "button" to mapOf("backgroundColor" to "#3b82f6", "color" to "#ffffff", "padding" to 12, "borderRadius" to 8),
                            ".bg-primary" to mapOf("backgroundColor" to "#3b82f6"),
                            ".bg-secondary" to mapOf("backgroundColor" to "#10b981"),
                            ".bg-surface" to mapOf("backgroundColor" to "#f3f4f6"),
                            ".text-themed" to mapOf("color" to "#1f2937"),
                            ".border-themed" to mapOf("borderColor" to "#d1d5db", "borderWidth" to 2)
                        )
                    ),
                    "dark" to mapOf(
                        "variables" to mapOf(
                            "colors" to mapOf(
                                "primary" to "#60a5fa",
                                "secondary" to "#a78bfa",
                                "text" to "#f9fafb",
                                "background" to "#111827",
                                "surface" to "#1f2937",
                                "border" to "#374151"
                            )
                        ),
                        "selectors" to mapOf(
                            "body" to mapOf("backgroundColor" to "#111827", "flex" to 1, "padding" to 16),
                            "text" to mapOf("color" to "#f9fafb"),
                            "button" to mapOf("backgroundColor" to "#60a5fa", "color" to "#111827", "padding" to 12, "borderRadius" to 8),
                            ".bg-primary" to mapOf("backgroundColor" to "#60a5fa"),
                            ".bg-secondary" to mapOf("backgroundColor" to "#a78bfa"),
                            ".bg-surface" to mapOf("backgroundColor" to "#1f2937"),
                            ".text-themed" to mapOf("color" to "#f9fafb"),
                            ".border-themed" to mapOf("borderColor" to "#374151", "borderWidth" to 2)
                        )
                    )
                ),
                "current_theme" to "light",
                "default_theme" to "light"
            )
        )

        hookRenderer.loadHook(assetFilename, props)
    }

    private fun logOutput(message: String) {
        val timestamp = SimpleDateFormat("HH:mm:ss", Locale.getDefault()).format(Date())
        val currentText = outputView.text.toString()
        outputView.text = "[$timestamp] $message\n$currentText"
    }

    override fun onDestroyView() {
        super.onDestroyView()
        scope.coroutineContext[Job]?.cancel()
    }
}
