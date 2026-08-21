# Platformondersteuning: Linux, macOS en Windows

`dpo-fg-tool` is één codebasis met drie zeer verschillende uitvoeringsomgevingen. De backend is Rust, de gebruikersinterface draait in de systeemwebview via Tauri v2, en de opslag is een versleutelde SQLite-database. Dat betekent dat vrijwel elke beveiligingsaanname — waar de sleutel staat, of een bestand vergrendeld kan worden, of een handtekening controleerbaar is — per besturingssysteem anders wordt ingevuld.

Dit hoofdstuk legt die verschillen vast als ontwerpbeslissing, niet als implementatiedetail. Het uitgangspunt is telkens hetzelfde: **de zwakste ondersteunde combinatie bepaalt het gedrag van het product.** Wat op één platform niet veilig kan, wordt op geen enkel platform stilzwijgend anders gedaan.

---

## 1. Ondersteunde versies

### 1.1 Ondersteuningsniveaus

| Niveau | Betekenis |
|---|---|
| **Ondersteund** | Wordt gebouwd, getest in de bouwstraat, en fouten hierop blokkeren een release. |
| **Best effort** | Wordt gebouwd maar niet volledig getest; fouten worden aangenomen maar blokkeren geen release. |
| **Niet ondersteund** | Start bewust niet, met een leesbare melding en een verwijzing naar de systeemeisen. |

### 1.2 Windows

| Item | Eis |
|---|---|
| Minimale versie | Windows 10 versie 21H2 (build 19044), x64 |
| Aanbevolen | Windows 11 23H2 of nieuwer |
| Architectuur | x64 ondersteund, arm64 best effort |
| Webview | Microsoft Edge WebView2 Runtime (Evergreen), minimaal versie 110 |
| Niet ondersteund | Windows 8.1 en ouder, alle 32-bits varianten, Windows Server Core zonder desktopervaring |

**Waarom deze ondergrens.** WebView2 is de beveiligingsgrens van de gebruikersinterface. Microsoft levert de Evergreen-runtime alleen nog bij op Windows-versies die zelf ondersteuning krijgen. Op een oudere Windows loopt de webview achter op beveiligingsupdates, en dat is voor een applicatie die het volledige kwetsbaarhedenoverzicht van een organisatie beheert onaanvaardbaar. Windows 10 21H2 is bovendien de eerste build waarin de langpadondersteuning (zie [§6.2](#62-padlengte-op-windows)) betrouwbaar via het applicatiemanifest te activeren is.

Windows Server (2019 en 2022) is technisch bruikbaar wanneer de Desktop Experience is geïnstalleerd, en wordt als *best effort* behandeld voor de gedeelde serveropstelling. Server Core valt af: geen webview, geen desktopsessie.

### 1.3 macOS

| Item | Eis |
|---|---|
| Minimale versie | macOS 12 Monterey |
| Aanbevolen | macOS 14 Sonoma of nieuwer |
| Architectuur | Universal 2 (x86_64 + aarch64) in één bundel |
| Webview | WKWebView, meegeleverd met het besturingssysteem |
| Niet ondersteund | macOS 11 Big Sur en ouder |

**Waarom deze ondergrens.** Tauri v2 zelf gaat verder terug, maar de WebKit-versie in macOS 11 mist te veel van de CSS- en JavaScript-basis die de rest van de doelgroep wel heeft (zie [§2.4](#24-de-gemeenschappelijke-basis)), en Apple levert er geen Safari-updates meer voor. macOS 12 is de eerste versie waarin de gedeelde basis samenvalt met die van WebKitGTK 2.36 op Linux, waardoor één frontend-build voor alle drie de platformen realistisch blijft.

### 1.4 Linux

De bepalende factoren op Linux zijn drie: de glibc-versie (die de ondergrens van het binaire bestand vastlegt), de WebKitGTK-ABI (Tauri v2 vereist `webkit2gtk-4.1`, niet `4.0`) en de aanwezigheid van een Secret Service-implementatie.

| Item | Eis |
|---|---|
| glibc | 2.35 of nieuwer (bouwbasis: Ubuntu 22.04 LTS) |
| WebKitGTK | `libwebkit2gtk-4.1` versie 2.36 of nieuwer |
| GTK | 3.24 of nieuwer |
| Architectuur | x86_64 ondersteund, aarch64 best effort |
| Grafische stack | X11 en Wayland (via GTK), beide ondersteund |
| Niet ondersteund | musl-gebaseerde distributies (Alpine), distributies met alleen `webkit2gtk-4.0` |

| Distributie | Versie | glibc | WebKitGTK 4.1 | Status |
|---|---|---|---|---|
| Ubuntu | 22.04 LTS | 2.35 | ja | Ondersteund (bouwbasis) |
| Ubuntu | 24.04 LTS | 2.39 | ja | Ondersteund |
| Debian | 12 Bookworm | 2.36 | ja | Ondersteund |
| Debian | 11 Bullseye | 2.31 | nee (alleen 4.0) | Niet ondersteund |
| Fedora | 40 en nieuwer | 2.39+ | ja | Ondersteund |
| RHEL / Rocky / AlmaLinux | 9.4 en nieuwer | 2.34 | ja (`webkit2gtk4.1`) | Ondersteund via de `.rpm` |
| openSUSE Leap | 15.6 | 2.38 | ja | Best effort |
| Arch, Manjaro, EndeavourOS | rolling | actueel | ja | Best effort |
| Alpine | alle | musl | n.v.t. | Niet ondersteund |

> **Waarom RHEL 9.4 hier lager staat dan de ondergrens.** glibc is voorwaarts compatibel en niet achterwaarts: een binair bestand gebouwd tegen 2.35 start niet op 2.34. Deze distributies draaien de AppImage dus niet, en zijn toch ondersteund — via de `.rpm` uit een eigen container, zoals hieronder en in §7.3 en §10.1 staat beschreven. De ondergrens van 2.35 geldt voor de bouwbasis van de AppImage, niet voor elk uitgeleverd pakket.

**Waarom de bouwbasis Ubuntu 22.04 is.** Een binair bestand gebouwd tegen glibc 2.35 draait op elke distributie met glibc 2.35 of hoger; andersom niet. De AppImage en de tarball worden daarom gebouwd in een container op basis van Ubuntu 22.04, ook wanneer de bouwstraat zelf op een nieuwere runner draait. De `.deb` wordt gebouwd op 22.04, de `.rpm` in een Rocky 9-container, zodat de afhankelijkheidsnamen per pakketfamilie kloppen.

RHEL 9 heeft glibc 2.34, dus lager dan onze bouwbasis. Daarom krijgt RHEL een eigen `.rpm` uit een eigen container in plaats van de AppImage.

### 1.5 Toolchain

| Item | Vastlegging |
|---|---|
| Rust | Vastgezet in `rust-toolchain.toml`, minimaal 1.82, exacte versie per release vastgelegd |
| Tauri | v2, exacte versie vastgezet in `Cargo.lock` |
| Node | LTS, vastgezet via `.node-version`, alleen als bouwgereedschap (niet in het product) |
| Pakketbeheerder frontend | `pnpm` met `--frozen-lockfile` |

### 1.6 Controle bij het opstarten

De applicatie controleert bij de start of de omgeving voldoet en weigert netjes wanneer dat niet zo is. De controle levert een `PlatformReport` op dat zichtbaar is onder **Help → Systeemcontrole** en dat integraal in het auditspoor wordt vastgelegd bij elke keer dat de kluis wordt geopend.

```rust
pub struct PlatformReport {
    pub os: OsKind,                 // Windows { build }, MacOs { major, minor }, Linux { distro, glibc }
    pub webview: WebviewInfo,       // engine, versie, ondersteund ja/nee
    pub secret_backend: BackendId,  // welke sleutelopslag daadwerkelijk actief is
    pub token_support: TokenSupport,// FIDO2, PIV, platformauthenticator
    pub storage: StorageInfo,       // pad, bestandssysteem, lokaal of netwerk, hoofdlettergevoelig
    pub portable_mode: bool,
    pub warnings: Vec<PlatformWarning>,
}
```

Dit rapport is niet alleen diagnostiek. Voor de verantwoordingsplicht moet later aantoonbaar zijn onder welke omstandigheden een dossier is bewerkt: met welke sleutelopslag, in draagbare modus of niet, op een lokale schijf of een netwerkschijf.

---

## 2. Webview-verschillen

### 2.1 Drie motoren, drie updateritmes

| | Windows | macOS | Linux |
|---|---|---|---|
| Motor | WebView2 (Chromium/Blink) | WKWebView (WebKit) | WebKitGTK (WebKit) |
| Versie gekoppeld aan | Edge-installatie, werkt zelfstandig bij | Het besturingssysteem | Het distributiepakket |
| Updatetempo | Elke vier weken, buiten onze controle | Bij OS-updates, twee tot vier keer per jaar | Volgt de distributie, soms maanden achter |
| Ontwikkelaarsgereedschap | Volledige Edge DevTools | Safari Web Inspector (aan te zetten) | Web Inspector, alleen in debug-build |
| Grootste risico | Een browserupdate breekt de interface zonder dat wij iets wijzigen | Oude WebKit op oude macOS | Grote spreiding in versies, bekende renderfouten |
| Aansturing | Custom protocol `http://tauri.localhost` | Custom protocol `tauri://localhost` | Custom protocol `tauri://localhost` |

Het asymmetrische risico is Windows: daar kan de interface stukgaan door een update die wij niet hebben veroorzaakt en niet kunnen tegenhouden. Op Linux is het risico het spiegelbeeld: een te oude motor bij een gebruiker die zijn distributie niet bijwerkt.

### 2.2 Concrete verschillen waar de interface op sneuvelt

| Onderwerp | WebView2 | WKWebView (macOS 12+) | WebKitGTK 2.36–2.44 | Aanpak |
|---|---|---|---|---|
| `<dialog>` | Ja | Vanaf Safari 15.4 | Wisselend, oudere builds onvolledig | Eigen modale component, geen `<dialog>` |
| Container queries | Ja | Vanaf macOS 13 | Vanaf 2.38 | Niet gebruiken; media queries en grid |
| `:has()` | Ja | Vanaf 15.4 | Vanaf 2.38 | Alleen met PostCSS-uitwijk |
| `structuredClone` | Ja | Vanaf 15.4 | Vanaf 2.34 | **Niet toegestaan**: de baan "de bundel nalopen" breekt de bouw af zodra het woord in `dist/assets/*.js` staat |
| `Intl.DurationFormat` | Recent | Recent | Nee | Niet gebruiken; termijnteksten in Rust opmaken |
| `crypto.subtle` | Alleen in secure context | Idem | Idem | Geen cryptografie in de webview, punt |
| Bestandskiezer (`showOpenFilePicker`) | Ja | Nee | Nee | Altijd de Tauri-dialoogplug-in |
| Slepen en neerzetten van bestanden | Via Tauri-gebeurtenis | Via Tauri-gebeurtenis | Onbetrouwbaar bij sommige bestandsbeheerders | Slepen is nooit de enige route |
| Contextmenu | Uit te schakelen | Uit te schakelen | Toont eigen WebKit-menu | Expliciet onderdrukken en eigen menu tonen |
| Afdrukken naar PDF | Chromium-opmaak | WebKit-opmaak | Vaak afwijkend of ontbrekend | PDF's worden in Rust gerenderd, niet in de webview |
| Lettertypen | Systeemstack | Systeemstack | Sterk afhankelijk van fontconfig | Lettertypen worden meegeleverd, geen systeemstack |
| Schuifbalken | Overlay | Overlay | Klassiek, neemt ruimte | Layout mag niet afhangen van schuifbalkbreedte |
| WebAuthn in de webview | Beperkt | Vereist gekoppeld domein | Afwezig | Tokens worden native in Rust aangesproken (zie [§4](#4-hardwaretokens)) |

### 2.3 Bekende platformkwalen en hun mitigatie

**Linux, leeg venster bij het starten.** WebKitGTK 2.42 en nieuwer gebruikt standaard een DMA-BUF-renderer die op een aantal drivercombinaties (met name NVIDIA-eigen drivers en sommige VM-grafische stacks) een volledig zwart of leeg venster oplevert. De applicatie detecteert dit bij het opstarten en zet in dat geval `WEBKIT_DISABLE_DMABUF_RENDERER=1` voordat de webview wordt aangemaakt, met een regel in het logboek. De start-scripts in `.deb`, `.rpm` en de AppImage bevatten dezelfde uitwijk, en de instelling is met een vlag te overrulen.

**Linux, transparantie en samenstelling.** Vensters met transparantie of afgeronde hoeken zijn op sommige samenstellers onbetrouwbaar. Het ontwerp gebruikt daarom een dekkend venster met standaard vensterdecoratie; geen aangepaste titelbalk.

**Windows, WebView2 ontbreekt.** De runtime is aanwezig op Windows 11 en op vrijwel elke bijgewerkte Windows 10, maar niet gegarandeerd. De omgang hiermee zit in het installatieprogramma en niet in de applicatie (zie [§7.1](#71-windows)).

**macOS, achtergrondbeperking.** WKWebView schort timers op in een achtergrondvenster. Termijnbewaking en de vergrendelingstimer van de kluis draaien daarom volledig in Rust, met een `tokio`-taak, en niet met `setInterval` in de webview.

### 2.4 De gemeenschappelijke basis

De frontend wordt gebouwd tegen één doel, niet per platform:

```js
// vite.config.ts
build: {
  target: ['safari15', 'chrome110'],   // laagste gemene deler van de ondersteunde matrix
  cssTarget: ['safari15', 'chrome110'],
  modulePreload: { polyfill: false },
  sourcemap: false,                     // geen bronkaarten in de release
}
```

Er wordt bewust **één** bundel gemaakt voor alle drie de platformen. Een per-platform doelversie zou betekenen dat een fout op macOS pas bij een macOS-build zichtbaar wordt. Met één doel is het gedrag op de drie motoren zo gelijk mogelijk, en vangt de Linux-bouw fouten af die anders pas bij een gebruiker opduiken.

Aanvullende regels:

- **Geen externe bronnen.** Geen CDN, geen weblettertypen, geen polyfillservice. Alles zit in de bundel. Dit is tegelijk een beveiligingseis (zie [§9](#9-aantoonbaar-geen-uitgaand-netwerkverkeer)).
- **Geen cryptografie, geen sleutels, geen bestands-I/O in de webview.** De webview is een weergavelaag. Sleutels, ontsleuteling, exportgeneratie, PDF-opmaak en klembordbewerkingen met gevoelige inhoud gebeuren in Rust. Daarmee vervalt een hele klasse platformverschillen, en het beperkt de schade van een fout in de interface.
- **Strikt contentbeveiligingsbeleid**, vastgelegd in `tauri.conf.json`:

```json
"security": {
  "csp": "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src 'none'; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'",
  "dangerousDisableAssetCspModification": false,
  "freezePrototype": true,
  "assetProtocol": { "enable": false }
}
```

Let op: `style-src 'self'` zonder `'unsafe-inline'` betekent dat de gekozen UI-bibliotheek geen stijlen in het element mag injecteren. Dat is een selectiecriterium voor de frontend, geen detail achteraf.

### 2.5 Zo voorkom je dat de interface op één platform stukloopt

| Laag | Gereedschap | Draait op | Vangt |
|---|---|---|---|
| Componenttests | `vitest` met jsdom | Linux | Logica, geen renderfouten |
| Motortests | Playwright, engines `webkit` en `chromium`, tegen de gebouwde frontend met een nagebootste IPC-brug | Linux (beide), macOS (webkit), Windows (chromium) | Het overgrote deel van de motorverschillen, zonder de volledige applicatie te bouwen |
| Visuele regressie | Playwright `toHaveScreenshot`, aparte referentiebeelden per motor | Idem | Opmaakverschuivingen |
| Volledige e2e | `tauri-driver` | Linux (`WebKitWebDriver` onder `xvfb`), Windows (`msedgedriver`) | Echte IPC, echte vensters |
| Handmatige rooktest | Draaiboek van tien stappen | macOS | Wat op macOS niet te automatiseren is (zie [§11](#11-bekende-beperkingen-per-platform)) |

De Playwright-laag is de belangrijkste: `webkit` benadert zowel WKWebView als WebKitGTK dicht genoeg om vrijwel alle opmaak- en API-verschillen te vinden, en draait op de Linux-runner. Daardoor breekt een pull request al voordat er een macOS-build bestaat.

Daarnaast draait bij elke nachtelijke bouw een controle tegen **Edge Beta en Edge Dev** op Windows. Dat geeft vier tot zes weken voorsprong op een WebView2-update die de interface zou breken.

---

## 3. Sleutel- en geheimenopslag

### 3.1 Wat er precies wordt opgeslagen

Dit is de kern van het ontwerp, en het is belangrijk om het exact te formuleren.

De database is versleuteld met een **datasleutel** (DEK) van 32 bytes, die willekeurig wordt gegenereerd bij het aanmaken van de kluis en nooit verandert. Die datasleutel wordt versleuteld opgeslagen met een **sleutelversleutelende sleutel** (KEK). De KEK wordt afgeleid uit het wachtwoord van de gebruiker met Argon2id, eventueel gemengd met een geheim uit een hardwaretoken.

```
wachtwoord ──Argon2id(salt, m, t, p)──┐
                                      ├──HKDF-SHA256──> KEK ──XChaCha20-Poly1305──> ontsluit DEK
token-geheim (optioneel) ─────────────┘                                                  │
                                                                                         v
                                                            SQLCipher: PRAGMA key = "x'<DEK hex>'"
```

De besturingssysteem-sleutelbos wordt **nooit** gebruikt om de datasleutel zelf op te slaan. Hij slaat hooguit een aanvullend geheim op waarmee het openen zonder wachtwoord mogelijk wordt gemaakt (de "snel openen"-optie), en dat is een bewuste keuze van de gebruiker die in het auditspoor landt. Wie die optie niet aanzet, heeft geen enkele afhankelijkheid van de sleutelbos.

Dit onderscheid maakt het hele platformprobleem beheersbaar: **de sleutelbos is een gemak, geen fundering.** Op een systeem zonder sleutelbos verliest de gebruiker een gemak, geen functionaliteit en geen gegevens.

De KDF-parameters worden bij het aanmaken van de kluis gekalibreerd op de machine (doel: ongeveer 1 seconde, met een ondergrens van 256 MiB geheugen, `t=3`, `p=4`) en opgeslagen in de kluisheader, zodat een kluis die op een krachtige machine is gemaakt op een zwakke machine nog te openen is.

### 3.2 De abstractielaag

Eén trait, zeven implementaties, een expliciete beschrijving van wat elke implementatie waarmaakt.

```rust
/// Vertrouwelijkheidsniveau van een backend. Bepaalt wat de gebruiker te zien krijgt.
pub enum Assurance {
    /// Geheim is gebonden aan hardware en verlaat die hardware niet.
    HardwareBound,
    /// Geheim is gebonden aan het gebruikersaccount van het besturingssysteem.
    OsUserBound,
    /// Geheim is beschermd met een wachtwoord dat de gebruiker steeds invoert.
    PassphraseOnly,
}

pub struct BackendCaps {
    pub id: BackendId,
    pub assurance: Assurance,
    pub requires_desktop_session: bool,
    pub supports_user_presence: bool,   // Touch ID, Windows Hello, aanraking van een token
    pub survives_os_reinstall: bool,
}

pub trait SecretStore: Send + Sync {
    fn caps(&self) -> BackendCaps;
    fn probe(&self) -> Result<(), ProbeError>;     // met korte time-out, mag nooit blokkeren
    fn wrap(&self, label: &str, secret: &Zeroizing<[u8; 32]>) -> Result<Wrapped>;
    fn unwrap(&self, label: &str, w: &Wrapped) -> Result<Zeroizing<[u8; 32]>>;
    fn delete(&self, label: &str) -> Result<()>;
}
```

`Wrapped` bevat altijd een backend-aanduiding en een versienummer, zodat een kluis die op een andere machine wordt geopend meteen kan zien dat het opgeslagen geheim daar onbruikbaar is, in plaats van met een cryptische fout te falen.

### 3.3 De backends

| Backend | Platform | Onderliggende API | Rust-kant | Werkt zonder desktopsessie |
|---|---|---|---|---|
| `WindowsDpapi` | Windows | `CryptProtectData` / `CryptUnprotectData`, `CRYPTPROTECT_UI_FORBIDDEN`, met eigen entropie | `windows` crate, `Security::Cryptography` | Ja |
| `WindowsCredMan` | Windows | Credential Manager (`CredWriteW`) | `keyring` crate, feature `windows-native` | Ja |
| `MacKeychain` | macOS | `SecItemAdd` met `kSecAttrAccessibleWhenUnlockedThisDeviceOnly` | `security-framework` crate | Nee |
| `MacSecureEnclave` | macOS | P-256-sleutel met `kSecAttrTokenIDSecureEnclave`, ECIES-omhulling | `security-framework` + `SecKeyCreateEncryptedData` | Nee |
| `SecretService` | Linux | D-Bus `org.freedesktop.secrets` (GNOME Keyring, KWallet 6) | `secret-service` crate op `zbus`, of `keyring` met `sync-secret-service` | Nee |
| `KernelKeyring` | Linux | `keyctl`, sessiesleutelbos — alleen vluchtige cache | `linux-keyutils` crate | Ja, maar niet persistent |
| `PassphraseFile` | Alle | Eigen bestandsformaat, Argon2id + XChaCha20-Poly1305 | `argon2`, `chacha20poly1305`, `zeroize` | Ja |

**Windows.** DPAPI is de eenvoudigste en meest robuuste optie: geen desktopsessie nodig, geen dienst die kan ontbreken, gebonden aan het gebruikersprofiel. De omhulde blob wordt in een eigen bestand naast de database gezet, niet in de Credential Manager, omdat DPAPI geen groottebeperking kent en de blob zo mee kan in een back-up. Extra entropie (`pOptionalEntropy`) wordt afgeleid van de kluis-identifier, zodat een blob van kluis A niet werkt voor kluis B. `CRYPTPROTECT_LOCAL_MACHINE` wordt **niet** gebruikt: dat maakt het geheim leesbaar voor elke lokale beheerder.

**macOS.** De Keychain wordt gebruikt met `kSecAttrAccessibleWhenUnlockedThisDeviceOnly` — geen iCloud-synchronisatie, geen herstel uit een back-up op een ander apparaat. Voor "snel openen" met Touch ID wordt een `SecAccessControl` met `.biometryCurrentSet` gebruikt, zodat het geheim ongeldig wordt zodra de vingerafdrukset verandert.

De Secure Enclave kan geen willekeurige geheimen bewaren; hij kan alleen P-256-sleutels genereren die de chip nooit verlaten. Het patroon is dus: genereer een Secure Enclave-sleutel, versleutel het geheim naar de bijbehorende publieke sleutel met `eciesEncryptionCofactorVariableIVX963SHA256AESGCM`, en laat ontsleuteling alleen toe na gebruikersaanwezigheid. Het gevolg is dat het geheim onherstelbaar verloren gaat bij een wisseling van het apparaat of het logische bord — vandaar dat dit alleen mag voor het aanvullende geheim, nooit voor de datasleutel.

Gebruik van de data protection keychain (`kSecUseDataProtectionKeychain: true`) vereist dat de applicatie is ondertekend en over een `keychain-access-groups`-entitlement beschikt. Dat werkt dus niet in een lokale debug-build, wat betekent dat de bouwstraat deze backend alleen op een ondertekende build kan testen.

**Linux.** Secret Service is de standaard maar is nadrukkelijk niet gegarandeerd aanwezig. GNOME Keyring levert het, KWallet 6 levert het via zijn Secret Service-brug, maar op een minimale desktop, in een SSH-sessie, in een container of op een server is er niets. De sleutelbos kan bovendien aanwezig maar vergrendeld zijn; dan verschijnt er een grafische wachtwoordprompt, wat in een niet-interactieve context vastloopt. Daarom:

```rust
// Detectie, in deze volgorde, elk met een harde time-out van 2 seconden.
fn detect_linux_backend() -> BackendId {
    if env::var_os("DBUS_SESSION_BUS_ADDRESS").is_none() { return BackendId::PassphraseFile; }
    if !dbus_name_has_owner("org.freedesktop.secrets", Duration::from_secs(2)) {
        return BackendId::PassphraseFile;
    }
    match secret_service_collection_is_unlocked() {
        Ok(true)  => BackendId::SecretService,
        Ok(false) => BackendId::SecretServiceLocked, // aanbieden, maar niet automatisch
        Err(_)    => BackendId::PassphraseFile,
    }
}
```

De time-out is essentieel. Een D-Bus-aanroep naar een dienst die niet reageert blokkeert anders het opstarten, en dat is precies wat er op een kale sessie gebeurt.

### 3.4 Terugval zonder sleutelbos

De terugval is geen noodgreep maar het **basisgeval**: `PassphraseFile`, waarbij er simpelweg geen opgeslagen geheim is en de gebruiker bij elke start zijn wachtwoord invoert. Dat is de veiligste variant en de standaardinstelling op alle platformen.

Voor de opstelling zonder desktopomgeving (een gedeelde server, een systeembeheerder die de applicatie in een sessie draait) zijn er drie aanvullende routes:

| Situatie | Route | Toelichting |
|---|---|---|
| Interactieve start op een server | Wachtwoord via `stdin`, niet via een argument | Argumenten zijn zichtbaar in `ps`. De prompt gebruikt `rpassword` en zet echo uit. |
| Start als systemd-dienst | `LoadCredentialEncrypted=` | systemd ontsleutelt met de TPM en levert het geheim via `$CREDENTIALS_DIRECTORY`; het staat nooit als leesbaar bestand op schijf. |
| Automatisering met beheerder erbij | `systemd-ask-password` | Vraagt bij de start om het wachtwoord op elke aangemelde console. |

Binnen één draaiende sessie wordt de ontsleutelde datasleutel op Linux desgewenst in de **kernel-sessiesleutelbos** (`keyctl`, `KEY_SPEC_SESSION_KEYRING`) gehouden in plaats van in gewoon procesgeheugen. Die verdwijnt bij afmelden en komt nooit op schijf.

Wat er nadrukkelijk **niet** gebeurt:

- Geen geheim in een omgevingsvariabele (zichtbaar in `/proc/<pid>/environ`, in crashdumps, in procesboomweergaven).
- Geen geheim in een commandoregelargument.
- Geen "leeg wachtwoord" of vast ingebouwde sleutel als de sleutelbos ontbreekt.
- Geen automatische stap terug naar een zwakkere backend zonder dat de gebruiker dat ziet én bevestigt. Een stille degradatie van `HardwareBound` naar `PassphraseOnly` is een beveiligingsincident, geen gemak.

De keuze van de backend en elke wijziging daarin worden vastgelegd in het auditspoor, met de `Assurance` van vóór en na.

### 3.5 Geheugen en crashdumps

Een sleutel die correct wordt opgeslagen maar in een crashdump belandt, is alsnog gelekt. De maatregelen verschillen per platform:

| Maatregel | Windows | macOS | Linux |
|---|---|---|---|
| Sleutelgeheugen wissen | `zeroize` crate op alle sleuteltypen | idem | idem |
| Uitwisselen naar schijf tegengaan | `VirtualLock` | `mlock` | `mlock`, beperkt door `RLIMIT_MEMLOCK` (standaard 8 MiB, ruim voldoende voor sleutels) |
| Crashdump onderdrukken | Windows Error Reporting uitsluiten voor het proces; `WerRegisterExcludedMemoryBlock` voor de sleutelpagina's | `setrlimit(RLIMIT_CORE, 0)`, entitlement `com.apple.security.get-task-allow` afwezig in de release | `prctl(PR_SET_DUMPABLE, 0)` |
| Debugger-koppeling bemoeilijken | — | Hardened Runtime zonder `get-task-allow` | `prctl(PR_SET_DUMPABLE, 0)` sluit `ptrace` door niet-root |
| Schermafdrukbescherming | `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` | `NSWindow.sharingType = .none` | Niet mogelijk (zie [§11](#11-bekende-beperkingen-per-platform)) |

`mlock` en `PR_SET_DUMPABLE` zijn een verdedigingslaag, geen garantie: zodra het besturingssysteem met gebruikersrechten is gecompromitteerd terwijl de kluis open is, vallen deze buiten het dreigingsmodel.

---

## 4. Hardwaretokens

### 4.1 Waarom niet via de webview

De voor de hand liggende route — `navigator.credentials.create()` in de webview — werkt hier niet, en het is beter dat vooraf te weten dan er halverwege tegenaan te lopen:

- WebAuthn vereist een relying party ID die een geldig domein is. De applicatie draait op `tauri://localhost` respectievelijk `http://tauri.localhost`; er is geen domein en er is bewust geen server.
- WKWebView staat WebAuthn alleen toe met een geconfigureerd Associated Domain, wat een server met een `apple-app-site-association`-bestand vereist. Dat is precies de netwerkafhankelijkheid die het product niet wil.
- WebKitGTK heeft geen WebAuthn-implementatie.

Alle tokencommunicatie gebeurt daarom **native in Rust**, en het token wordt niet als aanmeldmiddel gebruikt maar als **bron van sleutelmateriaal**.

### 4.2 Twee mechanismen, per platform

| | Windows | macOS | Linux |
|---|---|---|---|
| Toegangsroute FIDO2 | `webauthn.dll` (`WebAuthNAuthenticatorGetAssertion`); directe HID-toegang tot FIDO-apparaten is sinds Windows 10 1903 voorbehouden aan verhoogde processen | `libfido2` via IOHIDManager, geen extra rechten voor een niet-gesandboxte, ondertekende app | `libfido2` via `hidraw`, vereist een udev-regel |
| Rust-bibliotheek | `webauthn-authenticator-rs` (Windows-backend) of directe FFI naar `webauthn.dll` | `ctap-hid-fido2` of `libfido2`-binding | `ctap-hid-fido2` of `libfido2`-binding |
| `hmac-secret`-extensie | Beschikbaar via de Windows WebAuthn-API vanaf API-versie 6 (Windows 11 22H2); op Windows 10 in de praktijk niet bruikbaar zonder verhoging | Ja | Ja |
| Platformauthenticator | Windows Hello (TPM) | Touch ID + Secure Enclave | Geen |
| PIV / smartcard | CNG-minidriver, `ncrypt`; ook via PKCS#11 | CryptoTokenKit meldt de PIV-kaart automatisch aan bij de Keychain; ook via PKCS#11 | `pcscd` + `opensc-pkcs11.so` of `ykcs11` |
| PKCS#11 uit Rust | `cryptoki` crate | `cryptoki` crate | `cryptoki` crate |

**De consequentie van de Windows-beperking is groot genoeg om het ontwerp te sturen.** Omdat `hmac-secret` op Windows 10 niet betrouwbaar bereikbaar is voor een niet-verhoogd proces, is er een tweede mechanisme nodig dat op alle drie de platformen identiek werkt.

### 4.3 De gekozen opzet

| Mechanisme | Doel | Platformdekking | Opmerking |
|---|---|---|---|
| **FIDO2 `hmac-secret`** | Tweede factor in de sleutelafleiding | Linux, macOS, Windows 11 22H2+ | Voorkeursroute. Vereist PIN en aanraking. |
| **HMAC-SHA1 uitdaging-antwoord** (YubiKey OTP-applicatie, slot 2) | Zelfde doel, terugvalroute | Alle drie, ook Windows 10 | Bereikbaar via de HID-toetsenbordinterface, geen verhoging nodig. Dezelfde aanpak die KeePassXC gebruikt. |
| **PIV** (PKCS#11) | Ondertekenen van exports en van het auditspoor; optioneel ontsleutelen van de omhulde sleutel | Alle drie | Gebruikt sleutelslot 9c (digitale handtekening) en 9d (sleutelbeheer). |
| **Platformauthenticator** (Windows Hello / Touch ID) | Alleen "snel openen" ontgrendelen | Windows, macOS | Nooit als enige beschermlaag. |

Het tokengeheim gaat via HKDF-SHA256 samen met de wachtwoordafgeleide sleutel de KEK in. Dat betekent: **zonder token én zonder wachtwoord geen toegang, maar het token alleen is ook niet genoeg.**

### 4.4 Wat er nodig is voor een YubiKey

**Alle platformen.**
- YubiKey 5-serie, of Security Key-serie voor uitsluitend FIDO2. Firmware 5.2.3 of nieuwer is aan te bevelen; de uitdaging-antwoordroute werkt ook op de 4-serie.
- Een FIDO2-PIN moet zijn ingesteld voordat `hmac-secret` bruikbaar is.
- Slot 2 moet zijn geprogrammeerd voor HMAC-SHA1 uitdaging-antwoord (via de YubiKey Manager) wanneer de terugvalroute wordt gebruikt.
- De aanraaktimeout van het token is ongeveer 15 tot 30 seconden; de gebruikersinterface toont een expliciete "raak nu je token aan"-toestand met een aftelling en een annuleerknop, want zonder die terugkoppeling lijkt de applicatie te hangen.

**Linux, udev-regels.** Zonder regel heeft een gewone gebruiker geen toegang tot `/dev/hidraw*`. De `.deb` en `.rpm` installeren `/usr/lib/udev/rules.d/70-dpo-fg-tool-tokens.rules`; AppImage-gebruikers krijgen het bestand aangeboden met een instructie.

```
# FIDO2/U2F-apparaten, seat-gebonden toegang via systemd-logind
KERNEL=="hidraw*", SUBSYSTEM=="hidraw", \
  ATTRS{idVendor}=="1050", TAG+="uaccess", GROUP="plugdev", MODE="0660"

# Generiek: elk HID-apparaat met de FIDO-usage page (0xF1D0)
KERNEL=="hidraw*", SUBSYSTEM=="hidraw", \
  ATTRS{report_descriptor}=="*\xf1\xd0*", TAG+="uaccess", GROUP="plugdev", MODE="0660"

# YubiKey PIV via pcscd draait via de CCID-driver; alleen usb-toegang nodig
SUBSYSTEM=="usb", ATTRS{idVendor}=="1050", TAG+="uaccess", GROUP="plugdev", MODE="0660"
```

`TAG+="uaccess"` geeft toegang aan de gebruiker die is aangemeld op de lokale seat. In een SSH-sessie of op een server werkt dat niet — daar is de `GROUP`/`MODE`-combinatie de werkende route, en moet de gebruiker in `plugdev` zitten. Dat verschil moet in de documentatie staan, want het is de meest voorkomende ondersteuningsvraag op Linux.

Voor PIV is daarnaast `pcscd` nodig (`pcscd.socket` ingeschakeld) plus `opensc` of `yubico-piv-tool`.

**macOS.** Buiten de sandbox met Hardened Runtime is er geen aanvullend entitlement nodig voor HID-toegang tot een FIDO-token. Zou de applicatie ooit gesandboxt worden (App Store), dan is `com.apple.security.device.usb` vereist — een reden te meer om niet via de App Store te distribueren.

**Windows.** Geen stuurprogramma nodig; de WebAuthn-API doet het werk en toont de systeemdialoog van Windows. Voor PIV is de ingebouwde minidriver voldoende voor de standaardslots; voor de uitdaging-antwoordroute is geen stuurprogramma nodig.

### 4.5 Herstel

Een token dat kapotgaat, mag geen dossier onbereikbaar maken. Bij het inschakelen van tokenbeveiliging dwingt de applicatie af dat er ten minste één van twee dingen is:

1. Een tweede ingeschreven token (aanbevolen; de klassieke reserve-YubiKey), of
2. Een herstelcode van 128 bit, getoond in blokken van vijf tekens, die de gebruiker uitprint en fysiek bewaart. De code is een alternatieve omhulling van dezelfde datasleutel.

Het aanmaken van de herstelcode wordt in het auditspoor vastgelegd; de code zelf uiteraard niet.

---

## 5. Bestandslocaties

### 5.1 Standaardlocaties

De bundel-identificatie is `nl.dpofgtool.app`. Tauri's padplug-in levert de basispaden; wij leggen de onderverdeling zelf vast zodat die op alle platformen dezelfde structuur heeft.

| Soort | Windows | macOS | Linux |
|---|---|---|---|
| Database (`kluis.db`, `-wal`, `-shm`) | `%LOCALAPPDATA%\nl.dpofgtool.app\data\` | `~/Library/Application Support/nl.dpofgtool.app/data/` | `$XDG_DATA_HOME/nl.dpofgtool.app/data/` (standaard `~/.local/share/…`) |
| Bijlagen (inhoudsgeadresseerd) | `…\data\blobs\` | `…/data/blobs/` | `…/data/blobs/` |
| Instellingen | `%APPDATA%\nl.dpofgtool.app\config\` | `~/Library/Application Support/nl.dpofgtool.app/config/` | `$XDG_CONFIG_HOME/nl.dpofgtool.app/` |
| Logboeken | `%LOCALAPPDATA%\nl.dpofgtool.app\logs\` | `~/Library/Logs/nl.dpofgtool.app/` | `$XDG_STATE_HOME/nl.dpofgtool.app/logs/` |
| Cache (verwijderbaar) | `%LOCALAPPDATA%\nl.dpofgtool.app\cache\` | `~/Library/Caches/nl.dpofgtool.app/` | `$XDG_CACHE_HOME/nl.dpofgtool.app/` |
| Back-ups (standaardvoorstel) | `%USERPROFILE%\Documents\dpo-fg-tool-backups\` | `~/Documents/dpo-fg-tool-backups/` | `~/dpo-fg-tool-backups/` |
| Omhuld sleutelbestand | `…\data\keys\` | `…/data/keys/` | `…/data/keys/` |
| Vergrendelingsbestand | `…\data\kluis.lock` | idem | idem |

Toelichting op de keuzes die afwijken van de standaardreflex:

- **De database staat op Windows in `%LOCALAPPDATA%`, nooit in `%APPDATA%`.** In een Active Directory-omgeving met roaming profiles wordt `%APPDATA%` bij aan- en afmelden gesynchroniseerd met een netwerkshare. Een SQLite-database die op die manier heen en weer wordt gekopieerd raakt vroeg of laat beschadigd. Alleen de instellingen (klein, tekstueel, verlies is onschadelijk) mogen roamen.
- **Op macOS staat niets standaard in `~/Documents`, `~/Desktop` of `~/Downloads`.** Sinds macOS 10.15 vraagt het systeem daar toestemming voor (TCC), en die mappen worden bij veel gebruikers door iCloud Drive gesynchroniseerd — met dezelfde SQLite-schade tot gevolg. De back-upmap ligt daar wél, omdat back-ups een moment-opname zijn en de gebruiker ze moet kunnen vinden; de eerste keer levert dat één toestemmingsdialoog op, die de applicatie vooraf uitlegt.
- **Logboeken staan op Linux onder `XDG_STATE_HOME`**, niet onder `XDG_DATA_HOME`: het zijn geen gegevens die je zou terugzetten uit een back-up.
- De applicatie **weigert** een databaselocatie in een bekende synchronisatiemap (zie [§6.5](#65-netwerkschijven-en-sqlite)).

Bestandsrechten worden expliciet gezet, niet overgelaten aan de umask: `0700` op de mappen en `0600` op database, sleutelbestand en logboeken op macOS en Linux; op Windows een ACL met alleen de eigenaar en `SYSTEM`, met overerving uitgeschakeld.

### 5.2 Draagbare modus

Draagbare modus is expliciet en zichtbaar. Hij wordt geactiveerd door de aanwezigheid van een bestand `dpo-fg-tool.portable` naast het uitvoerbare bestand, of door de vlag `--portable`. Alles staat dan in één map:

```
dpo-fg-tool-draagbaar/
├── dpo-fg-tool(.exe)          # of dpo-fg-tool.AppImage, of dpo-fg-tool.app op macOS
├── dpo-fg-tool.portable       # markeringsbestand, mag leeg zijn
└── gegevens/
    ├── data/   kluis.db, blobs/, keys/
    ├── config/
    ├── logs/
    └── backups/
```

Op macOS is er een verbijzondering: schrijven **in** de `.app`-bundel maakt de codehandtekening ongeldig en zorgt ervoor dat Gatekeeper de applicatie de volgende keer weigert. In draagbare modus staat de map `gegevens/` daarom naast de bundel, in dezelfde bovenliggende map, nooit erin.

**Gedragsverschillen in draagbare modus:**

| Aspect | Normale modus | Draagbare modus |
|---|---|---|
| Sleutelopslag | Sleutelbos van het besturingssysteem toegestaan | Uitsluitend `PassphraseFile`; sleutelbos wordt genegeerd |
| "Snel openen" | Beschikbaar | Uitgeschakeld, niet aan te zetten |
| Hardwaretoken | Optioneel | Optioneel, en nadrukkelijk aanbevolen |
| Automatisch bijwerken | Naar keuze | Altijd uit |
| Interface | Normaal | Permanente markering in de statusbalk: "Draagbare modus" |
| Auditspoor | Normaal | Elke sessie legt vast dat het een draagbare sessie was |

**De risico's, en waarom ze in de interface staan.**

1. **Er is geen accountbinding.** De hele beveiliging rust op het wachtwoord en eventueel het token. Wie de stick heeft, heeft de versleutelde kluis en kan er onbeperkt en offline op proberen te raden. Dit is de reden dat de KDF-parameters in draagbare modus een hogere ondergrens krijgen en dat de applicatie hier een sterker wachtwoord afdwingt.
2. **Het bestandssysteem van de gemiddelde USB-stick is exFAT of FAT32.** Geen rechten, geen journaal, geen `fsync`-garanties zoals SQLite die verwacht. Het risico op beschadiging bij het uittrekken van de stick is reëel. De applicatie detecteert exFAT/FAT32 en toont een waarschuwing; WAL wordt daar niet gebruikt (zie [§6.4](#64-bestandsvergrendeling-bij-een-versleutelde-database)).
3. **Veilig wissen bestaat niet op flashgeheugen.** Door wear leveling blijven oude versies van pagina's fysiek aanwezig. Een verwijderde kluis is niet weg. Het advies in de documentatie is: gebruik hardware-versleuteling van de gehele stick (BitLocker To Go, LUKS, FileVault op een extern volume) *onder* de applicatieversleuteling.
4. **Verlies is stil.** Een verloren stick meldt zichzelf niet. Draagbare modus verplaatst het risico van "iemand breekt in op je werkplek" naar "iemand vindt je stick in de trein". Voor een FG-dossier is dat een afweging die bewust en schriftelijk gemaakt moet worden — de applicatie toont bij het activeren eenmalig een tekst die dit expliciet benoemt en die de gebruiker moet bevestigen.
5. **Praktische obstakels.** Op Linux zijn USB-media vaak met `noexec` gekoppeld, waardoor de AppImage niet start. Op Windows kan het bestandspad van de stick lang zijn en dieper genest dan verwacht. Op macOS blijft het quarantaine-attribuut aan het gekopieerde bestand hangen wanneer het via een browser op de stick is gezet.

De aanbeveling in de handleiding is helder: draagbare modus is bedoeld voor een auditbezoek of een tijdelijke werkplek, niet als permanente opstelling.

---

## 6. Valkuilen van het bestandssysteem

### 6.1 Hoofdlettergevoeligheid en Unicode

| Platform | Standaardgedrag | Uitzonderingen |
|---|---|---|
| Windows | Ongevoelig, behoudt schrijfwijze | Per map instelbaar gevoelig sinds Windows 10 1803; WSL-paden zijn gevoelig |
| macOS | APFS standaard ongevoelig, behoudt schrijfwijze | APFS kan gevoelig geformatteerd worden; oude HFS+-volumes normaliseren naar NFD |
| Linux | Gevoelig | Behalve op gekoppelde NTFS-, exFAT- of SMB-volumes |

De oplossing is om het probleem te vermijden in plaats van te managen: **bestandsnamen op schijf worden nooit afgeleid van gebruikersinvoer.** Elke bijlage wordt inhoudsgeadresseerd opgeslagen:

```
data/blobs/<eerste 2 hex>/<volgende 2 hex>/<volledige BLAKE3-hash>
```

De oorspronkelijke naam, het MIME-type en de datum staan in de database. Dat elimineert in één keer de hoofdlettergevoeligheid, de verboden tekens, de padlengte, de Unicode-normalisatie én de deduplicatie. De naam wordt pas weer een echte bestandsnaam op het moment van exporteren, en dan gaat hij door de sanering van [§6.3](#63-verboden-namen-en-tekens).

Tekst die wél vergeleken moet worden (namen van verwerkingen, leveranciers, zoekopdrachten) wordt bij het opslaan naar **NFC** genormaliseerd met de `unicode-normalization` crate. Zonder die stap wordt een naam die op macOS is ingevoerd niet teruggevonden op Linux.

### 6.2 Padlengte op Windows

`MAX_PATH` is 260 tekens, inclusief de terminator. Dat is nog steeds de standaard tenzij er twee dingen tegelijk waar zijn: de registerwaarde `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem\LongPathsEnabled = 1` én een applicatiemanifest met `<ws2:longPathAware>true</ws2:longPathAware>`.

De aanpak is drieledig:

1. Het manifest van de Windows-build zet `longPathAware` aan. Dat kost niets en helpt op systemen waar de registerwaarde al aanstaat.
2. De Rust-standaardbibliotheek zet absolute paden waar nodig om naar de verbatimvorm (`\\?\C:\…`), maar SQLite en andere C-bibliotheken doen dat niet. Het pad naar de database wordt daarom expliciet kort gehouden en bij het openen gecontroleerd; boven de 200 tekens volgt een waarschuwing, boven de 240 een weigering met de suggestie om de kluis te verplaatsen.
3. Zelfgegenereerde namen (exports, back-ups) worden begrensd op 64 tekens, met een gereserveerde ruimte voor het tijdstempel en de extensie.

Bij het exporteren naar een map die de gebruiker kiest, wordt de volledige lengte vooraf berekend en bij overschrijding een kortere naam voorgesteld in plaats van halverwege te falen.

### 6.3 Verboden namen en tekens

De sanering is één functie, met tests per platform, en wordt op **alle** platformen toegepast — ook op Linux, zodat een export die op Linux is gemaakt op Windows te openen is.

| Categorie | Windows | macOS | Linux |
|---|---|---|---|
| Verboden tekens | `< > : " / \| ? * \` en 0x00–0x1F | `:` (Finder toont het als `/`), 0x00 | `/`, 0x00 |
| Gereserveerde namen | `CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9`, ook met extensie (`CON.txt`) | — | — |
| Afsluitende tekens | Punt en spatie aan het eind worden stil verwijderd door Win32 | — | — |
| Beginnende tekens | — | Punt verbergt het bestand | Punt verbergt het bestand; koppelteken verwart CLI-gereedschap |
| Maximale naamlengte | 255 UTF-16-eenheden | 255 UTF-8-bytes | 255 bytes |

De sanering vervangt verboden tekens door `_`, plakt een underscore achter een gereserveerde naam, knipt afsluitende punten en spaties weg, kapt de naam op bytelengte af (op een tekengrens, niet midden in een meerbyteteken), en garandeert dat het resultaat niet leeg is. Vervolgens wordt op collisie gecontroleerd met een **hoofdletterongevoelige** vergelijking, ook op Linux — anders levert een map die op Linux prima is op Windows een overschrijving op.

### 6.4 Bestandsvergrendeling bij een versleutelde database

De database is SQLCipher 4 via `rusqlite`. De datasleutel wordt als ruwe sleutel doorgegeven (`PRAGMA key = "x'…'"`), zodat de KDF van SQLCipher niet de beveiligingsgrens is — dat is Argon2id in Rust.

```sql
PRAGMA cipher_page_size = 4096;
PRAGMA cipher_memory_security = ON;   -- vraagt om mlock op de paginacache
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;
PRAGMA synchronous = FULL;            -- vol, niet NORMAL: dit is een auditdossier
PRAGMA journal_mode = WAL;            -- alleen op een lokaal, journalend bestandssysteem
```

**Meerdere instanties.** SQLite kan gelijktijdige toegang aan, maar dat is hier ongewenst: twee vensters met hetzelfde geopende dossier leveren een verwarrend auditspoor op. De applicatie gebruikt daarom twee sloten tegelijk:

- `tauri-plugin-single-instance` om een tweede start naar het bestaande venster te leiden. Let op: dat werkt op Windows en Linux betrouwbaar; op macOS regelt het systeem dit al via de bundel-identificatie.
- Een exclusief bestandsslot op `kluis.lock` met de `fd-lock` crate (`LockFileEx` op Windows, `flock` op Unix), met daarin het proces-ID, de hostnaam en het starttijdstip. Bij een botsing wordt getoond *welke* sessie de kluis vasthoudt, niet alleen dát hij bezet is. Een verweesd slot (proces bestaat niet meer) wordt herkend en aangeboden om over te nemen.

**Waar het misgaat per platform:**

| Situatie | Platform | Gevolg | Maatregel |
|---|---|---|---|
| Virusscanner of zoekindexeerder houdt de database vast | Windows | `SQLITE_BUSY`, soms `database is locked` bij het afsluiten | Documenteer uitzonderingen voor Defender en de indexeerder; `busy_timeout`; retry met exponentiële wachttijd |
| Controlled Folder Access blokkeert schrijven | Windows | Schrijffout zonder duidelijke oorzaak | Herkennen aan de foutcode, met een uitleg in plaats van een stacktrace |
| Back-upprogramma kopieert een levend databasebestand | Alle | Beschadigde back-up | Nooit bestandskopieën; back-ups met `VACUUM INTO` |
| WAL-bestand blijft achter na een crash | Alle | Herstel bij de volgende opening | `PRAGMA wal_checkpoint(TRUNCATE)` bij een nette afsluiting |
| Slaapstand met open database op een extern volume | macOS, Windows | Verbroken bestandshandvat | Kluis automatisch vergrendelen bij een slaapmelding |

**Back-ups.** Nooit kopiëren met `fs::copy`. Altijd:

```rust
conn.execute("VACUUM INTO ?1", [backup_path])?;   // consistente, gecomprimeerde, versleutelde kopie
```

`VACUUM INTO` maakt een consistente kopie van een geopende database, behoudt de versleuteling en vereist geen extra vergrendeling. Daarna wordt van het resultaat een SHA-256-som en, indien er een PIV-token is, een handtekening opgeslagen, zodat de integriteit van een back-up achteraf aantoonbaar is.

### 6.5 Netwerkschijven en SQLite

SQLite's vergrendeling leunt op POSIX-advieslocks respectievelijk `LockFileEx`. Over NFS, SMB en vrijwel elke FUSE-koppeling zijn die niet betrouwbaar. WAL-modus werkt er sowieso niet: die gebruikt gedeeld geheugen via een `-shm`-bestand, wat een netwerkbestandssysteem niet levert. Het resultaat is geen nette foutmelding maar stille beschadiging, soms pas weken later zichtbaar.

Daarom: **de applicatie weigert standaard een database op een niet-lokaal bestandssysteem te openen.**

| Platform | Detectie |
|---|---|
| Windows | `GetDriveTypeW` levert `DRIVE_REMOTE`; daarnaast paddetectie op UNC (`\\server\share`) en op een toegewezen letter die naar UNC verwijst (`WNetGetConnection`) |
| macOS | `statfs`, controle op `MNT_LOCAL` in `f_flags`; daarnaast `f_fstypename` tegen `smbfs`, `nfs`, `afpfs`, `webdav` |
| Linux | `statfs`, `f_type` vergeleken met `NFS_SUPER_MAGIC` (0x6969), `SMB_SUPER_MAGIC`/`CIFS` (0xFF534D42), `FUSE_SUPER_MAGIC` (0x65735546); aanvullend `/proc/self/mountinfo` uitlezen |

Synchronisatiemappen krijgen dezelfde behandeling, want de fout is identiek van aard: OneDrive, Dropbox, Google Drive, Nextcloud en iCloud Drive worden herkend aan hun bekende paden en markeringsbestanden, plus op Windows aan het `FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS`-attribuut van bestanden op aanvraag.

Wie het toch wil, kan het forceren met `--allow-remote-storage`. Dan schakelt de applicatie over op `journal_mode=TRUNCATE`, `locking_mode=EXCLUSIVE` en enkelvoudige toegang, toont een permanente waarschuwing en legt de keuze vast in het auditspoor. Voor een gedeelde opstelling is dit expliciet níet de aanbevolen route; daarvoor is er de serveropstelling met één proces dat de database bezit.

### 6.6 Regeleindes en tekensets in exports

| Formaat | Regeleinde | Codering | Bijzonderheden |
|---|---|---|---|
| CSV, profiel "Excel (NL)" | CRLF | UTF-8 met BOM | Puntkomma als scheidingsteken, komma als decimaalteken, `sep=;` als eerste regel |
| CSV, profiel "standaard" | CRLF (RFC 4180) | UTF-8 zonder BOM | Komma als scheidingsteken, punt als decimaalteken |
| JSON, JSON Lines | LF | UTF-8 zonder BOM | Altijd, op elk platform |
| Markdown, tekst | LF, met CRLF als optie | UTF-8 zonder BOM | |
| PDF, DOCX | n.v.t. | | In Rust gegenereerd, byte-voor-byte gelijk op alle platformen |
| Auditspoor-export | LF | UTF-8 zonder BOM | Regeleindes vaststaand, want het bestand wordt ondertekend |

Twee punten die vaak misgaan en die hier expliciet zijn geregeld:

**De BOM.** Excel op Windows opent een UTF-8-CSV zonder BOM in de systeemcodepagina, waardoor elke `é` en `ë` verminkt raakt — en een verwerkingsregister staat vol Nederlandse namen. Met BOM gaat het goed in Excel, maar struikelen sommige Unix-gereedschappen erover. Vandaar twee profielen, met de Excel-variant als standaard voor CSV omdat de doelgroep het bestand doorgaans in Excel opent.

**Formule-injectie.** Een veld dat begint met `=`, `+`, `-`, `@`, een tab of een carriage return wordt door Excel en LibreOffice als formule uitgevoerd. In een datalekregister waarin externe partijen tekst hebben aangeleverd, is dat een reëel aanvalspad. Elke celwaarde die met een van die tekens begint, wordt voorafgegaan door een enkel aanhalingsteken. Dit geldt op alle platformen, want het bestand reist mee.

Het auditspoor wordt met vaste LF-regeleindes en in een canonieke vorm geëxporteerd, omdat de handtekening anders per platform verschilt. In de repository staat dit al vast via `.gitattributes` (`* text=auto eol=lf`); in de exportcode staat het onafhankelijk daarvan opnieuw vast, want de gebruiker heeft geen Git.

### 6.7 Tijdzones en zomertijd bij termijnbewaking

Termijnbewaking is een kernfunctie: 72 uur voor een datalekmelding aan de Autoriteit Persoonsgegevens (artikel 33 AVG), 24 uur voor de vroegtijdige waarschuwing en 72 uur voor de incidentmelding onder NIS2, één maand voor een verzoek van een betrokkene (artikel 12 lid 3 AVG). Een fout van een uur in oktober is hier geen schoonheidsfoutje.

**Opslag.** Alle tijdstippen worden opgeslagen als UTC in ISO 8601 met expliciete `Z`, plus een apart veld met de IANA-zone waarin de gebeurtenis is vastgelegd (standaard `Europe/Amsterdam`). Zonder dat tweede veld is een civiele termijn achteraf niet reproduceerbaar.

**Tijdzonedatabase.** Windows heeft geen IANA-tijdzonedatabase; het levert namen als `W. Europe Standard Time`. Vertalen via de CLDR-mapping is een bron van fouten en van afhankelijkheid van OS-updates. Daarom wordt de tijdzonedatabase **meegeleverd in de binary** via de `chrono-tz` crate, op alle drie de platformen identiek. Bijwerken gebeurt bij een release, niet door het besturingssysteem. De gebruikte tzdb-versie staat in het systeemrapport en in de softwarestuklijst.

**Twee soorten termijn, twee soorten rekenwerk.** Dit is het punt waar de meeste implementaties de fout in gaan:

| Termijn | Aard | Rekenwijze | Zomertijdgevolg |
|---|---|---|---|
| 72 uur (artikel 33 AVG), 24 en 72 uur (NIS2) | Vaste duur | Exact 72 × 3600 seconden opgeteld bij het UTC-tijdstip | Geen. De klokverzetting verandert de wandkloktijd van de deadline, maar niet de deadline zelf. |
| Eén maand (artikel 12 lid 3 AVG), verlenging met twee maanden | Kalendertermijn | Civiele rekenkunde in `Europe/Amsterdam` conform Verordening (EEG, Euratom) nr. 1182/71 | Wel. De deadline is een kalenderdatum op hetzelfde uur; die kan in een verzette uur vallen. |
| Bewaartermijnen, evaluatiedata | Kalendertermijn | Idem | Idem |

Voor kalendertermijnen gelden twee regels die getest moeten worden:

1. **Maandeindeklem.** 31 januari plus één maand is 28 februari (of 29 in een schrikkeljaar), niet 3 maart. `chrono`'s `checked_add_months` doet dit correct; zelf tellen met dagen doet het niet.
2. **Verzette uren.** Op de laatste zondag van maart bestaat de wandkloktijd 02:30 niet; op de laatste zondag van oktober bestaat 02:30 twee keer. De regel in dit product: bij een niet-bestaande tijd wordt vooruit geschoven naar het eerste bestaande moment; bij een dubbele tijd wordt het **eerste** voorkomen genomen. Dat is telkens de voor de betrokkene gunstigste uitleg, en het is als zodanig vastgelegd.

```rust
// Voorbeeldtests die in de bouwstraat op alle drie de platformen draaien
assert_eq!(deadline_uur(utc("2026-03-28T23:30:00Z"), Duration::hours(72)),
           utc("2026-03-31T23:30:00Z"));                    // duur, niet civiel
assert_eq!(deadline_maand(local("2026-01-31T09:00:00", AMS), 1),
           local("2026-02-28T09:00:00", AMS));               // maandeindeklem
assert_eq!(deadline_maand(local("2026-02-28T02:30:00", AMS), 1),
           local("2026-03-28T02:30:00", AMS));               // bestaat nog
// 29-03-2026 is de zomertijdovergang; 02:30 bestaat daar niet. Het anker moet
// dus twee maanden terug, want februari 2026 kent geen 29e.
assert_eq!(deadline_maand(local("2026-01-29T02:30:00", AMS), 2),
           local("2026-03-29T03:00:00", AMS));               // 02:30 bestaat niet, vooruit
```

**De systeemklok is niet te vertrouwen.** Een gebruiker kan de klok verzetten, en op een virtuele machine kan hij na een hervatting verspringen. Bij elke opslag wordt naast de wandklok een monotone teller (`std::time::Instant` sinds de start, plus de starttijd) vastgelegd. Een verschil van meer dan vijf minuten tussen wat de wandklok zegt en wat de monotone teller verwacht, levert een vermelding in het auditspoor op en een waarschuwing in de interface. Dat is geen beveiliging tegen een kwaadwillende beheerder, maar het maakt een klokverzetting achteraf zichtbaar — en dat is precies wat de verantwoordingsplicht vraagt.

De weergave van een termijn toont altijd zowel de resterende tijd als het absolute moment in de lokale zone mét zone-aanduiding: *"nog 14 uur — uiterlijk woensdag 28 oktober 2026 om 02:00 (CET)"*.

---

## 7. Distributie en ondertekening

### 7.1 Windows

**Pakketformaten.** De Tauri-bundelaar levert MSI (via WiX 3) en NSIS. Beide worden gebouwd, met een verschillende doelgroep:

| Formaat | Installatiebereik | Doelgroep | Rechten |
|---|---|---|---|
| NSIS `.exe` | `currentUser` | De individuele FG op een beheerde werkplek | Geen beheerdersrechten nodig |
| MSI | `perMachine` | Uitrol door de IT-afdeling via Intune, SCCM of GPO | Beheerdersrechten |

De NSIS-installatie voor de huidige gebruiker is de standaarddownload. Veel FG's werken op een werkplek waar zij geen beheerder zijn, en een product dat daar niet te installeren is, wordt niet gebruikt.

**WebView2.** De standaardinstelling van Tauri is `downloadBootstrapper`, die tijdens de installatie de runtime van Microsoft ophaalt. Dat is voor dit product ongewenst: het vereist internet tijdens de installatie, precies wat we niet willen aannemen. Er worden daarom twee varianten gepubliceerd:

| Variant | `webviewInstallMode` | Grootte | Wanneer |
|---|---|---|---|
| Standaard | `embedBootstrapper` | ongeveer +2 MB | Machine met internet tijdens de installatie |
| Offline | `offlineInstaller` | ongeveer +130 MB | Machine zonder internet, of een uitrol in een afgeschermd netwerk |

Een `fixedRuntime`-variant (een meegeleverde, vastgezette WebView2-versie van ongeveer 180 MB) blijft achter de hand voor omgevingen die willen voorkomen dat een Edge-update de applicatie beïnvloedt. Nadeel: wij worden dan verantwoordelijk voor de beveiligingsupdates van die runtime, en dat is een verplichting die je niet lichtvaardig aangaat.

**Authenticode.** Alles wordt ondertekend: het hoofd-uitvoerbare bestand, elke meegeleverde DLL, de NSIS-installer en de MSI. Ondertekenen gebeurt met een tijdstempel, zodat de handtekening geldig blijft nadat het certificaat is verlopen:

```
signtool sign /fd SHA256 /td SHA256 /tr http://timestamp.digicert.com \
              /a /d "dpo-fg-tool" dpo-fg-tool.exe
```

Sinds juni 2023 eisen de CA/Browser Forum-regels dat de private sleutel van een codeondertekeningscertificaat in gecertificeerde hardware zit (FIPS 140-2 niveau 2 of gelijkwaardig). Een `.pfx`-bestand in een geheimenkluis van de bouwstraat kan dus niet meer. In de praktijk blijven er drie routes over: een fysieke token met een zelfgehoste runner, een cloudondertekeningsdienst (Azure Trusted Signing, DigiCert KeyLocker, SSL.com eSigner), of ondertekenen op een aparte, afgeschermde machine.

**SmartScreen.** Dit is de grootste hobbel voor de gebruikerservaring bij de eerste start, en het is eerlijker om dat te benoemen dan het te verdoezelen. Een nieuw ondertekend programma zonder reputatie levert het scherm *"Windows heeft uw pc beschermd"* op, waarbij de gebruiker op "Meer informatie" moet klikken om te kunnen installeren. Reputatie bouwt zich op per uitgever en per certificaat, op basis van het aantal installaties. De vroegere aanname dat een EV-certificaat onmiddellijk reputatie geeft, gaat niet meer op.

Wat wel helpt:
- Van certificaat niet wisselen. Elke wissel begint de reputatieopbouw opnieuw.
- Elke release bij Microsoft indienen via *Submit a file for analysis* in het Microsoft Security Intelligence-portaal.
- Op de downloadpagina een schermafbeelding van het SmartScreen-venster tonen met de uitleg wat de gebruiker moet doen, plus de SHA-256-som en de te verwachten uitgeversnaam in het handtekeningvenster. Een gebruiker die vooraf weet wat hij gaat zien, hoeft niet te twijfelen.

### 7.2 macOS

**Pakketformaat.** Een `.app`-bundel in een `.dmg`, gebouwd als Universal 2 zodat één download op Intel en Apple Silicon werkt:

```
cargo tauri build --target universal-apple-darwin
```

**Ondertekening.** Een *Developer ID Application*-certificaat uit het Apple Developer Program, met Hardened Runtime ingeschakeld. Volgorde is belangrijk: eerst de meegeleverde binaries en frameworks, dan de bundel zelf. `codesign --deep` is verouderd en dekt niet alles.

```
codesign --force --options runtime --timestamp \
         --entitlements entitlements.plist \
         --sign "Developer ID Application: <organisatie> (<team-id>)" \
         dpo-fg-tool.app
codesign --verify --deep --strict --verbose=4 dpo-fg-tool.app
```

Het entitlements-bestand blijft bewust minimaal. Geen `com.apple.security.cs.allow-unsigned-executable-memory`, geen `allow-dyld-environment-variables`, en `com.apple.security.get-task-allow` staat op `false` in de release-build (zie [§3.5](#35-geheugen-en-crashdumps)). JIT is niet nodig in het hoofdproces: WKWebView voert JavaScript uit in een apart WebContent-proces met zijn eigen rechten.

**Notarisatie.** Verplicht sinds macOS 10.15. Zonder notarisatie weigert Gatekeeper de applicatie botweg, zonder bruikbare omweg voor een normale gebruiker.

```
xcrun notarytool submit dpo-fg-tool.dmg --key AuthKey.p8 \
      --key-id <KEY_ID> --issuer <ISSUER_ID> --wait
xcrun stapler staple dpo-fg-tool.dmg
xcrun stapler validate dpo-fg-tool.dmg
spctl -a -vvv -t install dpo-fg-tool.app
```

Notarisatie duurt doorgaans enkele minuten, incidenteel enkele uren. De bouwstraat moet daar met een ruime time-out rekening mee houden. Het `stapler`-commando hecht het notarisatiebewijs aan het bestand; dat is essentieel, want anders moet de machine van de gebruiker bij de eerste start Apple's server kunnen bereiken — en juist onze gebruikers zitten regelmatig op een afgeschermd netwerk.

**Quarantaine en de eerste start.** Een `.dmg` die via een browser is gedownload, krijgt het uitgebreide attribuut `com.apple.quarantine`. Bij de eerste start toont macOS: *"macOS heeft gecontroleerd op schadelijke software en niets gevonden"* — dat is de gewenste uitkomst en die verschijnt alleen bij een correct genotariseerde en gestapelde bundel. Slepen naar `/Applications` verwijdert het attribuut niet, maar dat is ook niet nodig.

Nadrukkelijk **niet** in de documentatie zetten: de instructie `xattr -d com.apple.quarantine`. Wie gebruikers aanleert om quarantaine weg te halen, leert ze een gewoonte aan die hen bij het volgende programma de das omdoet. Als de instructie nodig is, is er iets mis met de ondertekening en moet dat opgelost worden.

### 7.3 Linux

Er is geen door het besturingssysteem afgedwongen vertrouwensketen voor losse downloads. Ondertekening is hier dus een zaak van gepubliceerde sleutels en van gebruikers die controleren.

| Formaat | Doelgroep | Ondertekening |
|---|---|---|
| `.deb` | Debian 12, Ubuntu 22.04 en 24.04 | Losstaande GPG-handtekening; bij een eigen apt-repo een ondertekend `InRelease`-bestand |
| `.rpm` | Fedora, RHEL 9, Rocky, Alma | `rpm --addsign` met de projectsleutel; publieke sleutel te importeren met `rpm --import` |
| AppImage | Distributie-onafhankelijk | Losstaande `.asc` naast het bestand |
| `.tar.gz` | Handmatige installatie, servers | Losstaande `.asc` |
| Flatpak | Desktopgebruikers via Flathub | Ondertekening en distributie via de Flathub-infrastructuur (OSTree, GPG) |

Bij elke release wordt gepubliceerd:

```
SHA256SUMS          # alle artefacten van alle drie de platformen in één bestand
SHA256SUMS.asc      # losstaande handtekening met de releasesleutel
```

De vingerafdruk van de releasesleutel staat in `README.md`, in `SECURITY.md`, in de applicatie zelf onder **Help → Over** en op de projectpagina. Een gebruiker die de vingerafdruk op vier plaatsen ziet overeenkomen, heeft een redelijke zekerheid.

Aanvullend wordt elke release met `cosign sign-blob` ondertekend met een OIDC-identiteit uit de bouwstraat, waarbij de handtekening in de openbare Rekor-transparantielog terechtkomt. Dat levert een onafhankelijk, publiek controleerbaar spoor op dat aantoont wanneer welk artefact door welke bouwstraat is gemaakt — waardevol voor een organisatie die de herkomst van haar compliance-gereedschap moet kunnen onderbouwen.

**Flatpak** verdient een aparte opmerking. De sandbox is beveiligingswinst, maar hij verandert het gedrag: bestandstoegang loopt via XDG Desktop Portals, de sleutelbos via de portal `org.freedesktop.secrets`, en de USB-toegang tot een YubiKey vereist expliciete rechten (`--device=all` is te grof; `--device=hidraw` waar beschikbaar). Draagbare modus is in een Flatpak zinloos. De Flatpak wordt daarom gebouwd met een aangepast standaardgedrag en met een duidelijke vermelding van wat er anders is.

### 7.4 Kosten en doorlooptijden

Indicatief, peildatum medio 2026. Prijzen van certificaatuitgevers wisselen sterk en het loont om te vergelijken.

| Post | Kosten | Doorlooptijd eerste keer | Opmerking |
|---|---|---|---|
| Apple Developer Program (organisatie) | ongeveer € 99 per jaar | 2 dagen tot 3 weken | Vertraging zit vrijwel altijd in de D-U-N-S-verificatie van de rechtspersoon |
| Apple Developer Program (individu) | ongeveer € 99 per jaar | 1 tot 3 werkdagen | Geen D-U-N-S nodig, maar wel je eigen naam in het handtekeningvenster |
| Notarisatie | inbegrepen | minuten per build | Geen aparte kosten |
| Authenticode OV via Certum (open source, op smartcard) | € 100 tot € 180 eerste jaar, inclusief lezer | 1 tot 3 weken | Goedkoopste route; identificatie via notaris of videosessie |
| Authenticode OV via Sectigo, SSL.com of een reseller | € 250 tot € 450 per jaar | 1 tot 5 werkdagen na een compleet dossier | Vereist KvK-inschrijving en een controleerbaar telefoonnummer |
| Authenticode EV | € 350 tot € 700 per jaar | 3 tot 10 werkdagen, plus 1 tot 2 weken koeriersdienst voor de token | Geen automatische SmartScreen-voorsprong meer |
| Azure Trusted Signing | ongeveer $ 10 per maand | 1 tot 7 werkdagen | Vereist dat de organisatie aantoonbaar 3 jaar bestaat; individuele variant beschikbaar |
| Cloud-HSM (DigiCert KeyLocker, SSL.com eSigner) | € 200 tot € 600 per jaar bovenop het certificaat | direct na uitgifte | Alternatief voor een fysieke token in de bouwstraat |
| GPG-releasesleutel | € 0 | uren | Bij voorkeur op een hardwaretoken |
| Sigstore / cosign | € 0 | uren | Publieke transparantielog |
| Flathub-publicatie | € 0 | 1 tot 6 weken review | Eerste inzending duurt het langst |
| Eigen apt/rpm-repository | € 0 tot € 100 per jaar hosting | 1 tot 2 dagen inrichten | Alleen zinvol bij regelmatige releases |

**Volgorde van aanschaf.** Begin met de Apple Developer Program-inschrijving en de GPG-sleutel: die zijn goedkoop, hebben de langste doorlooptijd voor de rechtspersoonverificatie, en zonder ondertekening is macOS onbruikbaar. Windows kan tijdelijk zonder certificaat worden uitgeleverd — met een duidelijke waarschuwing en gepubliceerde hashes — terwijl de aanvraag loopt. Linux heeft vanaf dag één geen kosten.

### 7.5 De eerste start, per platform

| | Wat de gebruiker ziet | Wat wij doen om het te verzachten |
|---|---|---|
| Windows | SmartScreen-waarschuwing tot er reputatie is; daarna alleen de UAC-prompt bij de MSI | Vooraf uitleggen met schermafbeelding, hash publiceren, uitgeversnaam noemen |
| macOS | Sleep-naar-Programma's, daarna eenmalig "macOS heeft gecontroleerd op schadelijke software" | Correct notariseren en stapelen; geen `xattr`-instructies |
| Linux, `.deb`/`.rpm` | Normale pakketinstallatie; de udev-regels worden meegeïnstalleerd | Token vereist mogelijk opnieuw aansluiten of aanmelden — dat staat in de installatiemelding |
| Linux, AppImage | Uitvoerbaar maken (`chmod +x`), dan starten | Udev-regels apart aangeboden met een kopieerbaar commando |
| Linux, Flatpak | Normale installatie uit Flathub | Afwijkend gedrag benoemd in de beschrijving en bij de eerste start |

Bij elke eerste start toont de applicatie het systeemrapport uit [§1.6](#16-controle-bij-het-opstarten) met een groen, oranje of rood oordeel per onderdeel, vóórdat er een kluis wordt aangemaakt. Wie hier ziet dat er geen sleutelbos beschikbaar is en dat draagbare modus actief is, weet waar hij aan begint.

---

## 8. Bijwerken

### 8.1 Het uitgangspunt

Een automatische updater is per definitie een mechanisme dat code van buiten haalt en met de rechten van de gebruiker uitvoert. Voor een applicatie die het complete kwetsbaarhedenoverzicht van een organisatie beheert, is dat de aantrekkelijkste aanvalsroute die er is. Tegelijk is niet bijwerken ook een risico.

De oplossing is niet om te kiezen, maar om de gebruiker of de organisatie te laten kiezen — en om de gevaarlijke variant niet de standaard te maken.

### 8.2 Drie modi

| Modus | Standaard voor | Netwerkverkeer | Beschrijving |
|---|---|---|---|
| **Handmatig** | Alle installaties | Geen | De applicatie doet niets. De gebruiker downloadt zelf, controleert zelf, installeert zelf. |
| **Alleen melden** | Optioneel, na expliciet aanzetten | Eén HTTPS-verzoek, alleen op verzoek van de gebruiker | Een knop "controleer op updates" haalt een ondertekend manifest op en toont het resultaat. Er wordt niets gedownload en niets geïnstalleerd. |
| **Automatisch** | Nooit standaard | Periodiek | `tauri-plugin-updater` met handtekeningcontrole. Alleen aan te zetten door een gebruiker die daarvoor bewust kiest. |

Voor beheerde omgevingen bestaat een vierde variant: een **bouwvlag**. De enterprise-build wordt gecompileerd zonder de updater-feature, zodat de code fysiek niet in het binaire bestand zit. Dat is sterker dan een instelling, en het is precies wat een IT-afdeling wil kunnen aantonen die de applicatie via Intune of een eigen repository uitrolt.

```toml
[features]
default = []                     # geen updater in de standaardbuild
updater = ["tauri-plugin-updater"]
```

### 8.3 Handtekeningcontrole

Waar de updater wél wordt gebruikt, geldt:

- Het manifest én het artefact worden allebei ondertekend met een minisign-sleutelpaar (het formaat dat `tauri-plugin-updater` gebruikt). De **publieke sleutel is in de binary gecompileerd**, niet uit een bestand of van het netwerk geladen.
- De private sleutel staat niet in de gewone bouwstraat. Ondertekenen gebeurt in een aparte taak die alleen op een tag draait, in een beschermde omgeving, met handmatige goedkeuring.
- De handtekening wordt gecontroleerd **voordat** er iets naar schijf wordt geschreven dat als code kan worden uitgevoerd. Een gedownload bestand krijgt eerst een tijdelijke naam zonder uitvoerbare rechten.
- **Terugvalbescherming.** Een update naar een gelijk of lager versienummer wordt geweigerd, ook met een geldige handtekening. Anders kan een aanvaller die het transport beheerst een oude versie met een bekende kwetsbaarheid opdringen. Naast het semantische versienummer wordt een monotoon oplopend bouwnummer gecontroleerd.
- De TLS-verbinding gebruikt `rustls` met een vastgezette verzameling wortelcertificaten en een vastgelegde SPKI-hash van de distributiehost. Geen systeemcertificaatopslag, geen proxyvertrouwen.

### 8.4 Terugval bij mislukking

Het echte risico bij bijwerken is niet het binaire bestand maar de **databasemigratie**. Een halverwege afgebroken migratie op een auditdossier is een ernstig incident.

De procedure, in deze volgorde:

1. **Voor de migratie** wordt met `VACUUM INTO` een volledige kopie weggeschreven naar `data/backups/pre-migratie-<oude versie>-<tijdstempel>.db`, met de SHA-256-som ernaast. Zonder een geslaagde kopie start de migratie niet.
2. **Migraties zijn voorwaarts, genummerd en transactioneel.** Elke migratie draait in één `BEGIN IMMEDIATE … COMMIT` en werkt `PRAGMA user_version` in dezelfde transactie bij. Bij een fout is de database ongewijzigd.
3. **Een nieuwere binary weigert een oudere database niet te openen, maar een oudere binary weigert een nieuwere database wél** — met een leesbare melding en de exacte versie die nodig is. De schemaversie staat naast `user_version` ook in een tabel met de minimaal vereiste applicatieversie.
4. **De vorige versie blijft beschikbaar.** Op Windows blijft het vorige installatieprogramma staan in `%LOCALAPPDATA%\nl.dpofgtool.app\updates\vorige\`; op macOS blijft de vorige `.app` bewaard; op Linux blijft het vorige pakket staan. Terugrollen is: vorige versie installeren, back-up van stap 1 terugzetten.
5. **Elke stap landt in het auditspoor**, inclusief de hashes van de back-up voor en na. Voor de verantwoordingsplicht moet aantoonbaar zijn dat er tijdens een migratie niets aan de inhoud is veranderd.

Als de applicatie na een update niet start, is er een `--safe-mode` die de webview niet laadt, alleen het systeemrapport, de logboeken en het herstelmenu toont. Op alle drie de platformen aan te roepen vanaf de opdrachtregel, met de exacte aanroep beschreven in de foutmelding die bij een mislukte start wordt getoond.

### 8.5 Volledig handmatig bijwerken

De ondersteunde route zonder enig netwerkverkeer vanuit de applicatie:

1. De gebruiker haalt het nieuwe artefact en `SHA256SUMS` plus `SHA256SUMS.asc` op via een browser of via de IT-afdeling.
2. Controle van de handtekening met `gpg --verify SHA256SUMS.asc SHA256SUMS` en van de som met `sha256sum -c`, `shasum -a 256 -c` of `Get-FileHash`. De exacte commando's per platform staan in de release-aantekeningen, kopieerbaar.
3. Installeren over de bestaande versie heen. De gegevens blijven staan; ze zitten niet in de programmamap (behalve in draagbare modus, waar de gebruiker de map `gegevens/` zelf meeneemt).
4. Bij de eerste start voert de applicatie de migratie uit volgens [§8.4](#84-terugval-bij-mislukking).

Onder **Help → Over** staan de huidige versie, het bouwnummer, de commit-hash, de vingerafdruk van de gebruikte handtekening en de SHA-256-som van het draaiende binaire bestand. Daarmee kan een gebruiker zonder netwerkverbinding controleren dat wat er draait, is wat er hoort te draaien.

---

## 9. Aantoonbaar geen uitgaand netwerkverkeer

"Wij versturen geen gegevens" is een bewering. De vraag is hoe je hem controleerbaar maakt — voor jezelf, en voor een klant die dit in een leveranciersbeoordeling moet kunnen onderbouwen.

### 9.1 Voorkomen bij de bron

| Maatregel | Uitwerking |
|---|---|
| Geen netwerkbibliotheek in de afhankelijkheidsboom van de kern | Nagerekend op 21-08-2026: `dpofg-cli`, `dpofg-crypto`, `dpofg-store`, `dpofg-audit`, `dpofg-domain`, `dpofg-rules`, `dpofg-report` en `dpofg-verify` bevatten er geen. `dpofg-schil` wél: Tauri brengt `reqwest` en `hyper` mee, en die zijn er niet uit te halen zonder Tauri te verlaten. De `[bans]`-lijst in `deny.toml` die dit zou afdwingen bestaat nog niet — daar staat alleen `openssl`, dat toch al niet in de graaf zit. Zolang die lijst er niet is, rust deze maatregel op de netwerkstiltetest en niet op de afhankelijkhedencontrole |
| Geen Tauri-plug-ins met netwerkfuncties | De `http`-plug-in wordt niet meegecompileerd; de updater alleen achter een feature-vlag |
| Strikte capabilities | Tauri v2-capability-bestanden geven alleen de commando's vrij die daadwerkelijk gebruikt worden; geen jokertekens |
| CSP met `connect-src 'none'` | Blokkeert `fetch`, `XMLHttpRequest`, WebSocket en `EventSource` binnen de webview |
| Geen externe bronnen in de frontend | Geen CDN, geen weblettertypen, geen bronkaarten die naar buiten verwijzen |
| Geen telemetrie, geen crashrapportage naar buiten | Crashrapporten worden lokaal weggeschreven en door de gebruiker desgewenst handmatig meegestuurd |

### 9.2 Statische controle in de bouwstraat

```bash
# 1. Geen verboden afhankelijkheden, ook niet transitief
cargo deny check bans

# 2. Toon expliciet wie een netwerkbibliotheek binnenhaalt (moet leeg zijn)
cargo tree --workspace --edges normal --invert reqwest hyper ureq curl 2>/dev/null

# 3. Vergelijk de afhankelijkheidsboom met de vastgelegde momentopname
cargo tree --workspace --edges normal > tree.txt
diff -u audit/tree-baseline.txt tree.txt

# 4. Netwerksymbolen in het eigen binaire bestand
#    Linux/macOS: connect, getaddrinfo, SSL_connect
nm -D target/release/dpo-fg-tool | grep -E ' U (connect|getaddrinfo|SSL_connect)$'
#    Windows: geïmporteerde netwerk-DLL's
dumpbin /imports dpo-fg-tool.exe | findstr /I "ws2_32 winhttp wininet"
```

**Een eerlijke kanttekening.** De symbolencontrole is een signaal, geen bewijs. De webview zelf — WebView2, WKWebView, WebKitGTK — is een volwaardige browsermotor die netwerkcode bevat en die wij niet uit het proces kunnen verwijderen. WebView2 doet bovendien uit zichzelf verzoeken naar Microsoft-eindpunten voor onderdelen als SmartScreen en componentupdates, tenzij dat wordt uitgezet. Daarom is de statische controle nooit het enige bewijs, en daarom is de CSP met `connect-src 'none'` geen luxe maar de eigenlijke maatregel voor de webview-laag. In `tauri.conf.json` worden de WebView2-argumenten aangevuld met `--disable-features=msSmartScreenProtection,msEdgeComponentUpdater` en wordt de "browser signin" uitgezet.

### 9.3 Dynamische controle per platform

| Platform | Techniek | Uitvoering |
|---|---|---|
| Linux | Netwerknaamruimte zonder route naar buiten | `unshare --user --map-root-user --net -- ./dpo-fg-tool` — het proces heeft alleen `lo`. Wat werkt, werkt zonder netwerk. |
| Linux | Systeemaanroepen aftappen | `strace -f -e trace=connect,sendto,sendmsg,socket -o trace.log ./dpo-fg-tool`; de bouwstraat controleert dat er geen `connect()` naar een niet-`AF_UNIX`-adres in staat |
| Linux | Pakketopname | `ip netns` met een gespiegelde interface, `tcpdump -i any -w capture.pcap not host 127.0.0.1` |
| macOS | Pakketfilter in een VM | `pf`-regel `block drop out log all` op de niet-loopback-interface, logboek via `pflog0` met `tcpdump -n -e -ttt -i pflog0` |
| macOS | Verbindingen per proces | `lsof -nP -i -a -p <pid>` en `nettop -P -p <pid> -L 1` tijdens een scriptmatige sessie |
| Windows | Firewallregel plus auditlogboek | `New-NetFirewallRule -Direction Outbound -Program … -Action Block` en `auditpol /set /subcategory:"Filtering Platform Connection" /failure:enable`; gebeurtenis-ID 5157 in het beveiligingslogboek toont elke geblokkeerde poging |
| Windows | Pakketopname per proces | `pktmon start --capture --comp nics -p <pid>` en analyse van het resulterende ETL-bestand |
| Windows | Procesniveau | Sysmon met gebeurtenis-ID 3 (netwerkverbinding) gefilterd op het procespad |

### 9.4 De test in de bouwstraat

Bij elke release draait op elk platform een **netwerkstiltetest**:

1. Start het gebouwde artefact in een schone virtuele machine of container, met een pakketopname actief op alle interfaces.
2. Doorloop een scriptmatige sessie: kluis aanmaken, wachtwoord instellen, verwerking toevoegen, DPIA aanmaken, datalek registreren met termijnbewaking, exporteren naar CSV en PDF, back-up maken, afsluiten.
3. Stop de opname. De test slaagt wanneer er nul pakketten zijn naar een ander adres dan de loopback, en nul DNS-verzoeken. Aan-uit van de netwerkkaart en NTP van de gastheer worden buiten de meting gehouden en dat wordt in het rapport genoemd.
4. De opname (`.pcap` of `.etl`), het `strace`-logboek en het testrapport worden als bouwartefact bewaard en meegepubliceerd bij de release.

Dat laatste is het punt. Een klant die vraagt of de applicatie gegevens verstuurt, krijgt geen belofte maar een pakketopname bij de release die hij zelf kan openen. Dat is het verschil tussen "wij doen dat niet" en "hier is het bewijs".

Aanvullend draait de test **een tweede keer mét een netwerkverbinding** en met een onderscheppende proxy. Ook dan moeten er nul verbindingen zijn. De eerste variant toont aan dat de applicatie zonder netwerk werkt; de tweede dat zij het netwerk ook niet gebruikt als het er wél is.

---

## 10. Bouwstraat

### 10.1 Runners en artefacten

| Doel | Runner | Container | Artefacten |
|---|---|---|---|
| Linux x86_64 | `ubuntu-24.04` | Ubuntu 22.04 (glibc-ondergrens) | `.deb`, AppImage, `.tar.gz` |
| Linux, RPM-familie | `ubuntu-24.04` | Rocky Linux 9 | `.rpm` |
| Windows x64 | `windows-2022` | — | NSIS `.exe` (per gebruiker), MSI (per machine), offline-variant |
| macOS universal | `macos-14` (arm64) | — | `.dmg` met Universal 2-bundel |

De macOS-bouw draait op een arm64-runner met beide doelen (`x86_64-apple-darwin` en `aarch64-apple-darwin`) geïnstalleerd; `cargo tauri build --target universal-apple-darwin` voegt ze samen met `lipo`.

### 10.2 Testmatrix

| Test | Linux | macOS | Windows | Toelichting |
|---|---|---|---|---|
| `cargo test` (unit) | ja | ja | ja | Verplicht groen op alle drie |
| `cargo clippy -- -D warnings` | ja | ja | ja | |
| `cargo fmt --check` | ja | nee | nee | Platformonafhankelijk, één keer volstaat |
| Integratietests SQLCipher, migraties, `VACUUM INTO` | ja | ja | ja | Inclusief migraties over meerdere versies heen |
| Padsanering en bestandsnamen | ja | ja | ja | Per platform eigen assertions (gereserveerde namen, padlengte, hoofdletters) |
| Tijdzone- en termijntests | ja | ja | ja | Met meegeleverde tzdb; de OS-tijdzone wordt in de test verzet om te bewijzen dat dat niets uitmaakt |
| Sleutelbos-backendtests | ja, onder `dbus-run-session` met een ontgrendelde `gnome-keyring-daemon` | beperkt, vereist een ondertekende build en `security unlock-keychain` | ja, DPAPI en Credential Manager | Op macOS alleen in de release-tak |
| Terugvaltest zonder sleutelbos | ja, `DBUS_SESSION_BUS_ADDRESS` leeggemaakt | ja | ja, DPAPI uitgeschakeld gesimuleerd | Moet altijd op `PassphraseFile` uitkomen |
| Hardwaretokentests | nee | nee | nee | Vereist fysieke hardware; nachtelijk op een zelfgehoste runner met aangesloten YubiKey |
| Frontend unit (`vitest`) | ja | nee | nee | Motoronafhankelijk |
| Playwright, engine `chromium` | ja | nee | ja | Benadert WebView2 |
| Playwright, engine `webkit` | ja | ja | nee | Benadert WKWebView en WebKitGTK |
| Visuele regressie | ja | ja | ja | Referentiebeelden per motor, niet per platform |
| `tauri-driver` e2e | ja, `WebKitWebDriver` onder `xvfb` | **niet ondersteund** | ja, `msedgedriver` met exact overeenkomende WebView2-versie | Zie [§11](#11-bekende-beperkingen-per-platform) |
| Netwerkstiltetest | ja | ja, in een VM | ja | Zie [§9.4](#94-de-test-in-de-bouwstraat) |
| Installatie- en eerste-starttest | ja, `.deb`/`.rpm`/AppImage in een schone container | ja, in een VM met quarantaine-attribuut gezet | ja, in een schone VM inclusief SmartScreen-observatie | Alleen op een tag |
| Handtekeningverificatie van het eigen artefact | `gpg --verify`, `rpm --checksig` | `codesign --verify --deep --strict`, `spctl -a -t install`, `stapler validate` | `signtool verify /pa /v` | Verplichte poort vóór publicatie |
| Reproduceerbaarheidscontrole | ja | ja | ja | Twee keer bouwen, hashes van de ongetekende payload vergelijken |
| `cargo audit`, `cargo deny check` | ja | nee | nee | Platformonafhankelijk |
| SBOM-generatie | ja | ja | ja | Per platform, want de systeemafhankelijkheden verschillen |

Wat waar draait, volgt één regel: **platformonafhankelijke controles draaien één keer, op Linux.** Alles wat het besturingssysteem echt raakt — paden, sloten, sleutelopslag, ondertekening, webview — draait overal.

### 10.3 Reproduceerbaar bouwen

Doel: twee bouwen van dezelfde commit, op verschillende momenten, leveren identieke binaire bestanden. Dat is de basis onder de bewering dat het gepubliceerde artefact overeenkomt met de gepubliceerde broncode.

| Maatregel | Uitwerking |
|---|---|
| Vastgezette toolchain | `rust-toolchain.toml` met een exacte versie, niet `stable` |
| Vastgezette afhankelijkheden | `Cargo.lock` en `pnpm-lock.yaml` in de repository, gebouwd met `--locked` respectievelijk `--frozen-lockfile` |
| Vastgezette bouwomgeving | Container-images bij digest, niet bij tag |
| Tijdstempels | `SOURCE_DATE_EPOCH` gezet op de committijd; bundelaars die dat negeren worden nabewerkt |
| Padnamen | `RUSTFLAGS="--remap-path-prefix=$PWD=/build --remap-path-prefix=$HOME/.cargo=/cargo"` |
| Determinisme | `CARGO_INCREMENTAL=0`, één codegen-eenheid voor de release, vaste `-C debuginfo` |
| Frontend | Vaste bestandsnamen zonder inhouds-hash waar mogelijk, vaste minifier-versie, `sourcemap: false` |
| Verificatie | De bouwstraat bouwt elke release twee keer in aparte taken en vergelijkt de SHA-256-sommen |

**De handtekening breekt de reproduceerbaarheid, en dat is geen fout.** Een Authenticode-handtekening en een macOS-codehandtekening bevatten een tijdstempel; twee ondertekende bestanden zijn dus nooit bit-identiek. Daarom wordt de hash van de **ongetekende** payload gepubliceerd naast de hash van het uitgeleverde bestand, met de instructie hoe je de handtekening eraf haalt (`osslsigncode remove-signature` op Windows, `codesign --remove-signature` op een kopie op macOS) om de reproduceerbaarheid zelf na te rekenen.

### 10.4 Softwarestuklijst per platform

Een SBOM is per platform verschillend, want de afhankelijkheden verschillen: WebKitGTK op Linux, WebView2 op Windows, WebKit uit het besturingssysteem op macOS.

```bash
# Rust-onderdeel
cargo cyclonedx --format json --all --output-prefix sbom-rust

# Frontend-onderdeel
pnpm dlx @cyclonedx/cyclonedx-npm --omit dev --output-file sbom-frontend.json

# Samenvoegen tot één stuklijst per platform
cyclonedx-cli merge --input-files sbom-rust.json sbom-frontend.json sbom-system-<platform>.json \
                    --output-file sbom-<platform>-<versie>.cdx.json
```

`sbom-system-<platform>.json` wordt per platform samengesteld en bevat de onderdelen die het besturingssysteem levert maar die wel tot het draaiende geheel behoren:

| Platform | Systeemonderdelen in de stuklijst |
|---|---|
| Linux | `libwebkit2gtk-4.1`, `libgtk-3`, `glibc`, `libsecret-1`, `libfido2`, `pcsc-lite` — met de minimaal vereiste versie, gemarkeerd als geleverd door het besturingssysteem |
| Windows | Microsoft Edge WebView2 Runtime (met de versie waartegen is gebouwd en de minimale versie), `webauthn.dll`, `ws2_32.dll` |
| macOS | WebKit uit het besturingssysteem (minimale macOS-versie), Security.framework, LocalAuthentication.framework |

Aanvullend:

- `cargo auditable build` sluit de afhankelijkheidslijst in het binaire bestand zelf in, zodat `cargo audit bin dpo-fg-tool` op een willekeurige machine werkt — ook op een machine waar de broncode niet staat.
- De SBOM's worden ondertekend met dezelfde releasesleutel en meegepubliceerd, samen met een `cargo about`-licentieoverzicht.
- De versie van de meegeleverde tijdzonedatabase staat als apart onderdeel in de stuklijst, want die heeft een eigen updateritme en is voor de termijnbewaking van belang.

### 10.5 Omgang met ondertekeningsgeheimen

Ondertekenen gebeurt in een aparte taak, met een aparte omgeving die handmatige goedkeuring vereist, en alleen op een tag.

| Geheim | Opslag | Bereikbaar voor |
|---|---|---|
| Apple API-sleutel (`.p8`), team-ID, issuer-ID | Omgevingsgeheimen van de bouwstraat | Alleen de macOS-ondertekeningstaak |
| Apple Developer ID-certificaat | Als `.p12` in een geheim, geïmporteerd in een tijdelijke sleutelbos die na afloop wordt verwijderd | Idem |
| Windows-ondertekening | Cloud-HSM of een zelfgehoste runner met de fysieke token; nooit een `.pfx` in een geheim | Alleen de Windows-ondertekeningstaak |
| GPG-releasesleutel | Hardwaretoken op een zelfgehoste runner | Alleen de publicatietaak |
| Minisign-updatersleutel | Apart, offline, alleen ingezet bij een release met updater | Alleen de releasetaak |

Bouwen vanuit een pull request krijgt nooit toegang tot een geheim. Een PR-bouw levert ongetekende artefacten met een duidelijke aanduiding in de bestandsnaam.

---

## 11. Bekende beperkingen per platform

### 11.1 Het overzicht

| # | Platform | Beperking | Gevolg voor de gebruiker | Omgang |
|---|---|---|---|---|
| 1 | Linux | Geen WebAuthn in de webview | Geen zichtbaar gevolg; tokens werken native | Geen |
| 2 | Linux | Secret Service kan ontbreken of vergrendeld zijn | Geen "snel openen"; wachtwoord bij elke start | Gemeld in het systeemrapport, met uitleg hoe je GNOME Keyring of KWallet installeert |
| 3 | Linux | Schermafdrukbescherming onmogelijk (X11 noch Wayland biedt een equivalent van `WDA_EXCLUDEFROMCAPTURE`) | Schermdelen toont het dossier | Expliciet in het systeemrapport en in de handleiding; automatische vergrendeling bij inactiviteit als compensatie |
| 4 | Linux | Grote spreiding in WebKitGTK-versies | Kleine opmaakverschillen op oudere distributies | Ondergrens afgedwongen; onder de grens start de applicatie niet |
| 5 | Linux | Leeg venster door de DMA-BUF-renderer op sommige drivers | Applicatie lijkt niet te starten | Automatische detectie en uitwijk; in het logboek vermeld |
| 6 | Linux | Fractionele schaling onder Wayland geeft wazige tekst in WebKitGTK | Cosmetisch | Bekend, met `GDK_SCALE`-advies in de handleiding |
| 7 | Linux | Geen door het OS afgedwongen handtekeningcontrole voor losse downloads | De gebruiker moet zelf verifiëren | Kopieerbare verificatiecommando's bij elke release |
| 8 | Linux, Flatpak | Sandbox verandert bestandstoegang en tokentoegang | Afwijkend gedrag ten opzichte van de andere pakketten | Apart benoemd bij de eerste start in een Flatpak |
| 9 | macOS | `tauri-driver` ondersteunt macOS niet | Geen geautomatiseerde end-to-end-tests | Playwright met de `webkit`-engine plus een handmatig rooktestdraaiboek per release |
| 10 | macOS | Secure Enclave-geheimen overleven een apparaatwissel niet | Bij vervanging van de Mac moet de kluis met het wachtwoord worden geopend | Bij het inschakelen expliciet gemeld; herstelcode verplicht |
| 11 | macOS | Notarisatie vereist internet bij het bouwen | Geen gevolg voor de gebruiker | Alleen relevant voor de bouwstraat |
| 12 | macOS | Toestemmingsdialoog bij de eerste back-up naar `~/Documents` | Eén extra klik | Vooraf uitgelegd in de interface |
| 13 | Windows | WebView2 werkt zelfstandig bij; een Edge-update kan de interface beïnvloeden | Zeldzaam maar mogelijk | Nachtelijke tests tegen Edge Beta en Dev; `fixedRuntime`-variant beschikbaar |
| 14 | Windows | SmartScreen-waarschuwing tot er reputatie is | Extra klik bij de eerste installatie | Vooraf uitgelegd met schermafbeelding en hash |
| 15 | Windows 10 | `hmac-secret` niet betrouwbaar bereikbaar zonder verhoging | FIDO2-sleutelafleiding niet beschikbaar | Automatische uitwijk naar HMAC-SHA1 uitdaging-antwoord; het systeemrapport vermeldt welke route actief is |
| 16 | Windows | Roaming profiles en mapomleiding kunnen de gegevensmap naar het netwerk verplaatsen | Risico op databasebeschadiging | Detectie en weigering; alleen te forceren met een expliciete vlag |
| 17 | Windows | Virusscanners melden NSIS-installers soms onterecht | Installatie geblokkeerd | Hashes gepubliceerd; procedure om een melding als onterecht in te dienen |
| 18 | Alle | Draagbare modus levert geen accountbinding | Zwakkere bescherming bij verlies van de gegevensdrager | Permanente markering in de interface, sterkere KDF-parameters, eenmalige bevestiging |
| 19 | Alle | Geen bescherming tegen een gecompromitteerd besturingssysteem terwijl de kluis open is | Buiten het dreigingsmodel | Vastgelegd in `SECURITY.md` en in het dreigingsmodel |

### 11.2 Hoe dit de gebruiker bereikt

Een beperking die alleen in de projectdocumentatie staat, bestaat voor de gebruiker niet. Er zijn vier kanalen, en ze dragen alle vier dezelfde tekst uit één bron.

| Kanaal | Moment | Inhoud |
|---|---|---|
| **Systeemcontrole bij de eerste start** | Vóór het aanmaken van een kluis | Het volledige `PlatformReport` met per onderdeel een oordeel: goed, aandachtspunt, of blokkerend. Blokkerende punten kunnen niet worden overgeslagen. |
| **Help → Systeemcontrole** | Altijd bereikbaar | Hetzelfde rapport, plus de actieve sleutelbos-backend, de webview-versie, het bestandssysteem van de gegevensmap, de tzdb-versie en of draagbare modus actief is. Exporteerbaar als tekstbestand voor een ondersteuningsvraag. |
| **Contextuele melding** | Op het moment dat het ertoe doet | Bij het inschakelen van "snel openen" op een systeem zonder sleutelbos, bij het kiezen van een netwerkpad, bij het activeren van draagbare modus. Kort, ter plekke, en met de reden erbij. |
| **`BEKENDE-BEPERKINGEN.md`** | Meegeleverd in elk pakket en op de downloadpagina | Bovenstaande tabel, aangevuld per release. Genummerd, zodat een ondersteuningsvraag naar een nummer kan verwijzen. |

Twee redactionele regels voor deze teksten:

1. **Noem de reden, niet alleen de beperking.** "Geen sleutelbos gevonden" helpt niemand. "Er is op dit systeem geen sleutelbos beschikbaar (GNOME Keyring of KWallet ontbreekt). U voert daarom bij elke start uw wachtwoord in. Dat is veilig; het is alleen minder gemakkelijk." helpt wel.
2. **Zeg wat de gebruiker kan doen, of zeg expliciet dat er niets te doen is.** Een beperking waar niets aan te doen valt — punt 3 in de tabel, schermafdrukbescherming op Linux — moet dat ook zeggen, met de compenserende maatregel erbij.

Elk oordeel uit het systeemrapport landt bij het openen van de kluis in het auditspoor. Zo is achteraf te reconstrueren onder welke omstandigheden aan een dossier is gewerkt — en dat is niet alleen diagnostiek, maar onderdeel van de verantwoording die een FG moet kunnen afleggen.
