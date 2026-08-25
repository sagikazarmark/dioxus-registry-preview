(() => {
  const initialize = () => {
    const switcher = document.querySelector(".rpv-theme-switcher");
    const manifest = document.querySelector('[data-switcher="theme"]');
    if (!switcher || !manifest) return;

    const visibleControls = switcher.querySelectorAll('input[type="radio"]');
    const manifestControls = manifest.querySelectorAll('input[data-value]');
    const themes = new Set(Array.from(visibleControls, (control) => control.value));
    const url = new URL(window.location.href);
    const requestedTheme = url.searchParams.get("theme");
    const preferredTheme = window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
    const initialTheme = themes.has(requestedTheme) ? requestedTheme : preferredTheme;
    const selectTheme = (theme) => {
      document.documentElement.dataset.theme = theme;
      visibleControls.forEach((control) => {
        control.checked = control.value === theme;
      });
      manifestControls.forEach((control) => {
        control.checked = control.dataset.value === theme;
      });
    };

    const addressForTheme = (theme) => {
      const nextUrl = new URL(window.location.href);
      nextUrl.searchParams.set("theme", theme);
      return nextUrl.toString();
    };

    selectTheme(initialTheme);
    if (!themes.has(requestedTheme)) {
      window.location.replace(addressForTheme(initialTheme));
      return;
    }

    switcher.addEventListener("change", (event) => {
      if (event.target instanceof HTMLInputElement && themes.has(event.target.value)) {
        selectTheme(event.target.value);
        window.location.assign(addressForTheme(event.target.value));
      }
    });
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initialize, { once: true });
  } else {
    requestAnimationFrame(initialize);
  }
})();
