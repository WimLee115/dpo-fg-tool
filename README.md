# dpo-fg-tool

**Lokaal draaiend, sterk beveiligd werkplatform voor de Functionaris Gegevensbescherming (FG) en Data Protection Officer (DPO) — AVG-, UAVG- en NIS2-compliance in één dossier.**

---

## Waarom deze tool

Privacy- en securitycompliance wordt in de praktijk beheerd in spreadsheets, gedeelde mappen en dure cloudsuites. Alle drie zijn ongeschikt voor het materiaal dat een FG beheert: het verwerkingsregister, DPIA's, datalekdossiers, kwetsbaarheden en incidentmeldingen vormen samen een volledige plattegrond van de zwakke plekken van een organisatie. Dat dossier hoort niet ongevraagd bij een derde partij te staan.

`dpo-fg-tool` draait daarom **op de eigen machine of de eigen server**, zonder verplichte cloud, zonder telemetrie en zonder accountdwang. De gegevens blijven versleuteld op de schijf van de organisatie. Het programma opent uit zichzelf nooit een netwerkverbinding.

Daarbij geldt één eis boven alle andere: **de gebruiker moet de fout niet kunnen maken.** Waar een fout onmogelijk gemaakt kan worden, wordt hij onmogelijk gemaakt — waarschuwen is de laatste maatregel, niet de eerste.

De tool draait op **Linux, macOS en Windows** als gelijkwaardige platforms.

## Uitgangspunten

| Principe | Betekenis |
|---|---|
| **Lokaal tenzij** | Geen enkele functie vereist een internetverbinding. Optionele online bronnen zijn per stuk uit te zetten en standaard uit. |
| **Versleuteld in rust** | De volledige gegevensopslag is versleuteld; de sleutel wordt afgeleid uit een wachtwoord en optioneel een hardwaretoken. |
| **Aantoonbaar** | Elke wijziging landt in een manipulatiebestendig auditspoor, zodat verantwoording richting toezichthouder te onderbouwen is. |
| **Foutbestendig** | Verplichtingen volgen uit de gegeven antwoorden; de gebruiker hoeft de regel niet te kennen. Onvolledigheid is een teller, geen foutmelding. |
| **Minimale aanvalsoppervlakte** | Weinig afhankelijkheden, geheugenveilige backend, strikte contentbeveiliging, geen automatische updates zonder handtekeningcontrole. |
| **Geen vendor lock-in** | Alles is exporteerbaar in open formaten. De gegevens zijn en blijven van de organisatie. |
| **Nederlands eerst** | Terminologie, formulieren en sjablonen volgen de Nederlandse praktijk en de richtsnoeren van de Autoriteit Persoonsgegevens. |

## Snel beginnen

```sh
cargo build --release

# Een nieuwe kluis aanmaken
./target/release/dpofg kluis nieuw

# Een registerregel opbouwen — de tool zegt bij elke stap wat er nog ontbreekt
./target/release/dpofg register nieuw 0412-K "Verzuimregistratie" --eigenaar "afdeling P&O"
./target/release/dpofg register vul 0412-K --veld grondslag --waarde gerechtvaardigd-belang
./target/release/dpofg register vaststellen 0412-K

# Een incident registreren; de klokken gaan lopen op hun eigen ankers
./target/release/dpofg incident nieuw 2026-0041 "onbevoegde inzage" --signaal 2026-08-18T09:00:00Z
./target/release/dpofg incident kennisname 2026-0041 2026-08-18T09:20:00Z
./target/release/dpofg incident toon 2026-0041

# De controleregels over de hele verzameling
./target/release/dpofg controle

# Het logboek controleren en verankeren
./target/release/dpofg logboek verifieer
./target/release/dpofg logboek anker --bewaarplaats "notulen directieoverleg"

# Een dossier samenstellen voor een toezichthouder…
./target/release/dpofg dossier ./uitvraag \
  --aanleiding "uitvraag van 12 augustus" --bestemd-voor "de toezichthouder"

# …dat de ontvanger controleert zonder de kluis en zonder wachtwoord
./target/release/dpofg-verify dossier ./uitvraag/manifest.json
```

Het wachtwoord wordt nooit als argument aangenomen: het wordt gevraagd, of gelezen uit `DPOFG_WACHTWOORD` voor geautomatiseerd gebruik.

## Stand van zaken

Een uitgebreid overzicht van wat er werkt, wat er niet werkt en waar de grenzen liggen staat in [`STAND.md`](STAND.md).

De kern staat en is te gebruiken via de opdrachtregel. De grafische schil komt later; de opdrachtregel is bewust eerst gebouwd, omdat die afdwingt dat de logica in de lagen eronder zit en niet in een scherm.

| Onderdeel | Stand |
|---|---|
| Versleutelde kluis, sleutelhiërarchie, compartimenten | werkt |
| Manipulatiebestendig ketenlogboek met ankers | werkt |
| Termijnenmotor met getypeerde termijnen | werkt |
| Verwerkingsregister (art. 30 AVG) met afgeleide verplichtingen | werkt |
| Incidentdossier met de vijf klokken en het meldbesluit | werkt |
| Controleregels over de samenhang | 21 van de 55 regels draaien |
| Kennispakketten met handtekening en terugrolbescherming | werkt |
| Dossiers samenstellen en ondertekenen | werkt |
| Losse verificatiebinary voor toezichthouders | werkt |
| Betrokkenenverzoeken, DPIA, leveranciersregister | nog niet |
| Grafische schil | nog niet |

Vraag de werkelijke dekking op met `dpofg controle --dekking`. Het aantal regels in de catalogus zegt niets over wat er wordt bewaakt; die opdracht wel.

## Opbouw

| Crate | Verantwoordelijkheid |
|---|---|
| `dpofg-crypto` | sleutelafleiding, envelopversleuteling, sleutelhiërarchie, blinde index |
| `dpofg-audit` | ketenlogboek, ankers, verificatie |
| `dpofg-terms` | termijnenmotor: getypeerde termijnen, kalenderrekenkunde, opschorting |
| `dpofg-domain` | domeinmodel: registers, incidenten, volledigheid, afgeleide verplichtingen |
| `dpofg-store` | versleutelde opslag, versiegeschiedenis, bijlagen |
| `dpofg-content` | kennispakketten met de juridische inhoud |
| `dpofg-rules` | controleregels en waarschuwingsbudget |
| `dpofg-report` | dossiers samenstellen en ondertekenen |
| `dpofg-cli` | bedieningsschil |
| `dpofg-verify` | losse verificatiebinary; leest uitsluitend, vraagt geen wachtwoord |

## Documentatie

| Document | Inhoud |
|---|---|
| [`docs/PLAN.md`](docs/PLAN.md) | Het projectplan: afbakening, wettelijk kader, modules, architectuur, datamodel, compliance-mapping, roadmap en risico's |
| [`docs/FOUTBESTENDIGHEID.md`](docs/FOUTBESTENDIGHEID.md) | Foutbestendig ontwerp: de ontwerpladder, foutbestendiging per werkproces, interactiepatronen en de controleregels |
| [`docs/PLATFORMONDERSTEUNING.md`](docs/PLATFORMONDERSTEUNING.md) | Linux, macOS en Windows: sleutelopslag, hardwaretokens, ondertekening, distributie en de testmatrix |
| [`docs/REVIEW.md`](docs/REVIEW.md) | De juridische en beveiligingstoetsing waarop het plan berust |
| [`docs/FORMAAT.md`](docs/FORMAAT.md) | De bestandsformaten, zodat een toezichthouder een dossier kan controleren zonder software van de aanleverende organisatie |

## Reikwijdte

De tool ondersteunt het volledige werkproces van de FG en de security officer:

- Verwerkingsregister conform artikel 30 AVG, voor verwerkingsverantwoordelijke én verwerker
- Gegevensbeschermingseffectbeoordeling (DPIA/GEB) conform artikel 35 AVG
- Datalekregister en meldproces conform artikel 33 en 34 AVG, met bewaking van de 72-uurstermijn
- Verzoeken van betrokkenen conform artikel 15 tot en met 22 AVG, met termijnbewaking
- Verwerkersovereenkomsten en leveranciersbeheer conform artikel 28 AVG
- Doorgiften buiten de EER conform hoofdstuk V AVG
- NIS2-zorgplicht: risicobeheer, maatregelenregister en meldketen met de wettelijke termijnen
- Beheersmaatregelen en bewijsvoering, gekoppeld aan gangbare normenkaders
- Rapportage aan directie, bestuur en toezichthouder

## Voorbehoud bij de juridische inhoud

Het meegeleverde kennispakket is een **vertrekpunt, geen bron van recht**. De termijnen, feestdagen en grondslagen daarin zijn niet door een jurist vastgesteld en niet gecontroleerd tegen de geconsolideerde wettekst. Verifieer elk onderdeel tegen de bron voordat u erop vertrouwt; `dpofg pakket voorbehoud` toont wat er te controleren valt.

De consolidatiedatum van het pakket gaat mee in elke export en elk dossier, zodat zichtbaar is op welke stand van het recht een berekening berust.

## Beveiliging

Kwetsbaarheden meld je volgens de procedure in [`SECURITY.md`](SECURITY.md). Meld ze niet via een openbare issue.

## Licentie

Zie [`LICENSE`](LICENSE). Auteursrecht © 2026 WimLee115.

## Auteur

Ontwikkeld en onderhouden door **WimLee115**.
