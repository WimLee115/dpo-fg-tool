# dpo-fg-tool

**Lokaal draaiend, sterk beveiligd werkplatform voor de Functionaris Gegevensbescherming (FG) en Data Protection Officer (DPO) — AVG-, UAVG- en NIS2-compliance in één dossier.**

---

## Waarom deze tool

Privacy- en securitycompliance wordt in de praktijk beheerd in spreadsheets, gedeelde mappen en dure cloudsuites. Alle drie zijn ongeschikt voor het materiaal dat een FG beheert: het verwerkingsregister, DPIA's, datalekdossiers, kwetsbaarheden en incidentmeldingen vormen samen een volledige plattegrond van de zwakke plekken van een organisatie. Dat dossier hoort niet ongevraagd bij een derde partij te staan.

`dpo-fg-tool` draait daarom **op de eigen machine of de eigen server**, zonder verplichte cloud, zonder telemetrie en zonder accountdwang. De gegevens blijven versleuteld op de schijf van de organisatie. Netwerkverkeer is standaard uitgeschakeld en moet per functie bewust worden aangezet.

## Uitgangspunten

| Principe | Betekenis |
|---|---|
| **Lokaal tenzij** | Geen enkele functie vereist een internetverbinding. Optionele online bronnen zijn per stuk uit te zetten en standaard uit. |
| **Versleuteld in rust** | De volledige gegevensopslag is versleuteld; de sleutel wordt afgeleid uit een wachtwoord en optioneel een hardwaretoken. |
| **Aantoonbaar** | Elke wijziging landt in een manipulatiebestendig auditspoor, zodat verantwoording richting toezichthouder te onderbouwen is. |
| **Minimale aanvalsoppervlakte** | Weinig afhankelijkheden, geheugenveilige backend, strikte contentbeveiliging, geen automatische updates zonder handtekeningcontrole. |
| **Geen vendor lock-in** | Alles is exporteerbaar in open formaten. De gegevens zijn en blijven van de organisatie. |
| **Nederlands eerst** | Terminologie, formulieren en sjablonen volgen de Nederlandse praktijk en de richtsnoeren van de Autoriteit Persoonsgegevens. |

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

Daarbij geldt één eis boven alle andere: **de gebruiker moet de fout niet kunnen maken.** Waar een fout onmogelijk gemaakt kan worden, wordt hij onmogelijk gemaakt — waarschuwen is de laatste maatregel, niet de eerste.

De tool draait op **Linux, macOS en Windows** als gelijkwaardige platforms.

## Documentatie

| Document | Inhoud |
|---|---|
| [`docs/PLAN.md`](docs/PLAN.md) | Het projectplan: afbakening, wettelijk kader, modules, architectuur, datamodel, compliance-mapping, roadmap en risico's |
| [`docs/FOUTBESTENDIGHEID.md`](docs/FOUTBESTENDIGHEID.md) | Foutbestendig ontwerp: de ontwerpladder, foutbestendiging per werkproces, interactiepatronen en 123 continu draaiende controleregels |
| [`docs/PLATFORMONDERSTEUNING.md`](docs/PLATFORMONDERSTEUNING.md) | Linux, macOS en Windows: sleutelopslag, hardwaretokens, ondertekening, distributie en de testmatrix |
| [`docs/REVIEW.md`](docs/REVIEW.md) | De juridische en beveiligingstoetsing waarop het plan berust |

## Status

In ontwikkeling. Architectuur en functioneel ontwerp zijn vastgelegd; de implementatie start met de kern van de beveiligde opslag en het verwerkingsregister.

## Beveiliging

Kwetsbaarheden meld je volgens de procedure in [`SECURITY.md`](SECURITY.md). Meld ze niet via een openbare issue.

## Licentie

Zie [`LICENSE`](LICENSE). Auteursrecht © 2026 WimLee115.

## Auteur

Ontwikkeld en onderhouden door **WimLee115**.
