# Stand van de bouw

Bijgewerkt op 19 augustus 2026.

Dit document zegt wat er **werkt**, wat er **niet werkt**, en waar de grenzen liggen. Het is bewust nuchter: een overzicht dat meer suggereert dan er staat, kost bij de eerste inspectie meer dan het oplevert.

---

## Wat er staat

### Fase 0 — Kluiskern

| Onderdeel | Stand | Toelichting |
|---|---|---|
| Sleutelafleiding | werkt | Argon2id, parameters in de kluis zodat verzwaren later mogelijk blijft |
| Envelopversleuteling | werkt | XChaCha20-Poly1305, elke envelop gebonden aan veld, record en compartiment |
| Sleutelhiërarchie | werkt | Drie lagen; wachtwoord wijzigen herversleutelt niets |
| Compartimenten | werkt | Cryptografisch gescheiden, met rotatie per compartiment |
| Blinde index | werkt | Zoeken in versleutelde velden; weigert velden met lage variatie |
| Ketenlogboek | werkt | Hashketen, append-only tot in de database via triggers |
| Ankers | werkt | Ed25519; het enige middel tegen afkappen aan het einde |
| Termijnenmotor | werkt | Getypeerde termijnen, maandeindeklem, opschorting in kalenderdagen |
| Versleutelde opslag | werkt | Versiegeschiedenis, bijlagen inhoudsgeadresseerd |

### Fase 1 — Register, incident en aantoonbaarheid

| Onderdeel | Stand | Toelichting |
|---|---|---|
| Verwerkingsregister | werkt | Artikel 30 AVG, beide schema's, met afgeleide verplichtingen |
| Volledigheidsmechanisme | werkt | Teller met grondslag; blokkerend onderscheiden van signalerend |
| Incidentdossier | werkt | Vijf klokken op eigen ankers, meldbesluit met drie lagen |
| Klokkenmotor | werkt | Leidt verplichtingen af; ankers vallen niet samen |
| Controleregels | **deels** | 30 van de 55 regels in de catalogus hebben een evaluatiefunctie |
| Kennispakketten | werkt | Ondertekend, met terugrolbescherming en consolidatiedatum |
| Dossiers | werkt | Ondertekend manifest, weglatingen zichtbaar, voorbehoud meegetekend |
| Verificatiebinary | werkt | Leest uitsluitend, geen wachtwoord, geen kluis nodig |
| Bedieningsschil | werkt | Volledig werkproces via de opdrachtregel |
| Installatiesleutel | werkt | Eén vaste ondertekenidentiteit per kluis; overleeft een wachtwoordwissel |

---

## Wat er niet staat

| Onderdeel | Uit fase | Waarom het er nog niet is |
|---|---|---|
| Betrokkenenverzoeken | 2 | Vereist zoekorkestratie over systemen en redactieregie; dat is een eigen bouwslag |
| Effectbeoordeling (DPIA) als dossier | 2 | De criteria worden geteld en getoond; het dossier zelf ontbreekt |
| Belangenafweging, toestemming, doorgifte als eigen records | 2 | De verwijzingen bestaan in het model; de records nog niet |
| Leveranciers- en ketenregister | 3 | |
| Zorgplichtcontrolset | 3 | Vereist de normenkaders in het kennispakket |
| Toezichtdossier en bestuursrechtelijk spoor | 4 | |
| Vervalprognose | 3 | Vereist geldigheidsvensters op bewijsstukken |
| Ketenbewijs tussen organisaties | 4 | |
| Persoonlijk dossier van de functionaris | 1 | De cryptografische structuur ligt er; een tweede wachtwoord ontbreekt |
| Grafische schil | 1 | De opdrachtregel is bewust eerst gebouwd |
| Multi-entiteit | 5 | |

---

## Grenzen die benoemd horen te worden

**De juridische inhoud is niet vastgesteld.** Het meegeleverde kennispakket is een vertrekpunt. De termijnen, feestdagen en grondslagen zijn niet door een jurist gecontroleerd tegen de geconsolideerde wettekst. `dpofg pakket voorbehoud` toont wat er te verifiëren valt.

**De installatiesleutel is er; rotatie en intrekking niet.** Ankers en dossiermanifesten dragen sinds deze uitgave één vaste sleutel per kluisbestand, te tonen met `dpofg kluis sleutel` en te controleren met `dpofg-verify --sleutel`. Wat daar niet in zit:

* **Geen rotatie en geen intrekking.** Raakt de sleutel weg, dan is de enige weg een nieuwe publiceren en de oude ernaast blijven vermelden. Het schema draagt al een generatiekolom en de verificatiebinary aanvaardt al meerdere sleutels, dus rotatie is later toe te voegen zonder formaatwijziging.
* **"Installatie" betekent dit kluisbestand.** Twee kluizen op één machine krijgen twee identiteiten. Een kopie van een kluis draagt dezelfde privésleutel en is er cryptografisch niet van te onderscheiden. Wat er wél bij komt: twee kopieën die doortekenen leveren twee ankers op met hetzelfde volgnummer, een verschillende hash en dezelfde ondertekenaar — die vork is nu aan de installatie toe te schrijven.
* **De sleutel is zo sterk als kluisbestand plus wachtwoordzin.** Wie beide heeft, kan ondertekenen namens de organisatie. Vóór deze uitgave viel er niets te stelen; nu wel.
* **De bewaarplaats van een anker valt buiten de handtekening** en is dus na ondertekening aan te passen. Dat wacht op een formaatversie 2, samen met een installatie-identificatie in de ondertekende bytes; twee formaatbumps waar één volstaat, is de duurste manier om een gepubliceerde specificatie te onderhouden.

**Compartimenten hangen aan één wachtwoord.** De scheiding is cryptografisch echt — elk compartiment heeft een eigen sleutel — maar alle sleutels hangen aan dezelfde kluissleutel. Het persoonlijke dossier van de functionaris, dat de organisatie níet moet kunnen openen, vereist een tweede wachtwoord.

**De regelcatalogus is groter dan wat er draait.** 55 regels gedefinieerd, 30 met een evaluatiefunctie. `dpofg controle --dekking` toont welke nog niet draaien. Het aantal regels zegt niets over wat er wordt bewaakt; die opdracht wel.

De 25 regels die nog niet draaien wachten bijna allemaal op een recordsoort die er niet is, en niet op programmeerwerk. De twee die het meeste opleveren: het verwerkersregister met de leveranciersgegevens ontsluit vijf regels, het effectbeoordelingsdossier drie van de vier DPIA-regels — DPIA-01 draait al. Daarnaast wachten er regels op een systeemregister, een doorgifterecord, een schoningsopdracht, een rollen- en aanstellingsregister, een rapportagespoor en een toezichtdossier.

Een code komt pas in de dekking te staan wanneer de gegevens waarop hij oordeelt ook in te vullen zijn. Bewaking certificeren die op producteigen gegevens nooit kan aanslaan, is erger dan een lege plek: het lege vakje vraagt om werk, het gevulde vakje sust.

**Schemaversie 2 is eenrichtingsverkeer.** Een kluis die één keer met deze uitgave is geopend, staat op schemaversie 2 en wordt door uitgave 0.1.0 geweigerd — met de juiste melding, want het bestandsmerk blijft gelijk. Terug kan alleen met een reservekopie. Een logboek van een kluis die uit schemaversie 1 is gemigreerd, draagt bovendien de handelingsnaam `installatiesleutel_aangemaakt`; een oudere `dpofg-verify` loopt daarop stuk met afsluitcode 1 — luidruchtig, en dus acceptabel. Een kluis die met deze uitgave is aangemaakt, kent die regel niet: daar staat de sleutel in de omschrijving van `kluis_aangemaakt`, en dat leest een oudere binary gewoon.

**Geen hardwaretoken.** Het platformhoofdstuk beschrijft hoe FIDO2 en PIV per besturingssysteem werken; de implementatie ontbreekt.

**De netwerkstiltetest draait alleen op Linux.** De bouwstraat controleert met `strace` dat het programma geen verbinding opent. Voor macOS en Windows is een gelijkwaardige controle nodig.

---

## Cijfers

| | |
|---|---|
| Rust-code | ~10.950 regels, zonder commentaar en lege regels |
| Tests | 395 testfuncties, 399 uitgevoerde tests |
| Documentatie | ~4.700 regels |
| Crates | 10 |
| Clippy | geen waarschuwingen met `-D warnings` |

De verhouding tussen code en tests is bewust hoog. Bij een product waarvan de kernfunctie het bewaken van wettelijke termijnen is, is een rekenfout die niemand opmerkt de duurste fout die er is.

---

## Bevindingen die het testen heeft opgeleverd

Deze staan hier omdat ze laten zien waar het misging, en waarom de tests er zijn.

| Bevinding | Waar |
|---|---|
| De kalenderdekking werd niet gecontroleerd als verlenging niet nodig was, waardoor een feestdag buiten het venster stilzwijgend als werkdag gold | termijnenmotor |
| Opschorting werd in absolute uren verrekend, waardoor een deadline na een zomertijdovergang op 22:59 belandde | termijnenmotor |
| De volledigheidsteller meldde "1 van de 8" bij een leeg record, omdat de motivering alleen werd geteld als de grondslag al was gekozen | domeinmodel |
| 28 van de 55 controleregels blokkeerde — meer dan de helft, precies het patroon dat mensen leert wegklikken | regelmotor |
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
