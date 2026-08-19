# De grafische schil

De schil van `dpo-fg-tool`: Svelte 5 en Vite in de systeemwebview van Tauri v2.
Waarom die keuze en wat er is afgevallen, staat in
[`../docs/RAAMWERK.md`](../docs/RAAMWERK.md); wat de schermen doen en waarom,
in [`../docs/SCHIL.md`](../docs/SCHIL.md).

## Draaien

```
pnpm install --frozen-lockfile
cargo tauri dev --config crates/dpofg-schil/tauri.conf.json   # vanuit de hoofdmap
```

Zonder de Tauri-cli kan de schil ook los worden bekeken. Er is dan geen kluis
en dus geen inhoud; de schil zegt dat ook, in plaats van een leeg scherm te
tonen:

```
pnpm dev
```

## Toetsen

```
pnpm exec svelte-check --tsconfig ./tsconfig.json   # typecontrole
pnpm exec vitest run                                # eenheids- en componenttests
pnpm exec vite build                                # de bundel
pnpm exec playwright test                           # de motortests
```

De motortests draaien tegen de gebouwde bundel op WebKit en Chromium. WebKit is
de belangrijkste: hij benadert zowel WKWebView op macOS als WebKitGTK op Linux,
en breekt daardoor al voordat er een macOS-bouw bestaat. Op een machine die de
systeembibliotheken van WebKit mist, meldt Playwright dat; installeer ze met
`sudo pnpm exec playwright install-deps`.

## Wat hier niet in mag

Geen CDN, geen weblettertypen, geen polyfilldienst: alles zit in de bundel. Geen
stijl in het element — het beleid staat `style-src 'self'` zonder
`'unsafe-inline'` toe en een bibliotheek die stijlen injecteert loopt daarop
stuk. Geen sleutels, geen cryptografie, geen bestands-invoer en geen klokken in
de webview; dat gebeurt allemaal in Rust. En geen `$state.snapshot`: dat roept
`structuredClone` aan zonder functiedetectie, en dat bestaat niet in Safari 15.0
terwijl dat de vloer van het bouwdoel is.

De bouwstraat loopt de gebouwde bundel na op elk van die punten. Die controle is
een vangnet en geen bewijs; het bewijs zijn de motortests.
