import App from "./App.svelte";
import { mount } from "svelte";
import { initTheme } from "./lib/theme.js";
import { getPrefs } from "./lib/prefs.js";

initTheme(getPrefs().theme);

const app = mount(App, { target: document.getElementById("app") });
export default app;
