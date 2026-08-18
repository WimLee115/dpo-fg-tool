# Documentatie

| Document | Inhoud | Omvang |
|---|---|---|
| [PLAN.md](PLAN.md) | Het projectplan: aanleiding en afbakening, doelgroep, wettelijk kader met de resolvers voor toepassingsbereik en bevoegde autoriteit, de modules, systeem- en beveiligingsarchitectuur, datamodel, compliance-mapping, roadmap, kwaliteitsborging, risico's en wat de tool bewust niet doet | ~1450 regels |
| [FOUTBESTENDIGHEID.md](FOUTBESTENDIGHEID.md) | Foutbestendig ontwerp. De ontwerpladder van sterk naar zwak, foutbestendiging per werkproces, interactiepatronen, ingebouwde kennis, 123 continu draaiende controleregels, meetnormen en acceptatiecriteria | ~1540 regels |
| [PLATFORMONDERSTEUNING.md](PLATFORMONDERSTEUNING.md) | Linux, macOS en Windows: ondersteunde versies, webviewverschillen, sleutel- en geheimenopslag, hardwaretokens, bestandslocaties, valkuilen van het bestandssysteem, ondertekening en distributie, bijwerken, netwerkstilte en de testmatrix | ~1030 regels |
| [REVIEW.md](REVIEW.md) | De kritische toetsing door een privacyjurist met toezichtachtergrond en een beveiligingsarchitect, waarop herziening 2.0 van het plan berust | ~370 regels |
| [FORMAAT.md](FORMAAT.md) | De bestandsformaten, zo beschreven dat een toezichthouder een dossier kan controleren zonder software van de organisatie die het aanlevert | ~210 regels |

## Leesvolgorde

1. **PLAN.md** hoofdstuk 1 tot en met 4 — waarom de tool bestaat, voor wie, welk recht hij bedient en wat hij doet.
2. **FOUTBESTENDIGHEID.md** hoofdstuk 0 en 1 — de toetsnorm waaraan elk scherm moet voldoen.
3. **PLAN.md** hoofdstuk 5 tot en met 8 — architectuur, beveiliging, datamodel en de koppeling met de normenkaders.
4. **PLATFORMONDERSTEUNING.md** — wat de drie platforms concreet afdwingen.
5. **PLAN.md** hoofdstuk 9 tot en met 12 — planning, kwaliteitsborging, risico's en afbakening.

## Status van de documenten

De documenten beschrijven het ontwerp, niet de gebouwde werkelijkheid. Waar een keuze nog openstaat, is dat als beslispunt met een datum vastgelegd in bijlage B van het plan.

Wijzigingen lopen via een genummerde herziening met vermelding van datum en reden; de vorige versie blijft bewaard in de geschiedenis van de repository.
