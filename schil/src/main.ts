import { mount } from 'svelte';
import App from './App.svelte';
import './opmaak.css';
import { inWebview, tauribrug, zetBrug } from './brug';

// De brug wordt pas gezet als de schil in een webview draait. Zonder webview
// start de schil wél, maar zegt hij dat hij niets kan opvragen — dat is
// duidelijker dan een leeg scherm.
async function start() {
  if (inWebview()) {
    zetBrug(await tauribrug());
  }
  const doel = document.getElementById('schil');
  if (!doel) throw new Error('het element #schil ontbreekt in index.html');
  mount(App, { target: doel });
}

void start();
