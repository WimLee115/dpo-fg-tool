# Documentatie

Zeven documenten, deze index niet meegeteld. Vier beschrijven het ontwerp, twee leggen een besluit vast, en één — [FORMAAT.md](FORMAAT.md) — is bedoeld voor iemand buiten de organisatie.

## Wat er ligt

### Het ontwerp

| Document | Inhoud | Omvang |
|---|---|---|
| [PLAN.md](PLAN.md) | Het projectplan: aanleiding en afbakening, doelgroep, wettelijk kader met de resolvers voor toepassingsbereik en bevoegde autoriteit, de modules, systeem- en beveiligingsarchitectuur, datamodel, compliance-mapping, roadmap, kwaliteitsborging, risico's en wat de tool bewust niet doet | ~1470 regels |
| [FOUTBESTENDIGHEID.md](FOUTBESTENDIGHEID.md) | Foutbestendig ontwerp: de ontwerpladder van sterk naar zwak, foutbestendiging per werkproces, interactiepatronen, ingebouwde kennis, de catalogus van 147 continu draaiende controleregels, meetnormen en acceptatiecriteria | ~1590 regels |
| [PLATFORMONDERSTEUNING.md](PLATFORMONDERSTEUNING.md) | Linux, macOS en Windows: ondersteunde versies, webviewverschillen, sleutel- en geheimenopslag, hardwaretokens, bestandslocaties, valkuilen van het bestandssysteem, ondertekening en distributie, bijwerken, netwerkstilte en de testmatrix | ~1030 regels |
| [SCHIL.md](SCHIL.md) | Het ontwerp van de grafische schil: de werkbak als enige beginpunt, wat elk scherm belooft en wat het uitdrukkelijk níet zegt, en waar de schil ophoudt | ~450 regels |

### De besluiten waarop dat ontwerp berust

| Document | Inhoud | Omvang |
|---|---|---|
| [REVIEW.md](REVIEW.md) | De kritische toetsing door een privacyjurist met toezichtachtergrond en een beveiligingsarchitect, waarop herziening 2.0 van het plan berust | ~370 regels |
| [RAAMWERK.md](RAAMWERK.md) | De keuze van het frontendraamwerk, met de twee afgewezen alternatieven en wat de keuze kost | ~305 regels |

### Voor buiten de organisatie

| Document | Inhoud | Omvang |
|---|---|---|
| [FORMAAT.md](FORMAAT.md) | De bestandsformaten, zo beschreven dat een toezichthouder een dossier kan controleren zonder software van de organisatie die het aanlevert | ~240 regels |

De stand van de bouw staat niet hier maar in [`../STAND.md`](../STAND.md): wat er werkt, wat er niet werkt, en waar de draad ligt.

## Leesvolgorde

1. **PLAN.md** hoofdstuk 1 tot en met 4 — waarom de tool bestaat, voor wie, welk recht hij bedient en wat hij doet.
2. **FOUTBESTENDIGHEID.md** hoofdstuk 0 en 1 — de toetsnorm waaraan elk scherm moet voldoen.
3. **PLAN.md** hoofdstuk 5 tot en met 8 — architectuur, beveiliging, datamodel en de koppeling met de normenkaders.
4. **PLATFORMONDERSTEUNING.md** — wat de drie platforms concreet afdwingen.
5. **PLAN.md** hoofdstuk 9 tot en met 12 — planning, kwaliteitsborging, risico's en afbakening.

Wie alleen aan de schil werkt, leest **SCHIL.md** en daarna **RAAMWERK.md**. Los van de rest staan ze niet: het raamwerkbesluit is getoetst tegen PLATFORMONDERSTEUNING.md §1 tot en met §11, en de schermeisen komen uit FOUTBESTENDIGHEID.md §0.3, §3.4 en §3.6. Houd die bij de hand.

## Status van de documenten

**Deze documenten beschrijven het ontwerp, niet de gebouwde werkelijkheid.** Dat onderscheid is niet vrijblijvend en het is ook geen excuus: het betekent dat een verschil tussen document en code een *ontwerpafwijking* kan zijn, maar dat een document nog steeds niet zichzelf mag tegenspreken, geen getal mag dragen dat het elders heeft herroepen, en niet in de tegenwoordige tijd mag beweren dat de code iets doet wat zij niet doet.

De regelcatalogus in FOUTBESTENDIGHEID.md telt 147 regels; de gebouwde catalogus is kleiner en groeit ernaartoe. Vraag de werkelijke stand op met `dpofg controle --dekking` — het ontwerp zegt wat er zou moeten worden bewaakt, die opdracht wat er werkelijk wordt bewaakt.

Waar een keuze nog openstaat, is dat als beslispunt vastgelegd in bijlage B van het plan — met een vast beslismoment en een criterium, niet met een kalenderdatum. Het plan houdt vaste datums er bewust buiten (§3.2): één verschoven datum zou anders een uitgave afdwingen. Wijzigingen lopen via een genummerde herziening met vermelding van datum en reden; de vorige versie blijft bewaard in de geschiedenis van de repository.
