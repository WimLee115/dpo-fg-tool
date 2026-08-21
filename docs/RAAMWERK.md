# De keuze van het frontendraamwerk

*Besluit over de schil van dpo-fg-tool. Getoetst tegen `docs/PLATFORMONDERSTEUNING.md` §1 tot en met §11 en tegen de schermeisen in `docs/SCHIL.md`. Vandaag staat er een niet-vastgelegde opzet in `schil/`; dit besluit is wat die opzet legitimeert, niet andersom.*

---

## Het besluit

De schil wordt gebouwd met **Svelte 5 en Vite, zonder SvelteKit, zonder router en zonder componentbibliotheek**. De doorslag geeft niet de reactiviteit maar wat er in het uitgeleverde bestand terechtkomt: Svelte compileert alle sjablonen op bouwtijd, dus er is geen expressie-evaluator, geen `new Function` en geen inline script — en de stijlen worden bij het bouwen uit de componenten gehaald naar een statisch `.css`-bestand dat via `<link rel="stylesheet">` binnenkomt, dus `style-src 'self'` zonder `'unsafe-inline'` wordt niet met een truc overleefd maar structureel gehaald. Daarbovenop past het raamwerk zonder uitzondering in het gereedschap dat al vastligt: Vite is de eerstepartij-bouwketen, pnpm werkt, `vitest` met jsdom werkt, Playwright draait tegen de statische bouw. De twee alternatieven vallen niet af op smaak maar op feiten: de WebAssembly-route draait niet op de vastgelegde ondergrens, en het pleidooi voor "geen raamwerk" bouwt zijn belangrijkste belofte — focusbehoud bij het verhuizen van een regel tussen banden — op DOM-gedrag dat aantoonbaar niet bestaat. Wat we ervoor betalen staat verderop met naam en toenaam, en het is niet weinig.

---

## Waaraan het is getoetst

### De drie motoren en het bouwdoel (A, B)

De schil draait in de systeemwebview: WebView2 vanaf 110 op Windows, WKWebView uit macOS 12 op macOS, WebKitGTK 2.36 op Linux. Svelte levert statische HTML, JavaScript en CSS en veronderstelt geen motorversie. De gecompileerde uitvoer en de runtime leunen op `Proxy`, `Map`, `Set`, `WeakMap`, `Symbol` en `queueMicrotask`, alle ruim onder de ondergrens; de syntaxis wordt door het Vite-doel `['safari15','chrome110']` verlaagd, ook voor de code uit `node_modules`, dus ook voor de Svelte-runtime zelf. Er is één bundel voor drie motoren, want Svelte kent geen per-doel compilatiemodus.

Wat het bouwdoel **niet** dekt zijn runtime-API's: `target` verlaagt syntaxis, geen ontbrekende functies. Dat is bij deze keuze een blijvende controle en geen eenmalige toets. Eén geval is nu al bekend en gecontroleerd in de geïnstalleerde versie: `svelte/src/internal/shared/clone.js` roept op twee plaatsen `structuredClone` aan zonder functiedetectie, en dat is de code achter `$state.snapshot`. `structuredClone` bestaat pas vanaf Safari 15.4 terwijl het doel `safari15` is, en §2.2 van PLATFORMONDERSTEUNING.md verbiedt het symbool onvoorwaardelijk. De maatregel staat onder *De afspraken die erbij horen*: `$state.snapshot` wordt niet gebruikt, en de bouwstraat controleert dat het symbool niet in de bundel voorkomt.

De CSS gaat door dezelfde `cssTarget`-transformatie als de rest, dus native nesting en moderne kleurruimten worden op één plek verlaagd. `:has()` wordt niet verlaagd; daarom gebruiken we `:has()` niet en geven we de toestand als klasse aan de ouder mee. De PostCSS-uitwijk uit §2.2 blijft daarmee beschikbaar zonder dat we hem nodig hebben.

### Het contentbeveiligingsbeleid (C)

**`style-src 'self'` zonder `'unsafe-inline'` — het selectiecriterium.** In een productiebouw worden alle `<style>`-blokken uit alle componenten geëxtraheerd naar statische `.css`-bestanden; er is geen runtimepad dat een `<style>`-element in het document schiet. Svelte 5 gebruikt voor overgangen bovendien de Web Animations API in plaats van de geïnjecteerde `@keyframes`-stylesheet van Svelte 4 — precies de reden dat deze conclusie voor deze hoofdversie anders uitvalt dan voor de vorige.

Wat overblijft is één discipline: **nooit een `style`-attribuut in een sjabloon, ook niet statisch.** Dat is strenger dan het beleid technisch afdwingt. De reden om het toch zo te doen is dat de prozaregel bij §2.4 regel 177 ("geen stijlen in het element injecteren") niet technisch is afgebakend, dat het feitelijke browsergedrag afhangt van de route waarlangs het attribuut in de DOM komt, en dat dat gedrag per motor niet is geverifieerd. Onder A5 — de zwakste combinatie bepaalt het gedrag — is een regel die op alle drie de motoren hetzelfde doet meer waard dan een regel die op Chromium toevallig werkt. Dynamische positionering, zoals het naslagpaneel dat naast het veld opent, gaat via `el.style.setProperty` en wordt in één module geconcentreerd zodat die ene open vraag op één plek te toetsen is.

**`script-src 'self'` zonder `'unsafe-eval'`.** Dit is het punt waarop Svelte objectief het sterkst staat en waarop de meeste kandidaten sneuvelen. Alle sjablonen zijn vooraf gecompileerd, er is geen expressietaal in de browser, geen `eval`, geen `new Function`, geen WebAssembly en geen inline bootstrapscript. De HMR-runtime zit uitsluitend in de ontwikkelbouw.

**`connect-src 'none'`.** Kaal Svelte doet zelf geen enkele netwerkaanroep. Dat SvelteKit hier onmiddellijk zou vallen — clientnavigatie met loaders, payloads over `fetch`, form actions — is de reden dat SvelteKit niet alleen nu maar blijvend geweigerd wordt. Dynamische `import()` van eigen chunks valt onder `script-src` en blijft dus toegestaan: code mag gesplitst worden, gegevens niet. Er is één open punt dat elke kandidaat gelijk raakt en dat hieronder apart staat.

**De rest van het beleid.** Geen iframes, geen `<object>`, geen `<base href>` (er is geen router), geen native formulierverzending: formulieren zijn `onsubmit` met `preventDefault` en een aanroep over de brug. `blob:` komt nergens voor, dus geen `URL.createObjectURL` voor bijlagevoorbeelden; die komen als `data:`-URI door de brug. `assetsInlineLimit: 0` staat al in `vite.config.ts` en is geen cosmetische instelling: zonder die instelling lijnt Vite een klein lettertype als `data:`-URI in de geëxtraheerde CSS, en `font-src 'self'` staat `data:` niet toe. Iconen zijn inline SVG of SVG-bestanden uit de bundel.

### De overige Tauri-instellingen (D)

`freezePrototype: true` bevriest de prototypes van de ingebouwde objecten. Svelte 5's reactiviteit is op `Proxy` gebouwd en raakt `Object.prototype`, `Array.prototype` en `String.prototype` niet aan; er zijn geen polyfills, geen shims en geen monkey-patchende hulppakketten in de boom. De faalwijze die hier hoort — een `TypeError` bij het laden die alleen in de echte webview zichtbaar is — moet desondanks in de webview worden getoetst en niet in jsdom, want dat is de enige plek waar hij zich voordoet.

`dangerousDisableAssetCspModification: false` betekent dat er geen route is waarlangs een raamwerk zijn eigen, ruimere beleid meebrengt. Svelte vraagt daar niet om: geen nonce, geen eigen CSP, geen uitzondering.

De strikte capabilities per vensterlabel worden in de schil weerspiegeld door **twee bruggen, niet één**. Het persoonlijke FG-venster importeert de organisatiemodule domweg niet, zodat de scheiding uit `docs/SCHIL.md` regel 242 en 382 bovenop de platformgrens ook een bouwtijdgrens is.

### Wat niet in de webview mag (E)

Het merendeel hiervan zijn architectuurregels en geen raamwerkeigenschappen: geen cryptografie, geen sleutels, geen bestands-I/O, geen WebAuthn, geen PDF-opmaak, geen `Intl.DurationFormat`. Svelte brengt geen van die dingen mee, en de productieboom telt vandaag één pakket, dus er is ook niets dat het per ongeluk binnenhaalt.

Twee eisen verdienen aparte vermelding omdat ze de gebruikelijke reden om een componentbibliotheek te kiezen wegnemen. §2.2 verbiedt het native `<dialog>` en schrijft een eigen modale component voor; en de opmaak mag niet van de schuifbalkbreedte afhangen. Vrijwel elke volwassen UI-kit bouwt zijn dialoog op `showModal()` en compenseert de schuifbalk met `padding-right` in zijn scroll-lock, en beide zijn er niet uit te configureren. Een kit zou hier dus niet helpen maar tegenwerken. Dat verandert niets aan de kosten: de eigen modale component, de focusinsluiting bij het blokkadescherm, het herstelrapport en het time-outscherm, en het eigen contextmenu zijn en blijven handwerk.

Geen `setInterval`-gebaseerde tijdbewaking: alle tijd komt als gebeurtenis uit Rust en de schil schrijft weg wat hij krijgt. Dat is in het codevoorbeeld hieronder te zien.

Het blokkerende voorstartscherm vóór er een kluis bestaat, is bij deze keuze geen bijzonder geval: de eerste render vraagt geen gegevens, `PlatformReport` is de eerste toestand, en de dertien kluisloze schermen hebben geen hydratatie, geen store-initialisatie en geen router die eerst moet slagen.

### De testlagen (F)

`vitest` met jsdom werkt en staat al geconfigureerd; componenten monteren met `@testing-library/svelte`. De eerlijke kanttekening: de Svelte-kern beveelt tegenwoordig de browsermodus aan voor componenttests omdat de effectvolgorde en de microtask-flush in jsdom net anders lopen, en je hebt `flushSync()` nodig om synchroon te kunnen asserteren. jsdom is hier dus het pad dat bovenstrooms minder aandacht krijgt. Dat is een minpunt, geen blokkade.

De Playwright-laag — volgens §2.5 de belangrijkste — past goed: de frontend is statisch en start zonder Rust, dus `dist/` draait rechtstreeks onder `webkit` en `chromium` met een nagebootste brug. Dat de brug injecteerbaar moet zijn, is daarmee een ontwerpeis aan de datalaag geworden en niet aan de tests. Het residurisico ligt niet bij Svelte: de WebKit-bouw van Playwright is nieuwer dan Safari 15.0 en nieuwer dan WebKitGTK 2.36, dus deze laag is de belangrijkste maar niet de ondergrens.

Voor visuele regressie geldt één voorwaarde. Svelte scopet CSS met een gegenereerde klasse waarvan de hash uit de inhoud volgt; die is deterministisch per commit, maar niet stabiel over refactors. De `cssHash`-optie wordt daarom expliciet vastgezet op een leesbare, padonafhankelijke naam. Twee opeenvolgende bouwen leveren dan byte-identieke artefacten, wat de poort uit §10.3 haalt.

### Het vastliggende bouwgereedschap (G)

Vite is vastgelegd en `@sveltejs/vite-plugin-svelte` is de eerstepartij-integratie. pnpm met `--frozen-lockfile` werkt; Svelte declareert zijn afhankelijkheden netjes en heeft geen `postinstall` die iets van het net haalt. Node blijft bouwgereedschap en komt niet in het product, want er is geen server, geen adapter en geen server-side rendering. Reproduceerbaar bouwen is haalbaar met de `cssHash`-instelling, vaste bestandsnamen en een vastgezette minifierversie. Svelte heeft geen telemetrie, geen updatecontrole en geen foutrapportage, dus de netwerkstiltetest kan niet vallen op gedrag dat we niet kenden.

Bij de SBOM zit een val die we bewust omzeilen. `svelte` staat conventioneel in `devDependencies` terwijl zijn runtimecode wél in de bundel zit; `cyclonedx-npm --omit dev` zou dan een ondertekende SBOM opleveren die het belangrijkste uitgeleverde JavaScript niet noemt. Dat is erger dan een grote SBOM. `svelte` en `clsx` worden daarom als productieafhankelijkheid gedeclareerd. Of het SBOM-gereedschap zelf werkt, is een open punt dat hieronder staat.

---

## Wat er afvalt en waarom

### Een Rust-raamwerk naar WebAssembly (Leptos)

**Wat eraan goed was.** Dit is de enige optie waarin de zwaarste eis van het hele project door de compiler wordt gedragen: de uitklapcomponent die een verplicht kind weigert, wordt een nieuwtype in een componentsignatuur en dus een compilatiefout, niet een lint of een afspraak. Eén taal, één lockbestand, één auditgereedschap; `cargo vendor` legt volledige, leesbare broncode in de repository en `cargo deny` maakt van "geen netwerkbibliotheek in de graaf" een bouwpoort in plaats van een codereviewregel. Gedeelde typen tussen domein en schil laten een wijziging in het domein falen bij `cargo build` in plaats van bij een gebruiker. Codegeneratie uit het handelingsmanifest via `build.rs` sluit precies aan op de eis dat een schermdefinitie met een onbekende handeling de bouw laat falen. Het pleidooi noemt zijn eigen diskwalificerende voorwaarde ook zelf, en dat is meer dan de andere twee doen.

**Wat de doorslag gaf.** De optie draait niet op de vastgelegde ondergrens. Chromium weigert WebAssembly-compilatie zolang er een beleid is zonder `'wasm-unsafe-eval'`, en die bronexpressie bestaat in WebKit pas vanaf Safari 16. De ondergrens is macOS 12 met Safari 15 en het bouwdoel is letterlijk `safari15`. Op macOS rendert de applicatie dan niets. De enige token die daar nog werkt is `'unsafe-eval'`, en die heropent `eval`, `new Function` en `setTimeout` met een string — dat is geen verruiming van de regel maar het schrappen ervan. Het pleidooi presenteert de prijs als "één smalle CSP-regel" en heeft daarbij de risicoanalyse omgekeerd: Windows wordt als het probleem gepresenteerd en WebKit als veilig, terwijl het andersom ligt. Onder A5 — de zwakste combinatie bepaalt het gedrag — is dat beslissend en verder geen discussie waard.

Daar komt bovenop dat de teststrategie dit risico structureel niet kan vangen: Playwright levert een actuele WebKit-bouw die `'wasm-unsafe-eval'` wél kent, dus die laag rapporteert groen op precies het punt waar de ondergrens faalt. En de rekening loopt door: Vite, pnpm en de vitest/jsdom-laag vervallen alle drie, er blijft geen enkele geautomatiseerde componenttest op een WebKit-motor over, en debuggen op twee van de drie platformen wordt zonder bronkaarten en zonder breekpunten praktisch onmogelijk.

Twee argumenten die het pleidooi als structurele garantie opvoert, houden bovendien niet. `web-sys` gate't per WebIDL-interface en niet per methode: er bestaat geen feature `Window_setInterval`, en zodra `Window` aanstaat — en dat moet — is `set_interval` gewoon beschikbaar. Dat is dus geen sterkere garantie dan een codereviewregel. En het vlaggenschip `Optioneel<T>` heeft in het eigen voorbeeld een publiek veld dat met de hand wordt geconstrueerd op een lijst wettelijke verplichtingen; zonder privé-constructor in een gegenereerde module bewijst het type dat iemand iets heeft ingepakt, niet dat het manifest het onderdeel als optioneel registreert.

### Geen raamwerk: TypeScript, de DOM-API en Vite

**Wat eraan goed was.** Dit is het serieuste alternatief en het verdient een eerlijke behandeling. `style-src 'self'` wordt hier per constructie gehaald in plaats van per empirische toets op de gecompileerde uitvoer, per versie — en dat is het enige punt dat het brondocument zelf een selectiecriterium noemt. `freezePrototype` en de netwerkstiltetest zijn niet-gebeurtenissen omdat er geen derde partij is die op een prototype schrijft of ongevraagd verbinding zoekt. Er zijn geen gegenereerde klassenamen en geen runtime-geïnjecteerde stijlvolgorde, dus de twee grootste bronnen van niet-determinisme in de visuele regressietests en de reproduceerbaarheidspoort bestaan niet. Er is geen hoofdversie-migratie van de DOM-API, en over een doorlooptijd van jaren met één ontwikkelaar is dat het verschil tussen nul en minstens één periode waarin de testsuite onbetrouwbaar is. En het punt dat het brondocument zelf niet trekt: devtools-extensies zijn in geen van de drie webviews te installeren, dus een inspector die één op één op je eigen sjabloon staat is meer waard dan de ontwikkelervaring die een raamwerk in een gewone browser zou geven.

**Wat de doorslag gaf.** De dragende bewering onder het enige uitgewerkte codevoorbeeld is onjuist. `insertBefore` op een reeds gekoppelde knoop verplaatst hem niet zonder gevolgen: de DOM-specificatie verwijdert hem eerst van zijn oude ouder, en die verwijdering draait synchroon de focus-fixup uit de HTML-specificatie, waardoor `document.activeElement` naar `body` springt. Het opnieuw invoegen herstelt dat niet. Dat is precies de reden dat `Element.prototype.moveBefore()` als aparte API bestaat, en die is er vanaf Chrome 133 — niet in Chrome 110, niet in Safari 15, niet in WebKitGTK 2.36, dus onder B1 zonder uitweg. Omdat bandlidmaatschap continu wordt herrekend en een klok tijdens de focusmodus van band verhuist, is dit geen randgeval maar de normale gang van zaken bij een toetsenbord-eerst-interface. Het pleidooi voert de eigenschap op als bewezen en noemt "focus die na een herrender op body belandt" als de bugklasse die je hier juist níet krijgt.

Om eerlijk te zijn: dit probleem is niet exclusief. Een gesleutelde `{#each}` verplaatst knopen op dezelfde manier en verliest de focus even goed; het staat hieronder dan ook bij de open punten. Wat de doorslag geeft is niet dat de fout bestaat, maar dat de kolom "per constructie" van dit pleidooi op meer plaatsen berust op beweringen die niet houden. Het claimt dat er geen modale component nodig is terwijl §2.2 er letterlijk een voorschrijft, dus juist de duurste component wordt weggeredeneerd in plaats van weggenomen. Het claimt dat de API-ondergrens met lint afdwingbaar is, terwijl `arr.at`, `arr.findLast` en zelfs het uitdrukkelijk verboden `Intl.DurationFormat` ongemerkt door die controle gaan — prototypemethoden op onbekende typen zijn voor zulk gereedschap principieel niet te detecteren. Het claimt een productieboom en een SBOM van één regel, terwijl de dialoogplug-in verplicht is en dus meetelt. En de enige tegenmaatregel tegen de faalwijze die het pleidooi zelf als meest waarschijnlijk aanwijst — dat de eigen laag ongemerkt een raamwerk wordt — is een regelplafond dat op de ene plaats 400 en op de andere 300 heet en waarvan nooit wordt gezegd welke bestanden meetellen, terwijl het makkelijkste scherm van de hele schil er al 130 kost en er zestien werkbladen, een dossiervenster van zes secties, een ketenvenster, een springpalet en een type-ahead met duplicaatlijst achteraan staan.

Het bewijsmateriaal staat bovendien niet op de plek waar de twijfel zit. Uitgewerkt is de werkbak, het meest stringachtige en minst dynamische scherm van de schil. Van progressieve onthulling — de velddiff bij elke keuze, drie gelijktijdige aankondigingen, waardebehoud van verborgen velden, de altijd aanwezige buiten-beeldstrook — staat geen regel code, terwijl het pleidooi zelf zegt dat dáár de klap valt.

---

## Wat we opgeven

**De bouwtijdweigering van de uitklapcomponent kan niet in de component zitten.** Svelte-snippets zijn ondoorzichtige functies; een component kan zijn kind niet inspecteren. De weigering uit `docs/SCHIL.md` regel 258 moet dus in de generator worden afgedwongen, waar zij faalt bij het genereren in plaats van bij het compileren van het scherm. Dat is verdedigbaar en past bij regel 36 en 62, maar het is minder dan de Rust-optie bood, en het maakt de generator tot een harder stackvereiste dan de raamwerkkeuze zelf.

**C1 blijft een empirische toets in plaats van een eigenschap.** Bij elke Svelte-hoofdversie en bij elke wijziging in de compileruitvoer moet opnieuw op de gebouwde bundel worden gecontroleerd dat er geen stijlen in het element belanden. Bij de optie zonder raamwerk was er niets te toetsen. Dat verschil is echt en wordt niet met een lintregel weggenomen.

**Er komt een hoofdversie-migratie.** Svelte 3 naar 4 was triviaal, 4 naar 5 verving het reactiviteitsmodel: runes in plaats van `$:`, `onclick` in plaats van `on:click`, snippets in plaats van slots. Er is geen reden om aan te nemen dat er binnen de doorlooptijd van dit project geen tweede zo'n breuk komt. De enige echte verzachting is dat de generator het overgrote deel van de schermen produceert, zodat een migratie een wijziging aan één sjabloon plus hergeneratie wordt.

**Twee gereedschapsketens voor typecontrole.** `.svelte` is een eigen bestandsformaat; `tsc --noEmit` kan het project niet controleren, dat doet `svelte-check` — met een eigen hoofdversie, een eigen foutformaat en een eigen versie om vast te zetten. En de bronnen zijn over jaren alleen te bouwen met een compiler van precies die reeks.

**Een dun ecosysteem op precies de twee plekken waar het pijn doet.** Virtualisatie van lange lijsten bij vijfduizend registerregels, en toegankelijke primitieven voor de type-ahead met fuzzy duplicaatlijst, het springpalet, het eigen contextmenu en de weigering bij het veld. In de React-wereld bestaan daarvoor referentie-implementaties waar jaren aan schermlezermeldingen in zitten; hier bouwen en onderhouden we dat alleen, zonder vastgelegd toegankelijkheidsniveau dat de kwaliteit vasthoudt.

**Minder mensen die het kunnen overnemen.** De verzameling ontwikkelaars die Svelte 5-runes beheerst is een fractie van die voor React, en het beschikbare materiaal gaat nog jaren overwegend over Svelte 4 — met een syntaxis die compileert maar zich anders gedraagt.

**Een structureel gat tussen ontwikkelen en uitleveren.** `tauri dev` laadt de Vite-ontwikkelserver, die stijlen als `<style>`-tags injecteert en een websocket gebruikt. We ontwikkelen dus onder een ander beleid dan we uitleveren, uitgerekend bij de eis die het brondocument als selectiecriterium aanmerkt. Een `style`-fout is in de ontwikkelbouw niet te zien.

**De runtime kent functiegedetecteerde paden.** *(Nagerekend op 21-08-2026: `moveBefore()` komt in de geïnstalleerde Svelte 5.56.9 nergens voor. Er is dus geen functiegedetecteerd pad en geen per-motorverschil; de alinea hieronder beschrijft een eigenschap die dit raamwerk in deze versie niet heeft.)* Svelte 5 gebruikt `moveBefore()` waar de motor het heeft en valt anders terug op verwijderen-en-invoegen. Dat betekent per-motorgedrag in precies de operatie die focus en voorleespositie raakt, en dat is wat A5 wil uitsluiten. Het is geen breuk met het beleid, maar het is een gedragsverschil dat de motortests moeten bewaken en dat we niet hadden als er geen runtime was.

**De doorlichtbaarheidsclaim is beperkter dan zij klinkt.** De compileruitvoer van een Svelte-component is leesbaar, maar het releaseartefact wordt geminificeerd en er zijn geen bronkaarten. Een doorlichter leest dus de bron plus een reproduceerbare herbouw, niet het uitgeleverde bestand. Dat geldt overigens voor elke kandidaat en dus ook voor de optie zonder raamwerk, waar hetzelfde argument te sterk werd aangezet.

---

## Hoe het eruitziet

De werkbak, met de vaste bandindeling, de sporen en de permanente voetregel. Dit is echte Svelte 5, geen schets.

```svelte
<!-- src/schermen/Werkbak.svelte -->
<script lang="ts">
  import type { Werkbak, Tik } from '../soorten';
  import Regel from './Regel.svelte';
  import { roep, luister } from '../brug/organisatie';

  // Rust levert de volledige, al geaggregeerde werkbak. De schil rekent niet.
  let { werkbak }: { werkbak: Werkbak } = $props();

  // De enige clienttoestand op dit scherm: open of dicht.
  let overigOpen = $state(false);

  // Geen setInterval en geen setTimeout. Rust duwt de opgemaakte aftelling;
  // de schil schrijft weg wat hij krijgt.
  let aftelling = $state<Record<string, string>>({});
  $effect(() => luister<Tik>('klok:tik', (t) => { aftelling[t.sleutel] = t.tekst; }));
</script>

<section class="werkbak" aria-labelledby="wb-kop">
  <h1 id="wb-kop">Werkbak</h1>
  <!-- Opgemaakt in Rust met chrono-tz: "woensdag 19 augustus 2026, 09:14 CEST". -->
  <p class="peilmoment">{werkbak.nu}</p>

  <!-- Vastgezette strook bóven de banden. Altijd gerenderd, ook leeg. -->
  <section class="uitvoering" aria-label="In uitvoering">
    {#each werkbak.uitvoering as taak (taak.sleutel)}
      <p class="uitvoering-regel">
        <span class="taak">{taak.omschrijving}</span>
        <span class="venster">annuleren kan nog {aftelling[taak.sleutel] ?? taak.venster}</span>
        <button type="button" onclick={() => roep({ naam: 'taak.annuleer', sleutel: taak.sleutel })}>
          Annuleren
        </button>
      </p>
    {:else}
      <p class="uitvoering-leeg">Niets in uitvoering.</p>
    {/each}
  </section>

  <!-- De bandvolgorde komt in volgorde uit Rust en is niet om te draaien.
       Er is geen sorteertoestand, geen kolomkop en geen kolomconfiguratie. -->
  {#each werkbak.banden as band (band.code)}
    {@const dicht = band.inklapbaar && !overigOpen}
    <section class="band" data-band={band.code} aria-labelledby="kop-{band.code}">
      <h2 id="kop-{band.code}" class="bandkop">
        <span class="bandtitel">{band.kop}</span>
        <!-- De telling komt uit Rust, ook wanneer de rijen niet zijn geleverd. -->
        <span class="telling">{band.telling}</span>
      </h2>

      {#if band.inklapbaar}
        <button type="button" aria-expanded={!dicht} onclick={() => (overigOpen = !overigOpen)}>
          {dicht ? 'Tonen' : 'Inklappen'}
        </button>
      {/if}

      {#if !dicht}
        <ul class="regels">
          {#each band.regels as regel (regel.sleutel)}
            <li><Regel {regel} /></li>
          {/each}
        </ul>
      {/if}
    </section>
  {/each}

  <!-- Permanent en niet in te klappen, dus altijd in de toegankelijkheidsboom.
       Elke telling is een knop, want er bestaat geen getal zonder route naar de rijen. -->
  <footer class="voetregel">
    {#each werkbak.voet as teller (teller.code)}
      <button type="button" onclick={() => roep({ naam: 'werkbak.filter', code: teller.code })}>
        {teller.tekst}
      </button>
    {/each}
  </footer>
</section>

<style>
  /* Wordt bij het bouwen geëxtraheerd naar een .css-bestand en via
     <link rel="stylesheet"> geladen. Geen runtime-injectie, geen style-attribuut. */
  .band { border-top: 2px solid var(--lijn); padding-block: 0.75rem; }
  .bandkop { display: flex; gap: 0.5rem; align-items: baseline; font-size: 1rem; }
  .telling { font-variant-numeric: tabular-nums; }
  .regels { list-style: none; margin: 0; padding: 0; }
  .voetregel { border-top: 3px double var(--lijn); padding-block: 0.75rem; }
</style>
```

En één regel, met alle feiten inline — geen tooltip, geen hover, één knop:

```svelte
<!-- src/schermen/Regel.svelte -->
<script lang="ts">
  import type { Verplichting } from '../soorten';
  import { roep } from '../brug/organisatie';
  let { regel }: { regel: Verplichting } = $props();
</script>

<article class="regel" class:onomkeerbaar={regel.onomkeerbaar}>
  <p class="kop">
    <span class="dossier">{regel.dossier}</span>
    <span class="titel">{regel.titel}</span>
    <span class="spoor">spoor {regel.spoor} van {regel.sporen}</span>
  </p>
  <!-- Absoluut, met tijdzone, opgemaakt in Rust. Resterende tijd hooguit erbij. -->
  <p class="deadline"><time datetime={regel.deadlineIso}>{regel.deadlineTekst}</time></p>
  <p class="grondslag">{regel.grondslag}</p>
  <p class="rekenregel">{regel.rekenregel} — anker {regel.ankerTekst}</p>
  <p class="eigenaar">{regel.eigenaarsrol} — {regel.bezetting}</p>

  <!-- Absentie in plaats van grijs-uitgeschakeld: een knop die pas na een
       beoordeling betekenis heeft, bestaat vóór die beoordeling niet. -->
  {#if regel.actie}
    {@const actie = regel.actie}
    <p class="actie">
      <button type="button" onclick={() => roep({ naam: actie.handeling, sleutel: regel.sleutel })}>
        {actie.label}
      </button>
    </p>
  {/if}
</article>

<style>
  .regel { display: grid; gap: 0.15rem; padding-block: 0.5rem; }
  .kop { display: flex; gap: 0.5rem; flex-wrap: wrap; }
  /* Onomkeerbaar heeft vorm en niet alleen tint: volle breedte, dubbele omlijning. */
  .onomkeerbaar { border: 3px double var(--rand-onomkeerbaar); padding: 0.5rem; }
</style>
```

Drie dingen aan deze code die het besluit dragen. Er staat nergens een `style`-attribuut, ook niet statisch; vorm loopt over `class` en `class:`. De bandtinten staan bewust niet in het `<style>`-blok hierboven maar in de basisstylesheet met `[data-band="…"]`-selectors — niet omdat de snoeier ze zou weghalen (die is juist conservatief bij dynamische waarden), maar omdat componentgescopete stijlen de markup van een kindcomponent nooit bereiken en de bandvorm op meerdere schermen terugkomt. En er is geen store, geen router, geen datalaagbibliotheek en geen UI-kit: de brug is de enige route naar buiten.

---

## De afspraken die erbij horen

**Afhankelijkheden.** In productie staan `@tauri-apps/api`, `svelte` en `clsx` — de laatste twee uitdrukkelijk als productieafhankelijkheid, tegen de conventie in, omdat hun code in de bundel zit en de ondertekende SBOM anders het belangrijkste uitgeleverde JavaScript verzwijgt. Geen SvelteKit, nu niet en later niet. Geen router uit het ecosysteem: schermwisseling is toestand, geen adres. Geen componentbibliotheek, geen CSS-in-JS, geen atomaire-CSS-generator, geen netwerkbibliotheek, geen datum- of duurbibliotheek, geen cryptografie, geen export- of PDF-generator, geen klembordhulp. Elke nieuwe productieafhankelijkheid is een besluit met een reden in de commit, niet een installatie.

**Bouwgereedschap.** Vite met het doel dat al in `vite.config.ts` staat: `target` en `cssTarget` op `['safari15','chrome110']`, `modulePreload.polyfill` uit, `sourcemap` uit, `assetsInlineLimit` op nul. pnpm met `--frozen-lockfile` en het lockbestand in de repository. Node vastgezet via `.node-version`, uitsluitend als bouwgereedschap. `cssHash` expliciet vastgezet op een leesbare, padonafhankelijke naam, vaste bestandsnamen waar dat kan, een vastgezette minifierversie, en een bouwtest die twee bouwen van dezelfde commit byte voor byte vergelijkt. PostCSS blijft in de keten.

**Regels in de code.** Nooit een `style`-attribuut in een sjabloon; dynamische stijl uitsluitend via de `style:`-directive of via `setProperty` in één daarvoor aangewezen module. `{@html}` is verboden zonder uitzondering. `$state.snapshot` wordt niet gebruikt, want de runtime roept daar `structuredClone` aan zonder functiedetectie. Geen `setInterval` en geen `setTimeout` voor tijdbewaking; aftellingen zijn weergave van wat Rust stuurt. Elk getal en elk label op het scherm komt uit Rust of uit het handelingsmanifest, niet uit de DOM en niet uit schermtekst. De brug is per venster een aparte module met een eindige, getypeerde commandolijst zonder jokertekens, en één injectiepunt zodat de motortests hem kunnen vervangen.

**Testen.** `vitest` met jsdom voor logica en componenteenheden, met `flushSync()` waar synchroon geasserteerd wordt. Playwright met `webkit` en `chromium` tegen `dist/` met een nagebootste brug — dit is de laag waarop een pull request breekt. Referentiebeelden per motor, niet per platform. `tauri-driver` op Linux en Windows; macOS krijgt het handmatige draaiboek van tien stappen. De nachtelijke controle tegen Edge Beta en Dev blijft staan.

**Poorten bij het uitgeven.** De netwerkstiltetest met onderscheppende proxy: nul pakketten, nul DNS-verzoeken. Een grep over `dist/` op `eval(`, `new Function(`, `fetch(`, `XMLHttpRequest`, `WebSocket`, `EventSource`, `WebAssembly`, `setInterval`, `createObjectURL`, `structuredClone` en `{@html}`-uitvoer, plus op `url(data:` en `@import` in de CSS. Die poort is uitdrukkelijk een vangnet en geen bewijs: Svelte zet attributen via eigen hulpfuncties en spread-props lopen langs een berekende attribuutnaam, dus een `style` kan er langs. Het bewijs voor C1 is de motortest op de gebouwde bundel, niet de grep.

**Wat niet in de bundel mag.** Geen weblettertypen en geen als `data:`-URI ingelijnde lettertypen. Geen icoonfont. Geen externe afbeelding en geen `blob:`-URL. Geen web worker als blob. Geen bronkaarten. Geen ontwikkelaarsruntime, geen HMR-code, geen `$inspect`-uitvoer. Geen polyfill en geen shim die op een ingebouwd prototype schrijft. Geen telemetrie, geen updatecontrole, geen foutrapportage.

---

## Wanneer dit besluit wordt herzien

Bij vijf omstandigheden, elk concreet genoeg om vast te stellen.

**Als de C1-toets op de echte motoren faalt.** Zodra de gebouwde bundel op WebKitGTK 2.36 of op WKWebView uit macOS 12 aantoonbaar stijlen in het element zet, of zodra dat gedrag per motor verschilt, is A5 geschonden en vervalt de grond onder deze keuze. Dat is de eerste meting die gedaan wordt, vóór er schermen worden gebouwd.

**Als de meting op vijfduizend registerregels laat zien dat de werkbak niet per minuut te hertekenen is** en virtualisatie nodig blijkt terwijl er geen onderhouden adapter is. Die meting hoort vóór de derde stap, niet erna.

**Als er een tweede ontwikkelaar bijkomt of de opvolging binnen een half jaar geregeld moet zijn.** Dan weegt de omvang van de groep die het gereedschap kent zwaarder dan de eigenschappen van het gereedschap, en verschuift het gewicht naar het grootste ecosysteem.

**Als er een tweede hoofdversiebreuk komt terwijl de generator nog niet het merendeel van de schermen produceert.** Dan is de migratie handwerk op honderd bestanden in plaats van op één sjabloon, en is de rekening hoger dan wat deze keuze opbrengt.

**Als er een formeel toegankelijkheidsniveau wordt geëist** — in een aanbesteding, bij een externe audit, of omdat een gebruiker met een schermlezer vaste doelgroep wordt. Dan is de tijd die getoetste primitieven besparen op combobox, menu en aankondigingsgedrag reëel, en weegt die op tegen de toets die we per component moeten doen.

Voor de duidelijkheid: de WebAssembly-route komt alleen terug in beeld als de ondergrens uit §1.3 verschuift naar macOS 13 of hoger. Zolang macOS 12 de grens is, is die optie niet marginaal maar onbruikbaar.

---

## Wat nog niet vaststaat

**Of `connect-src 'none'` samengaat met de IPC van Tauri v2.** Dit is het belangrijkste open punt en het raakt elke kandidaat gelijk. Tauri v2 wikkelt `invoke` op Windows af over een eigen protocol, en het doccommentaar in `@tauri-apps/api/core.js` noemt als voorbeeldwaarde een beleid met `ipc: http://ipc.localhost` erin. Als dat hier klopt, valt niet het raamwerk om maar het beleid uit §2.4, en dan moet dat besluit apart worden genomen en vastgelegd. Dit wordt gemeten voordat er iets anders wordt gebouwd.

**Of de prozaregel bij `style-src` de CSSOM-route toestaat.** Het document beslecht dat niet. Wij houden de strengste lezing aan, maar het naslagpaneel dat naast het veld opent en het springpalet vragen om dynamische positionering, en daar is `setProperty` de enige route. De vraag wordt daarmee uitgesteld, niet vermeden.

**Of de SBOM-route werkt.** `pnpm dlx @cyclonedx/cyclonedx-npm --omit dev` roept intern `npm ls` aan, en dat leest een pnpm-symlinkboom niet. G2 en G7 botsen dus op gereedschapsniveau, ongeacht de raamwerkkeuze. Er moet een werkende route komen — een andere generator, of de SBOM afleiden uit de bundel-importgraaf in plaats van uit `package.json` — voordat de eerste ondertekende uitgave zin heeft.

**Focus en voorleespositie bij een bandwissel.** Een regel die tijdens gebruik van band verhuist, wordt in elke DOM-aanpak verwijderd en opnieuw ingevoegd, en daarmee verliest hij de focus. `moveBefore()` lost dat op vanaf Chrome 133 en bestaat niet in WebKit; Svelte gebruikt dat pad functiegedetecteerd, wat per motor verschillend gedrag geeft. Dit heeft een eigen meting en waarschijnlijk expliciet focusherstel nodig. Het is nu niet opgelost en het moet niet als opgelost worden gepresenteerd.

**De schaal van de werkbak.** Vijfduizend registerregels, records één voor één gelezen als losse envelop: ongemeten. Daarvan hangt af of de werkbak per minuut volledig herrekend kan worden of incrementeel moet, en dat bepaalt weer of virtualisatie nodig is.

**Twee vensters in de bouw.** Twee kluizen zijn twee vensters met eigen capabilities. Dat betekent twee HTML-ingangen in Vite en een gedeelde chunk die de grens tussen organisatiecode en FG-code moet respecteren. Hoe die splitsing eruitziet, en hoe de reproduceerbaarheidspoort en de referentiebeelden zich daartoe verhouden, is niet uitgewerkt.

**De weergavekant van kopiëren en afdrukken.** Er zijn vier afdrukpaden, een kopieerknop per veld in de meldtekstopbouw, de exacte opdrachtregelaanroep met kopieerknop, en een klembord dat na dertig seconden wordt gewist. Al die handelingen lopen over de brug omdat klembordbewerkingen met gevoelige inhoud in Rust horen, en er staat vandaag geen enkel klembord- of exportcommando in de commandolijst.

**De orkestratie van de live-regio's.** Bij de velddiff moeten drie mededelingen tegelijk aankomen, de bandwissel tijdens de focusmodus krijgt een eigen strook, en de weigering verschijnt bij het veld met focusverplaatsing. Tegelijk is het budget vijf onderbrekingen per week, dus te vaak aankondigen is hier een defect en geen hinder. Hoe dat wordt gemodelleerd, en hoe het per motor wordt getest terwijl macOS alleen een handmatig draaiboek heeft, staat nog open.

**De versies van het gereedschap zelf.** Het document legt Vite, vitest, Playwright en `svelte-check` nergens vast; alleen "vaste minifierversie" staat er. Voor een keten waarin vier pakketten in de pas moeten lopen, is dat te los en hoort er een aparte vastlegging te komen.
