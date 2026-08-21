# Stand van de bouw

Bijgewerkt op 21 augustus 2026.

Dit document zegt wat er **werkt**, wat er **niet werkt**, en waar de grenzen liggen. Het is bewust nuchter: een overzicht dat meer suggereert dan er staat, kost bij de eerste inspectie meer dan het oplevert.

De cijfers erin zijn gemeten, niet geschat, en de opdrachten zijn nagelopen door ze te draaien. Waar een getal snel veroudert, staat de opdracht die het actuele antwoord geeft.

---

## Inhoud

- [Waar de draad ligt](#waar-de-draad-ligt) — wat als eerste aan de beurt is
- [Wat er staat](#wat-er-staat)
- [Wat er niet staat](#wat-er-niet-staat)
- [Grenzen die benoemd horen te worden](#grenzen-die-benoemd-horen-te-worden)
- [Cijfers](#cijfers)
- [Bevindingen die het nalopen heeft opgeleverd](#bevindingen-die-het-nalopen-heeft-opgeleverd)

---

## Waar de draad ligt

Wat als eerste aan de beurt is, en wat er bekend is over de plaats waar het is blijven liggen. Deze sectie staat vooraan omdat dit is wat je bij het oppakken binnen een minuut wilt weten, in plaats van na een uur.

### De bouwstraat wordt geweigerd, hij is niet stuk

GitHub weigert de banen te starten met de melding dat een betaling is mislukt of de bestedingslimiet omhoog moet. De baan faalt binnen enkele seconden zonder ook maar één stap te draaien; in `gh run list` ziet dat eruit als een gewone `failure`, en `gh run view --log-failed` geeft "log not found" omdat er geen log is. Haal in dat geval eerst de annotatie op voordat er iets aan de code verandert:

```sh
gh api repos/WimLee115/dpo-fg-tool/check-runs/<baan-id>/annotations
```

Dit is alleen op te lossen in **Billing & plans**. Wat er intussen wél is gedaan: de platformmatrix draait niet meer bij elke duw. macOS telt tien minuten per minuut en Windows twee, en samen aten die twee banen meer dan de helft van het maandbudget. Linux draait nu bij elke duw, macOS en Windows wekelijks en op afroep. Dat is een concessie — een fout die alleen op Windows optreedt, komt tot een week later aan het licht — en daarom hoort de matrix vóór een uitgave met de hand te worden gestart.

Zolang de bouwstraat stilstaat, is de enige weg de banen lokaal draaien: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features --exclude dpofg-schil -- -D warnings`, `cargo test --workspace` en `cargo deny check`.

### De motortests op WebKit draaien niet op deze machine

Playwright mist er systeembibliotheken voor en wil ze met beheerdersrechten installeren:

```sh
sudo pnpm exec playwright install-deps
```

WebKit is de motor die er het meest toe doet — hij benadert zowel WKWebView op macOS als WebKitGTK op Linux — dus zolang dat niet is gebeurd, dekt alleen de bouwstraat die kant af. De schil draait wél op de echte WebKitGTK zodra `dpofg-schil` start; dat is een andere toets, maar geen niets.

### Waar de functionaliteit ophoudt

De controleregels zijn het duidelijkst af te bakenen stuk werk dat overblijft. Vraag de stand op met:

```sh
dpofg controle --dekking
```

Die opdracht noemt precies welke regels nog geen evaluatiefunctie hebben en wat zij zouden controleren. Van de regels die nog niet draaien zijn er zeven uit de groep **organisatie**; die wachten alle zeven op hetzelfde ontbrekende rollen- en aanstellingsregister. Dat is daarmee het grootste samenhangende blok, en tegelijk het blok dat het meest oplevert: één nieuw subdomein ontsluit zeven regels.

Let op de volgorde die het project zichzelf oplegt: een code komt pas in de dekkingslijst wanneer de gegevens waarop hij oordeelt via de opdrachtregel ook in te vullen zijn. Eerst het register, dan de opdrachten, dan pas de regel. Andersom levert het het patroon van LEK-15 op — een regel die aanslaat op iets waar de gebruiker niets aan kan doen.

---

## Wat er staat

### Fase 0 — Kluiskern

| Onderdeel | Stand | Toelichting |
|---|---|---|
| Sleutelafleiding | werkt | Argon2id, parameters in de kluis zodat verzwaren later mogelijk blijft |
| Envelopversleuteling | werkt | XChaCha20-Poly1305, elke envelop gebonden aan veld, record en compartiment |
| Sleutelhiërarchie | werkt | Drie lagen; wachtwoord wijzigen herversleutelt niets |
| Compartimenten | **deels** | Cryptografisch gescheiden, met een eigen sleutel per compartiment. Rotatie bestaat als cryptografische functie, maar heeft geen route door de opslaglaag en geen opdracht die haar aanroept |
| Blinde index | werkt | Zoeken in versleutelde velden; weigert velden met lage variatie |
| Ketenlogboek | werkt | Hashketen, append-only tot in de database via triggers |
| Ankers | werkt | Ed25519; het enige middel tegen afkappen aan het einde |
| Termijnenmotor | werkt | Getypeerde termijnen, maandeindeklem, opschorting in kalenderdagen |
| Versleutelde opslag | **deels** | Bijlagen inhoudsgeadresseerd en ontdubbeld. Elke wijziging bewaart de vorige versie in de kluis, maar geen opdracht en geen scherm kan die teruglezen of terugzetten |

### Fase 1 — Register, incident en aantoonbaarheid

#### Registers en dossiers

| Onderdeel | Stand | Toelichting |
|---|---|---|
| Verwerkingsregister | werkt | Artikel 30 AVG, beide schema's, met afgeleide verplichtingen |
| Effectbeoordeling | werkt | Voortoets, de vier onderdelen van art. 35 lid 7, restrisico met weging, raadplegingsklok |
| Incidentdossier | **deels** | Drie AVG-klokken op eigen ankers, en een meldbesluit met drie lagen. Het model kent daarnaast vier zorgplichtklokken, maar elke aanroeper zet de zorgplichtcontext op 'niet van toepassing', zodat die vier nergens in het product verschijnen |
| Betrokkenenverzoeken | werkt | Maandtermijn met beide lezingen, vindplaatsen uit het register, art. 19-kennisgeving, art. 12 lid 4-bericht |
| Woo-spoor | werkt | Eigen beslistermijn van vier weken, weigeringsgronden gescheiden in absoluut en relatief, zienswijze van derden |
| Wpg-spoor | werkt | Toepasselijkheid met motivering, jaarlijkse controle en vierjaarlijkse audit, verbeterplan met eigenaar per maatregel |
| Leveranciersregister | werkt | De acht onderdelen van art. 28 lid 3 met vindplaats per onderdeel, contractuele meldtermijn, subverwerkerslijst met controledatum |
| Doorgiften buiten de EER | werkt | Instrument, doorgiftebeoordeling, artikel 49 met telling, controle tegen het kennispakket |
| Belangenafweging | werkt | Vier onderdelen vóór de uitkomst; waarborgen kantelen de uitslag maar vervangen de afweging niet |
| Risicobeoordeling | werkt | Eigen dossier met methode, reikwijdte, bronnen en onderkende risico's; het restrisico daalt alleen met een maatregel erbij, en een hoog restrisico aanvaardt het bestuur |
| Zorgplichtcontrolset | werkt | De tien onderdelen van art. 21 lid 3 Cbw, afgeleid uit het kennispakket; bewijs met een geldigheidsvenster maakt een maatregel aantoonbaar, of niet, en is in te trekken zonder te verdwijnen |
| Veldmapping | werkt | Eén profiel per bronsysteem, verschilrapport in twee richtingen, genegeerde velden met reden |
| Redactieregie | werkt | Profiel, uitlevering aan extern hulpmiddel, terugleescontrole; verstrekking geblokkeerd tot die slaagt |
| Persoonlijk dossier van de functionaris | werkt | Een tweede kluisbestand met een eigen wachtwoordzin; adviezen met comply-or-explain, onafhankelijkheidsincidenten, en een hash in de kluis van de organisatie waarmee het bestaan is aan te tonen zonder de inhoud |

#### Termijnen en verplichtingen

| Onderdeel | Stand | Toelichting |
|---|---|---|
| Klokkenmotor | werkt | Leidt verplichtingen af; ankers vallen niet samen |
| Volledigheidsmechanisme | werkt | Teller met grondslag; blokkerend onderscheiden van signalerend |
| Werkbak | werkt | Eén lijst over alle regimes heen, met de grondslag en het anker per regel; een verplichting verdwijnt doordat het dossier verandert, nooit door een vinkje. Ook als JSON |
| Correctieplicht | werkt | Per bevinding een besluit met eigenaar en einddatum: herstel of gemotiveerde afwijking; alleen een lopende afwijking onderdrukt de bevinding |

#### Bewaking en vooruitzicht

| Onderdeel | Stand | Toelichting |
|---|---|---|
| Controleregels | **deels** | Niet elke regel in de catalogus heeft een evaluatiefunctie; `dpofg controle --dekking` noemt welke wel en welke niet |
| De drie factoren van aantoonbaarheid | **deels** | Vastgesteld, uitgevoerd en actueel als drie tellingen; geen gewogen score, want het plan geeft daarvoor geen schaal |
| Vervalprognose | werkt | Welke eisen op 30, 90 en 365 dagen niet meer aantoonbaar zijn, met oorzaak, eigenaar en datum; over vijf dossiersoorten heen |

#### Aantoonbaarheid naar buiten

| Onderdeel | Stand | Toelichting |
|---|---|---|
| Kennispakketten | werkt | Ondertekend, met terugrolbescherming en consolidatiedatum |
| Dossiers | werkt | Ondertekend manifest, weglatingen zichtbaar, voorbehoud meegetekend |
| Bestuursstuk | werkt | De prognose als ondertekende bundel met leesbaar stuk, machineleesbare kant en logboek; te controleren met dezelfde losse verificatiebinary |
| Verificatiebinary | werkt | Leest uitsluitend, geen wachtwoord, geen kluis nodig |
| Installatiesleutel | werkt | Eén vaste ondertekenidentiteit per kluis; overleeft een wachtwoordwissel |

#### Bediening

| Onderdeel | Stand | Toelichting |
|---|---|---|
| Bedieningsschil | werkt | Volledig werkproces via de opdrachtregel |
| Grafische schil | werkt | Svelte 5 en Vite in de webview van Tauri v2; de controleronde draait er dezelfde veertien beoordelingen als op de opdrachtregel, uit dezelfde functie. Twee vensters met elk een eigen wachtwoordzin en een eigen brug: het slot, de werkbak, het dossiervenster, de controleronde en de prognose voor de organisatie, en het persoonlijke dossier met de spiegel apart |
| Installatie | werkt | Eén beheerscript per platform dat bouwt, plaatst, bijwerkt, de stand toont en verwijdert; het laat de gegevens bij het verwijderen staan en wist ze alleen wanneer u dat apart bevestigt door het woord WISSEN over te typen, en het weigert een schil te plaatsen die als ontwikkelbouw is uitgevallen |

---

## Wat er niet staat

| Onderdeel | Uit fase | Waarom het er nog niet is |
|---|---|---|
| Toestemming als eigen record | 1 | De verwijzing bestaat in het model; het record nog niet |
| Ketenregister voorbij de eerste schil | 3 | De verwerker en zijn subverwerkers staan er; de laag daaronder nog niet |
| Toezichtdossier en bestuursrechtelijk spoor | 4 | |
| Raamwerkvariant B en C | 3 | Het mechanisme staat er; de kaders zelf ontbreken in het kennispakket |
| Driefactorscore als één gewogen getal | 3 | De drie factoren worden geteld; een gewogen score vergt een schaal en een weging die het plan niet geeft |
| Volwassenheidsniveau | 3 | Komt in het plan één keer voor als veldnaam, zonder definitie |
| Crosswalk naar informatiebeveiligingsnormen | −1 / 1 | Vereist een mappinggraaf met reviewhoudbaarheid per rand |
| Certificaten en mandaten in de prognose | 3 | Die records bestaan nog niet. Het mappingprofiel bestaat wél (`dpofg mapping`), maar de geldigheid van een mappingreview wordt nog niet in de prognose meegenomen; de prognose meldt uitdrukkelijk wat zij niet overziet |
| Ketenbewijs tussen organisaties | 4 | |
| Overdrachtsdossier bij een wisseling van functionaris | 5 | De scheiding tussen het persoonlijke dossier en het adviesregister van de organisatie is er; de overdracht zelf nog niet |
| Multi-entiteit | 5 | |

---

## Grenzen die benoemd horen te worden

### Sleutels, cryptografie en formaat

**De installatiesleutel is er; rotatie en intrekking niet.** Ankers en dossiermanifesten dragen één vaste sleutel per kluisbestand, te tonen met `dpofg kluis sleutel` en te controleren met `dpofg-verify --sleutel`. Wat daar niet in zit:

* **Geen rotatie en geen intrekking.** Raakt de sleutel weg, dan is de enige weg een nieuwe publiceren en de oude ernaast blijven vermelden. Het schema draagt al een generatiekolom en de verificatiebinary aanvaardt al meerdere sleutels, dus rotatie is later toe te voegen zonder formaatwijziging.
* **"Installatie" betekent dit kluisbestand.** Twee kluizen op één machine krijgen twee identiteiten. Een kopie van een kluis draagt dezelfde privésleutel en is er cryptografisch niet van te onderscheiden. Wat er wél bij komt: twee kopieën die doortekenen leveren twee ankers op met hetzelfde volgnummer, een verschillende hash en dezelfde ondertekenaar — die vork is aan de installatie toe te schrijven.
* **De sleutel is zo sterk als kluisbestand plus wachtwoordzin.** Wie beide heeft, kan ondertekenen namens de organisatie. Vóór deze uitgave viel er niets te stelen; nu wel.
* **De bewaarplaats van een anker valt buiten de handtekening** en is dus na ondertekening aan te passen. Dat wacht op een formaatversie 2, samen met een installatie-identificatie in de ondertekende bytes; twee formaatbumps waar één volstaat, is de duurste manier om een gepubliceerde specificatie te onderhouden.

**Compartimenten hangen aan één wachtwoord.** De scheiding is cryptografisch echt — elk compartiment heeft een eigen sleutel — maar alle sleutels hangen aan dezelfde kluissleutel. Het persoonlijke dossier van de functionaris, dat de organisatie níet moet kunnen openen, heeft daarom een tweede kluisbestand met een eigen wachtwoordzin.

**Schemaversie 2 is eenrichtingsverkeer.** Een kluis die één keer met deze uitgave is geopend, staat op schemaversie 2 en wordt geweigerd door elke binary van vóór die migratie — met de juiste melding, want het bestandsmerk blijft gelijk. Let op dat het versienummer daarbij geen houvast geeft: het staat sinds het begin op 0.1.0, dus twee binaries met hetzelfde nummer kunnen een verschillende schemaversie aan. Terug kan alleen met een reservekopie. Een logboek van een kluis die uit schemaversie 1 is gemigreerd, draagt bovendien de handelingsnaam `installatiesleutel_aangemaakt`; een oudere `dpofg-verify` loopt daarop stuk met afsluitcode 1 — luidruchtig, en dus acceptabel. Een kluis die met deze uitgave is aangemaakt, kent die regel niet: daar staat de sleutel in de omschrijving van `kluis_aangemaakt`, en dat leest een oudere binary gewoon.

**Geen hardwaretoken.** Het platformhoofdstuk beschrijft hoe FIDO2 en PIV per besturingssysteem werken; de implementatie ontbreekt.

### De juridische inhoud is niet vastgesteld

**Het kennispakket is een vertrekpunt, geen bron van recht.** De termijnen, feestdagen en grondslagen zijn niet door een jurist gecontroleerd tegen de geconsolideerde wettekst. `dpofg pakket voorbehoud` toont wat er te verifiëren valt. Dit geldt in het bijzonder voor:

* **De effectbeoordeling.** De negen criteria, de drempel van twee en de zesendertig maanden voor herbeoordeling staan in het pakket en zijn daar als te verifiëren gemarkeerd. De vier inhoudseisen van artikel 35 lid 7 staan er wél in maar dragen dat voorbehoud niet, terwijl zij dezelfde controle verdienen. Het lidnummer waarop de raadplegingstermijn berust is gecorrigeerd van lid 3 naar lid 2; die tekst reist mee naar elk dossier en hoort dus tegen de bron te worden gecontroleerd.
* **De Woo-inhoud.** De beslistermijn van vier weken, de verdaging van twee weken en de opsomming van weigeringsgronden staan in het pakket. Of de opsomming van artikel 5.1 volledig is en of elke grond bij het juiste lid staat, hoort tegen de geconsolideerde wettekst te worden gecontroleerd.
* **De drempel voor structureel gebruik van artikel 49.** De verordening noemt geen getal bij het woord "incidenteel". Het pakket hanteert twee toepassingen per jaar als werkbare grens en zegt daar met zoveel woorden bij dat dit getal niet uit de wet komt. Wie een andere grens verdedigbaar acht, stelt hem daar bij zonder de programmacode te raken.

**De lezing van de verzoektermijn is een keuze, geen antwoord.** Of de maand loopt vanaf ontvangst van het verzoek of pas vanaf de vaststelling van de identiteit, is omstreden. De tool toont beide lezingen met hun bron en legt de gekozen lezing met motivering vast; zij kiest niet. Wie de ruimste lezing neemt, kan nooit te laat zijn omdat hij van een later moment uitging.

### Wat de bewaking niet ziet

**De regelcatalogus is groter dan wat er draait.** Niet elke regel in de catalogus heeft een evaluatiefunctie. `dpofg controle --dekking` toont welke wél en welke niet; het aantal regels zegt niets over wat er wordt bewaakt, die opdracht wel.

De meeste regels die nog niet draaien wachten op een recordsoort die er niet is, en niet op programmeerwerk: er wachten regels op een systeemregister, een schoningsopdracht, een rollen- en aanstellingsregister, een rapportagespoor en een toezichtdossier. Niet allemaal: REG-07 oordeelt over de doelomschrijving van een registerregel, en dat veld bestaat al — die regel wacht op programmeerwerk en op een keuze welke woorden als "uitsluitend algemene termen" gelden.

Een code komt pas in de dekking te staan wanneer de gegevens waarop hij oordeelt ook in te vullen zijn. Bewaking certificeren die op producteigen gegevens nooit kan aanslaan, is erger dan een lege plek: het lege vakje vraagt om werk, het gevulde vakje sust.

**De berichttermijn van artikel 36 lid 2 wordt niet bewaakt.** De verordening verlangt dat de toezichthouder binnen één maand na ontvangst van het verzoek meldt dát zij verlengt. De termijnenmotor kent alleen "binnen de oorspronkelijke termijn" en niet "binnen een eigen termijn na het anker", dus een verlenging wordt op elk moment aanvaard. Een verlenging die te laat is aangekondigd, wordt daarmee niet opgemerkt.

**De feestdagenkalender reikt tot en met 2030.** Een raadpleging die daarna wordt ingediend, laat de berekening luid vastlopen in plaats van een verkeerde datum op te leveren. Dat is de juiste kant om op te falen, maar de gebruiker krijgt het op het slechtste moment. Er staat inmiddels een wekker in de tests: `de_kalender_reikt_ver_genoeg_vooruit_vanaf_vandaag` gaat af zodra de dekking minder dan drie jaar vooruit reikt — met het huidige pakket op 1 januari 2028, twee jaar voordat het werkelijk knelt.

**De instrumentcontrole leest alleen wat er in het kennispakket staat.** `dpofg doorgifte controleer` houdt elke doorgifte tegen de status van haar instrument aan. Die status komt uit het meegeleverde pakket en wordt niet ergens opgehaald — het programma opent geen verbinding. Blijft het pakket achter bij de werkelijkheid, dan meldt de controle een geldigheid die er niet meer is.

### Waar de tool ophoudt

**De zoekorkestratie reikt tot waar het register reikt.** `dpofg verzoek vindplaatsen` leidt af waar gegevens kunnen staan uit de registerregels, en meldt hoeveel conceptregels zijn overgeslagen. Wat niet in het register staat, wordt hier niet gevonden — en dat is precies de fout die de onderzoeksbasis het vaakst aanwijst. Een koppeling naar systemen buiten het register bestaat nog niet.

**De tool redigeert niet zelf, en controleert maar één van de drie dingen machinaal.** De redactieregie wijst aan wát er weg moet en levert uit aan een extern hulpmiddel; het bewerken van een tekstlaag of het zwart maken van beeld blijft daarbuiten. Van de drie terugleescontroles kan het programma er precies één zelf: zoeken of de letterlijke waarden nog in de bytes van het teruggeleverde bestand staan. Dat vindt de meest gemaakte fout — een zwart vlak over tekst die in de tekstlaag blijft staan — en het vindt hem **niet** wanneer die tekst in een samengedrukte stroom zit. De metagegevens en het beeld moeten buiten de tool worden gecontroleerd. Een controle die niet is uitgevoerd telt niet als geslaagd en houdt de verstrekking tegen, tot een tweede persoon vastlegt dat hij het heeft nagekeken.

**De veldmapping leest geen systemen uit.** Zij vergelijkt een lijst veldnamen met de categorieën in de registerregel. Die lijst komt van de beheerder van het bronsysteem; de tool legt geen verbinding en ontleedt geen bestandsformaat. Dat is bewust: een importeur per bron is drie keer hetzelfde onderhoud voor hetzelfde probleem, en het openen van een verbinding botst met het uitgangspunt dat het programma nooit een netwerkverbinding opent.

### Platform en bouwstraat

**De netwerkstiltetest draait alleen op Linux.** De bouwstraat controleert met `strace` dat het programma geen verbinding opent. Voor macOS en Windows is een gelijkwaardige controle nodig.

**De platformmatrix draait wekelijks en niet bij elke duw.** Zie [Waar de draad ligt](#waar-de-draad-ligt). Een fout die alleen op macOS of Windows optreedt, komt daardoor tot een week later aan het licht.

---

## Cijfers

Gemeten op 21 augustus 2026.

| | |
|---|---|
| Rust-code | 26.503 regels productiecode in `src`, zonder commentaar en lege regels |
| Testcode | 5.141 regels in `#[cfg(test)]`-blokken, plus zes mappen met integratietests |
| Tests | 811 testfuncties, 815 uitgevoerde tests |
| Schil | 45 componenttests en 15 motortests |
| Documentatie | 5.984 regels |
| Crates | 11 |
| Clippy | geen waarschuwingen met `-D warnings` |

De verhouding tussen code en tests is bewust hoog. Bij een product waarvan de kernfunctie het bewaken van wettelijke termijnen is, is een rekenfout die niemand opmerkt de duurste fout die er is.

---

## Bevindingen die het nalopen heeft opgeleverd

Deze staan hier omdat ze laten zien waar het misging, en waarom de tests er zijn. De laatste vijf komen niet uit een test maar uit inspectie — een test die groen blijft terwijl hij niets meer bewijst, meldt zichzelf niet.

| Bevinding | Waar |
|---|---|
| De kalenderdekking werd niet gecontroleerd als verlenging niet nodig was, waardoor een feestdag buiten het venster stilzwijgend als werkdag gold | termijnenmotor |
| Opschorting werd in absolute uren verrekend, waardoor een deadline na een zomertijdovergang op 22:59 belandde | termijnenmotor |
| De volledigheidsteller meldde "1 van de 8" bij een leeg record, omdat de motivering alleen werd geteld als de grondslag al was gekozen | domeinmodel |
| 28 van de toen 55 controleregels blokkeerde — meer dan de helft, precies het patroon dat mensen leert wegklikken; de catalogus telt er inmiddels 78, waarvan 23 blokkerend | regelmotor |
| Het signaalmoment van een incident was niet op te geven, waardoor een melding van gisteren niet te registreren was | bedieningsschil |
| De duurweergave rondde twintig minuten af tot "0 uur", precies waar precisie telt | bedieningsschil |
| De voortgangsbalk kon onderlopen bij een teller boven het totaal | bedieningsschil |
| De publicatieroute van de installatiesleutel gaf de klare-tekstkolom ongetoetst terug, zodat één databasebewerking zónder wachtwoord de organisatie een vreemde sleutel liet publiceren | opslaglaag |
| `dpofg-verify logboek --sleutel` zonder `--anker` meldde groen terwijl de sleutel nergens mee was vergeleken | verificatiebinary |
| Een gemanipuleerd dossier van de eigen installatie werd gemeld als "ondertekend met een andere sleutel", wat de lezer naar de onschuldige verklaring duwt | verificatiebinary |
| Het wachtwoordloos voorlezen van de sleutel legde toch een WAL-index aan en brak daarmee op een alleen-lezen medium — precies het geval waarvoor die opdracht bestaat | opslaglaag |
| `logboek toon` brak af op een dossierpad met een accent, doordat er op bytes en niet op tekens werd afgekapt | bedieningsschil |
| GRO-04 en GRO-05 werden al beoordeeld maar stonden niet in de dekkingslijst, waardoor de opdracht die moet zeggen wát er bewaakt wordt, onder haar eigen stand meldde | regelmotor |
| LEK-15 sloeg aan op elk incident terwijl er geen enkele opdracht bestond om een incident aan een verwerking te koppelen — honderd procent onwegneembare meldingen bij iemand die er niets aan kon doen | regelmotor |
| LEK-12 stond als bewaakt aangemerkt terwijl er geen route was om een incident af te ronden, waardoor de regel op producteigen gegevens nooit kon aanslaan | regelmotor |
| `register vul --veld bsn --waarde Ja` legde stilzwijgend 'nee' vast, doordat er letterlijk op "ja" werd vergeleken | bedieningsschil |
| Een lege waarde voor een verplicht veld werd als ingevuld vastgelegd en haalde daarmee de bevinding weg zonder dat er iets was opgelost | bedieningsschil |
| De termijn voor voorafgaande raadpleging verwees naar artikel 36 lid 3, terwijl de acht weken, de verlenging, de berichttermijn en de opschortingsgrond alle in lid 2 staan; lid 3 somt op welke stukken bij het verzoek gaan | kennispakket |
| De feestdagenkalender reikte tot en met 2027, waardoor een termijn van acht weken die na half november 2027 werd gestart niet meer te berekenen was | kennispakket |
| De sleutelafleiding kostte in een ongeoptimaliseerde bouw 2,3 seconden per opdracht, waardoor de integratietests bijna acht minuten liepen; alleen de cryptografische afhankelijkheden optimaliseren bracht dat terug tot 21 seconden zonder één regel eigen code te wijzigen | bouwstraat |
| Hervatten en verlengen schreven de klok weg vóórdat de nieuwe einddatum was berekend; faalde die berekening, dan stond er een klok in de kluis die het dossier onleesbaar maakte | effectbeoordeling |
| Een opschorting die nog liep werd bij het afronden niet gesloten, waardoor de einddatum van een afgesloten dossier elke dag verder opschoof — ook in een dossier dat al was uitgeleverd | termijnenmotor |
| Een tweede effectbeoordeling op dezelfde registerregel kaapte de terugverwijzing, waarna een risicowijziging stilzwijgend aan de eerste voorbijging | effectbeoordeling |
| Een advies met een datum in de toekomst werd aanvaard en zette daarmee de bewaking van de raadplegingstermijn uit | effectbeoordeling |
| De controleronde sloeg een termijn die zij niet kon berekenen stilzwijgend over en telde het dossier toch als beoordeeld | regelmotor |
| De verwachting dat een verzoek van 31 januari op 28 februari verstrijkt bleek te naïef: die dag is een zaterdag, dus komt de maandeindeklem en dáárna de verlenging naar de eerstvolgende werkdag | verzoekdossier |
| Twintig integratietests ankerden op vaste datums in de toekomst, terwijl het product hun uitkomst tegen *nu* afzet; de eerste zou op 24 september 2026 zijn omgevallen, negen op 1 juni 2027 | testsuite |
| Een `moet_falen` zou vanaf 1 december 2026 zijn gaan falen op de einddatum in plaats van op de grond die hij toetst — groen blijven terwijl hij niets meer bewijst | testsuite |
| De kalendertest mat het verschil tussen twee vaste getallen uit het pakket zelf, en zou dus nooit afgaan; hij zou in 2029 nog "vier jaar dekking" hebben gemeld over een kalender die het lopende jaar nauwelijks haalde | kennispakket |
| De grafische schil draaide twee van de veertien beoordelingen van de controleronde en zei daar niets over; leveranciers, risicobeoordelingen, incidenten, effectbeoordelingen, doorgiften en correcties ontbraken in het venster terwijl het scherm er compleet uitzag | grafische schil |
| De drempels van de zorgplichtcontrolset stonden in de schil hardgecodeerd, terwijl de opdrachtregel ze uit het kennispakket haalt — een tweede waarheid die pas bij het eerste bijgestelde pakket zichtbaar zou worden | grafische schil |
| `cargo deny` liet onbetrouwbare code in de afhankelijkheden van Tauri stilzwijgend door, doordat de standaardinstelling voor `unsound` alleen naar rechtstreekse afhankelijkheden kijkt; de melding over glib werd door GitHub wél en door de eigen bouwstraat níet gezien | bouwstraat |
