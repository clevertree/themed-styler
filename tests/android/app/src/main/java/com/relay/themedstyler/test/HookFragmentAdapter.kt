package com.relay.themedstyler.test

import androidx.fragment.app.Fragment
import androidx.fragment.app.FragmentActivity
import androidx.viewpager2.adapter.FragmentStateAdapter

class HookFragmentAdapter(fragmentActivity: FragmentActivity) : FragmentStateAdapter(fragmentActivity) {
    override fun getItemCount(): Int = 6

    override fun createFragment(position: Int): Fragment {
        return when (position) {
            0 -> LocalHookFragment.newInstance("test-hook.jsx")
            1 -> LocalHookFragment.newInstance("breakpoint-demo.jsx")
            2 -> LocalHookFragment.newInstance("style-keywords-test.jsx")
            3 -> LocalHookFragment.newInstance("string-keywords-test.jsx")
            4 -> LocalHookFragment.newInstance("rn-parity.jsx")
            5 -> LocalHookFragment.newInstance("static-import-test.jsx")
            else -> LocalHookFragment.newInstance("test-hook.jsx")
        }
    }
}
