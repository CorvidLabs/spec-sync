/* ============================================================
   CorvidLabs theme controller — the standard light/dark behavior.

   Drop this on any page that imports tokens.css. It:
   - defaults to the OS preference (prefers-color-scheme)
   - lets ?theme=light|dark force a mode (handy for QA screenshots)
   - persists an explicit choice in localStorage
   - wires every [data-corvid-theme-toggle] button: click to flip,
     with aria-pressed + a descriptive aria-label kept in sync.

   The sun/moon icon swap itself is pure CSS (see theme-toggle.html),
   so the correct icon shows on first paint with no flash. This script
   only manages state + accessibility.

   For zero flash of the wrong THEME, also inline the pre-paint snippet
   from theme-toggle.html in <head>; this file can load with defer.
   ============================================================ */
(() => {
    "use strict";

    const root = document.documentElement;
    const STORE_KEY = "corvid-theme";

    const systemDark = () => window.matchMedia("(prefers-color-scheme: dark)").matches;
    const isDark = () =>
        root.dataset.theme === "dark" || (!root.dataset.theme && systemDark());

    // Apply a stored or URL-forced choice (the pre-paint snippet may
    // have already done this; re-applying is idempotent).
    const urlTheme = new URLSearchParams(location.search).get("theme");
    let saved = urlTheme;
    if (!saved) {
        // localStorage access can throw in sandboxed iframes / disabled-storage
        // private modes — guard it so the toggle still initializes.
        try {
            saved = localStorage.getItem(STORE_KEY);
        } catch (_) {
            /* no stored preference available */
        }
    }
    if (saved === "dark" || saved === "light") {
        root.dataset.theme = saved;
    }

    const buttons = document.querySelectorAll("[data-corvid-theme-toggle]");

    const reflect = () => {
        const dark = isDark();
        buttons.forEach((btn) => {
            btn.setAttribute("aria-pressed", String(dark));
            btn.setAttribute(
                "aria-label",
                dark ? "Switch to light theme" : "Switch to dark theme"
            );
        });
    };

    buttons.forEach((btn) => {
        btn.addEventListener("click", () => {
            root.dataset.theme = isDark() ? "light" : "dark";
            try {
                localStorage.setItem(STORE_KEY, root.dataset.theme);
            } catch (_) {
                /* storage may be unavailable; the in-page choice still applies */
            }
            reflect();
        });
    });

    reflect();
})();
