# Stand van de bouw

Bijgewerkt op 18 augustus 2026.

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
| Controleregels | **deels** | 21 van de 55 regels in de catalogus hebben een evaluatiefunctie |
| Kennispakketten | werkt | Ondertekend, met terugrolbescherming en consolidatiedatum |
| Dossiers | werkt | Ondertekend manifest, weglatingen zichtbaar, voorbehoud meegetekend |
| Verificatiebinary | werkt | Leest uitsluitend, geen wachtwoord, geen kluis nodig |
| Bedieningsschil | werkt | Volledig werkproces via de opdrachtregel |

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

**Sleutelbeheer voor handtekeningen ontbreekt.** Ankers en dossiermanifesten worden per stuk met een nieuw sleutelpaar ondertekend. Dat toont aan dát er is ondertekend en dat de inhoud niet is gewijzigd, maar niet dat het van deze installatie komt. Een vaste installatiesleutel is nodig voordat dit bij een toezichthouder standhoudt.

**Compartimenten hangen aan één wachtwoord.** De scheiding is cryptografisch echt — elk compartiment heeft een eigen sleutel — maar alle sleutels hangen aan dezelfde kluissleutel. Het persoonlijke dossier van de functionaris, dat de organisatie níet moet kunnen openen, vereist een tweede wachtwoord.

**De regelcatalogus is groter dan wat er draait.** 55 regels gedefinieerd, 21 met een evaluatiefunctie. `dpofg controle --dekking` toont welke nog niet draaien. Het aantal regels zegt niets over wat er wordt bewaakt; die opdracht wel.

**Geen hardwaretoken.** Het platformhoofdstuk beschrijft hoe FIDO2 en PIV per besturingssysteem werken; de implementatie ontbreekt.

**De netwerkstiltetest draait alleen op Linux.** De bouwstraat controleert met `strace` dat het programma geen verbinding opent. Voor macOS en Windows is een gelijkwaardige controle nodig.

---

## Cijfers

| | |
|---|---|
| Rust-code | ~9.800 regels, zonder commentaar en lege regels |
| Tests | 319 testfuncties |
| Documentatie | ~4.600 regels |
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
