# dpo-fg-tool

**Projectplan — herziening 2.0**
Opgesteld door en namens **WimLee115** · 18 augustus 2026
Vervangt herziening 1.0 van 18 augustus 2026. Reden van herziening: verwerking van de externe kritische review door een privacyjurist/FG met toezichtachtergrond en een security-architect. De vorige versie blijft bewaard.

> **Leeswijzer bij deze herziening.** Elke bevinding uit de review is verwerkt, weerlegd of expliciet als restpunt benoemd. Bijlage C bevat de volledige herleidbaarheidstabel van bevinding naar sectie. De drie zwaarste conclusies van de review zijn integraal overgenomen: de AVG-kant is verdubbeld, de bewijskrachtclaim is teruggebracht tot wat zij werkelijk waarmaakt plus externe verankering, en de planning is met een factor twee opgehoogd terwijl de scope op vijf punten is versoberd.

---

## Inhoud

1. Aanleiding, doel en bindende afbakening
2. Doelgroep, gebruikers en gebruiksscenario's
3. Wettelijk kader, toepassingsbereik en resolvers
4. Functioneel overzicht: de modules
5. Systeemarchitectuur en techniekkeuzen
6. Beveiligings- en bewijsarchitectuur
7. Datamodel
8. Compliance-mapping
9. Roadmap en inspanningsschatting
10. Kwaliteitsborging
11. Risico's en mitigaties
12. Wat de tool niet doet

Bijlage A — Vaste waarschuwingsteksten
Bijlage B — Beslispunten met een vaste datum
Bijlage C — Verwerking van de kritische review

**Afzonderlijke ontwerphoofdstukken**

- [FOUTBESTENDIGHEID.md](FOUTBESTENDIGHEID.md) — foutbestendig ontwerp: de ontwerpladder, foutbestendiging per werkproces, interactiepatronen, ingebouwde kennis, 123 continu draaiende controleregels en de meetnormen
- [PLATFORMONDERSTEUNING.md](PLATFORMONDERSTEUNING.md) — Linux, macOS en Windows: ondergrenzen, webviewverschillen, sleutelopslag, hardwaretokens, bestandslocaties, ondertekening, distributie en de testmatrix
- [REVIEW.md](REVIEW.md) — de juridische en beveiligingstoetsing waarop deze herziening berust

---

## 1. Aanleiding, doel en bindende afbakening

### 1.1 Aanleiding

Sinds 15 augustus 2026 gelden in Nederland de Cyberbeveiligingswet (Cbw) en het Cyberbeveiligingsbesluit (Cbb). De Cbw vervangt de Wet beveiliging netwerk- en informatiesystemen. Naast de al bestaande verplichtingen uit de Algemene verordening gegevensbescherming (AVG) en de Uitvoeringswet AVG (UAVG) krijgt één en dezelfde functionaris — in de praktijk de functionaris voor gegevensbescherming (FG), vaak in personele unie met de security officer — er een tweede volwaardig regime bij, met eigen termijnen, eigen meldketens, eigen registers en eigen bestuurdersverplichtingen.

Die functionaris werkt vandaag in werkbladen, gedeelde mappen, e-mail en losse hulpmiddelen. Dat werkt niet meer, om drie redenen:

1. **De termijnen zijn hard en meervoudig.** Eén incident kan tegelijk vijf klokken starten, in twee regimes, met verschillende ankers en verschillende rekenregels.
2. **De bewijslast ligt bij de organisatie.** AVG art. 5 lid 2 en Cbb art. 6 lid 4 vragen niet om een gevoel maar om aantoonbaarheid, op het moment dat de toezichthouder het vraagt en over de periode waarover hij het vraagt.
3. **De gegevens die bij dit werk horen zijn zelf hoogrisico.** Inzageverzoeken met bijzondere persoonsgegevens, de identiteit van klagers, het volledige kwetsbaarhedenbeeld en de vertrouwelijke drempels ex Cbb art. 23 lid 4 horen niet in een cloudsuite met een leveranciersketen die de FG niet kan overzien.

`dpo-fg-tool` is een lokaal draaiend, versleuteld dossier- en termijnsysteem voor precies dit werk.

### 1.2 Doel en bindende afbakening

> **Toetsingsprincipe.** De tool ondersteunt uitsluitend *wat de FG en de security officer zelf doen en zelf moeten kunnen aantonen*. Alles wat een andere afdeling doet, wordt niet nagebouwd maar via import en export ontsloten. Elk functieverzoek wordt aan deze zin getoetst en aan risico R8. Sectie 12 is bindend.

Concreet betekent dit drie beloften:

| Belofte | Invulling |
|---|---|
| **Geen egress** | Het programma opent uit zichzelf nooit een netwerkverbinding. Geen telemetrie, geen updatecontrole, geen contentpackophaal, geen crashrapportage. Contentpacks worden als ondertekend bestand geïmporteerd. |
| **Geen leveranciersafhankelijkheid voor toegang** | Er is geen account, geen licentieserver, geen sleutelherstel door de leverancier. De klant kan de tool tien jaar draaien zonder dat de leverancier bestaat. |
| **Geen schijnzekerheid** | Er is geen scherm, rapport of certificaat waarin staat dat een organisatie "voldoet". Elke status benoemt óf bewijs, óf een tekort, óf een openstaand menselijk oordeel. |

### 1.3 Ontwerpprincipes

| # | Principe | Consequentie in het ontwerp |
|---|---|---|
| P1 | Juridische inhoud staat nooit in de binary | Wetteksten, artikelnummers, termijnen, drempels, feestdagenkalenders, autoriteiten, meldkanalen en **alle datums** zitten in ondertekende contentpacks met versie en consolidatiedatum |
| P2 | Elke open norm eindigt bij een mens | De tool doet een voorstel met zichtbare redenering; het besluit draagt naam, tijdstip en motivering |
| P3 | Bewijs vóór status | Een status verandert niet omdat iemand een vinkje zet, maar omdat er een bewijsstuk met een geldigheidsvenster aan hangt |
| P4 | Vastlegging is append-only en extern verankerd | Zie §6.4. De keten alleen bewijst geen tijdstip; verankering wel |
| P5 | Vertrouwelijk is onzichtbaar, niet geblokkeerd | Compartimenten zijn cryptografisch gescheiden, niet met een applicatieregel afgeschermd |
| P6 | Termijnen zijn getypeerd | Uren, dagen, weken, maanden en jaren zijn verschillende typen met verschillende rekenregels; een maandtermijn wordt nooit in dagen omgerekend |
| P7 | Niets wordt namens de organisatie verzonden | De tool stelt samen; de mens verzendt |
| P8 | De tool moet zichzelf kunnen verantwoorden | Meegeleverd: dreigingsmodel, hardeningshandleiding, DPIA op de tool, formaatspecificatie, kwetsbaarhedenbeleid |
| P9 | Wat de tool niet goed kan, doet zij niet | Zie §12 en de versobering in §9 |
| P10 | Alles is exporteerbaar zonder de tool | Open formaatspecificatie, gepubliceerd schema, volledige export in JSON plus bestanden plus manifest |
| P11 | De gebruiker kan de fout niet maken | Waar een fout onmogelijk gemaakt kan worden, wordt hij onmogelijk gemaakt — niet gesignaleerd. Waarschuwen is de zwakste maatregel, niet de eerste. Volledig uitgewerkt in [FOUTBESTENDIGHEID.md](FOUTBESTENDIGHEID.md) |
| P12 | Drie platforms, één gedrag | Linux, macOS en Windows zijn gelijkwaardige doelplatforms. Een functie die op één platform niet veilig werkt, bestaat op geen enkel platform. Uitgewerkt in [PLATFORMONDERSTEUNING.md](PLATFORMONDERSTEUNING.md) |

### 1.4 Wat er in deze herziening is veranderd

| Categorie | Wijziging |
|---|---|
| **AVG verdiept** | Doorgifteregister met instrumenten en TIA, gezamenlijke verwerkingsverantwoordelijkheid, art. 19-kennisgeving, art. 22 en algoritmeregister, BSN, strafrechtelijke gegevens, toestemmingsbewijs, belangenafweging, verwerkersmeldketen, vertegenwoordiger in de Unie, en de wettelijk voorgeschreven documentatievelden van het datalekregister |
| **Nationaal recht toegevoegd** | Wpg-privacyaudit, Woo-spoor, bestuursrechtelijke klokken en het cautiepunt, verordening (EU) 2024/1689 |
| **Cbw/NIS2 aangevuld** | Significante cyberdreiging, vrijwillige melding, initiële registratie, KMO-scoperesolver met tweejaarsregel en consolidatie, jurisdictiebepaling, CSIRT- en autoriteitenresolver, raamwerkvariant C, risicobeoordeling als artefact |
| **Feitelijke correcties** | Aggregatiedrempel, termijnrekenkunde, ENISA-termijn, DORA-verhouding, normaanduidingen, MijnNCSC als enig kanaal, sectorale regelingen als normenkader |
| **Beveiliging** | Externe verankering verplicht, TOTP- en PIV-pad geschrapt, compartimenten cryptografisch, sleutelrotatie, SLIP-0039, sandboxspecificatie per platform, WebView-hardening, export standaard versleuteld, klembordbeleid |
| **Geschrapt of versoberd** | Eigen redactiepijplijn, on-premise servervariant, twee extra sleutelpaden, vier van de tien crosswalkraamwerken in v1, bitemporaliteit op alles, campagnebeheer, drie driftimporters, uitvraagsimulatie, benchmarks, byte-identieke builds, volledige toegankelijkheidsaudit in week één |
| **Toegevoegd** | Externe tijdsverankering met open bewijsformaat, bestuursrechtelijk spoor, vervalprognose, ketenbewijs, persoonlijk onafhankelijkheidsdossier van de FG |
| **Planning** | Nieuwe fase −1 (inhoud en marktbewijs) vóór de eerste regel productiecode; opslag van 40 procent; halve capaciteit vanaf de eerste betalende klant; stackbeslissing van week 44 naar week 3 |

---

## 2. Doelgroep, gebruikers en gebruiksscenario's

### 2.1 Gebruikers

| Rol | Wat deze persoon doet | Wat de tool voor hem doet |
|---|---|---|
| **Functionaris voor gegevensbescherming** | Toezicht, advies, voorlichting, contactpunt AP, register bewaken | Werkbak met klokken, adviesregister met bestuursreactie, persoonlijk onafhankelijkheidsdossier, dossierbundels |
| **Security officer / CISO** | Zorgplicht, incidenten, leveranciers, advisories, controlset | Incidentcockpit met vijf klokken, controlset met bewijs, leveranciers- en advisoryregister |
| **Bestuurder** | Goedkeuren maatregelenpakket, opleidingsplicht, restrisico aanvaarden | Bestuursrapportage, vervalprognose op datum, opleidingsspoor met certificaatvalidatie |
| **Proceseigenaar / afdelingscontact** | Levert registergegevens, voert maatregelen uit | Beperkte, gecompartimenteerde invoer via de registerreviewcyclus |
| **Toezichthouder / auditor (lezer)** | Verifieert een aangeleverd dossier | Losse, apart ondertekende verifier plus gepubliceerde formaatspecificatie; geen installatie van de hoofdapplicatie nodig |

### 2.2 Sectoren in de startbibliotheken

Gemeente en gemeenschappelijke regeling, zorginstelling, onderwijsinstelling, en het middenbedrijf dat als belangrijke entiteit onder de Cbw valt. Waterschap en drinkwaterbedrijf volgen als aparte startbibliotheek zodra de bevoegde autoriteit en de sectorale regeling in het contentpack zijn geverifieerd.

### 2.3 Het koopmotief

Een FG koopt dit product niet voor een dashboard. Hij koopt het voor vier momenten:

1. De ochtend waarop een incident binnenkomt en er vijf klokken tegelijk gaan lopen.
2. De brief van de toezichthouder met een uitvraag en een reactietermijn.
3. De bestuursvergadering waarin hij moet uitleggen wat er over negentig dagen niet meer aantoonbaar is.
4. Het moment waarop zijn advies niet wordt overgenomen en hij dat over twee jaar nog moet kunnen laten zien.

Elke functie in dit plan is aan één van die vier momenten gekoppeld. Functies die aan geen van vier hangen, staan in §12.

---

## 3. Wettelijk kader, toepassingsbereik en resolvers

### 3.1 De regimes die de tool bedient

| Regime | Kerninhoud voor dit product | Toezicht |
|---|---|---|
| **AVG** | Register, rechten van betrokkenen, datalekken, DPIA, doorgiften, verwerkers, FG-positie | Autoriteit Persoonsgegevens |
| **UAVG** | BSN, strafrechtelijke gegevens, geheimhouding FG, geautomatiseerde besluitvorming bij de overheid | Autoriteit Persoonsgegevens |
| **Cbw** | Zorgplicht, meldplicht, registratieplicht, bestuursverplichtingen, weringsbesluiten, handhaving | Sectorale bevoegde autoriteit |
| **Cbb** | Uitwerking zorgplicht (art. 6-18), significantiedrempels, bestuurdersopleiding, registratie | Idem |
| **Uitvoeringsverordening (EU) 2024/2690** | Raamwerkvariant B voor de entiteitstypen van Cbb art. 4; aggregatie- en significantiecriteria | Idem |
| **Sectorale ministeriële regelingen** | Voorgeschreven normenkader, drempelset en aanvullende verplichtingen — zie §3.5 | Idem |
| **Wpg** | Interne controle en vierjaarlijkse externe privacyaudit bij gemeenten met boa's | Autoriteit Persoonsgegevens |
| **Woo** | Beslistermijnen op informatieverzoeken, onderscheid met het inzageverzoek | Bestuursrechter |
| **Awb** | Zienswijze, bezwaar, beroep, medewerkingsplicht tegenover zwijgrecht, verjaring boetebevoegdheid | Bestuursrechter |
| **Verordening (EU) 2024/1689** | Transparantieplichten en classificatie van algoritmische systemen; koppeling aan het algoritmeregister | Aangewezen markttoezichthouder |
| **Verordening (EU) 2024/2847 (CRA)** | Aan de inkoopkant een toetsingscriterium; aan de leverancierskant een eigen verplichting — zie §10.7 | Markttoezicht |
| **DORA** | Regime-verhouding, geen uitvoering — zie §3.6 | DNB/AFM |

> In dit document wordt Verordening (EU) 2024/1689 aangeduid als **algoritmeverordening**. De verordening is te herkennen aan haar nummer; de aanduiding wordt in het contentpack met vindplaats en consolidatiedatum vastgelegd, inclusief de wijzigingen die bij Verordening (EU) 2026/1744 zijn doorgevoerd. De ingangsdata van de transparantieplichten zijn contentpackwaarden en staan niet in de binary.

### 3.2 Datumbeheer — correctie op de vorige versie

In herziening 1.0 stonden 15-08-2026, 15-08-2028, 11-09-2026 en 11-12-2027 verspreid in lopende tekst, testgevallen en risicotabel. Dat is in strijd met principe P1: één verschoven datum zou een release afdwingen.

**Correctie.** Alle rechtsfeiten worden gemodelleerd als contentpackobjecten:

```
RECHTSFEIT
  id, code, omschrijving, soort (inwerkingtreding|backstop|toepassingsdatum|
  vervaldatum|overgangsmoment), datum, jurisdictie, grondslag_ref,
  bron, consolidatiedatum, geverifieerd_door, geverifieerd_op
```

Verplichtingen, tests en teksten verwijzen naar `RECHTSFEIT.code`, nooit naar een letterlijke datum. In dit document staan datums uitsluitend als *huidige contentpackwaarde*, herkenbaar aan de notatie **[RF-CBW-IWT: 15-08-2026]**. De golden-testsuite draait tegen een testcontentpack met dezelfde codes en afwijkende datums, zodat een verschoven datum een contentpackwijziging is en geen codewijziging.

| Code | Betekenis | Huidige waarde |
|---|---|---|
| `RF-CBW-IWT` | Inwerkingtreding Cbw en Cbb | 15-08-2026 |
| `RF-BEST-BACKSTOP` | Collectieve backstop bestuurdersopleiding | 15-08-2028 |
| `RF-CRA-MELD` | Aanvang meldplicht fabrikanten CRA | 11-09-2026 |
| `RF-CRA-VOL` | Volledige toepassing CRA | 11-12-2027 |
| `RF-BIO2-VERPL` | Voorgeschreven normenkader overheidsinstanties | 15-08-2026 |
| `RF-ALGVO-ART50` | Toepassing transparantieplichten algoritmeverordening | contentpackwaarde, te verifiëren |

### 3.3 Scope- en regime-resolver

De resolver bepaalt per entiteit welke regimes gelden en legt de uitkomst vast als **classificatiebesluit-artefact**: een ondertekend, gedateerd document met invoer, redenering, uitkomst en de naam van degene die het vaststelt. Herbeoordeling is een verplichting met eigen frequentie.

**Toegevoegd in deze herziening: de omvangstoets.** De vorige versie behandelde de grootte-eis als een eenvoudige drempel. Dat is onjuist.

| Element | Regel | Grondslag |
|---|---|---|
| Personeelsbestand en financiële drempels | Middelgroot: minder dan 250 werkzame personen én (jaaromzet ten hoogste 50 miljoen euro óf balanstotaal ten hoogste 43 miljoen euro) | Aanbeveling 2003/361/EG, bijlage art. 2 |
| **Tweejaarsvereiste** | Een overschrijding of onderschrijding leidt pas tot statuswijziging als zij zich in **twee opeenvolgende boekjaren** voordoet | Bijlage art. 4 lid 2 |
| **Partneronderneming** | Deelneming van 25 tot 50 procent: gegevens naar rato optellen | Bijlage art. 3 lid 2 en art. 6 |
| **Verbonden onderneming** | Deelneming boven 50 procent of doorslaggevende zeggenschap: gegevens volledig optellen | Bijlage art. 3 lid 3 |
| **Grootte-onafhankelijke aanwijzing** | Bepaalde entiteiten vallen ongeacht omvang onder het regime | NIS2 art. 2 lid 2 en 3, Cbw-pendant |

De resolver toont de volledige rekenketen: welke ondernemingen zijn meegeteld, met welk percentage, over welke twee boekjaren. Zonder die keten is er geen classificatiebesluit.

**Jurisdictie en hoofdvestiging.** Voor entiteitstypen waarvoor het hoofdvestigingscriterium geldt, bepaalt de resolver in welke lidstaat de entiteit onder toezicht valt, en of een vertegenwoordiger in de Unie is vereist. De uitkomst is een veld op `ENTITEIT` met motivering en herbeoordelingsverplichting. Dezelfde vraag speelt onder de AVG (art. 27) voor niet-EU-verwerkingsverantwoordelijken en -verwerkers; beide worden apart vastgelegd omdat de criteria verschillen.

### 3.4 Autoriteiten- en CSIRT-resolver — correctie op de vorige versie

Herziening 1.0 ging uit van één meldkanaal. Dat is feitelijk onjuist en operationeel gevaarlijk: een zorginstelling die bij het nationale kanaal meldt, meldt bij het verkeerde CSIRT.

**Correctie.** Er staat nergens in de code een naam van een toezichthouder, een CSIRT of een meldportaal.

```
AUTORITEIT
  id, soort (bevoegde_autoriteit|csirt|gegevensbeschermingsautoriteit|
  markttoezichthouder), naam, sector[], entiteitstypen[], meldkanaal,
  kanaal_ref, formuliersjabloon_id, geldig_van, geldig_tot,
  bron, geverifieerd_op
```

De resolver bepaalt op grond van sector, entiteitstype en peildatum welke autoriteit en welk CSIRT van toepassing zijn en toont dat expliciet in de meldcockpit:

> *"Uw CSIRT is X; uw bevoegde autoriteit is Y; beide moeten worden geïnformeerd."*

Wisselingen in aanwijzing — bijvoorbeeld een sectoraal CSIRT dat een taak overneemt van een instantie die de dienstverlening voorlopig verzorgt — zijn contentpackwijzigingen met een ingangsdatum, geen releases. Bij een wijziging binnen een lopend incident toont de tool beide adressen met de datumgrens erbij en dwingt een expliciete keuze met motivering af.

### 3.5 Sectorale ministeriële regelingen — correctie op de vorige versie

Herziening 1.0 modelleerde sectorale regelingen uitsluitend als drempelset op het significantiespoor. Dat is een architectuurfout: een sectorale regeling kan een normenkader dwingend voorschrijven, en dat raakt het raamwerkvariantmodel.

**Correctie.** Een sectorale regeling kan drie dingen tegelijk doen en wordt daarom als drie afzonderlijke koppelingen gemodelleerd:

| Koppeling | Effect |
|---|---|
| **Voorgeschreven normenkader** | Nieuwe **raamwerkvariant C**: het normenkader is niet gekozen maar opgelegd. De controlset wordt uit het voorgeschreven kader afgeleid, afwijken vereist een expliciete grondslag in de regeling zelf |
| **Drempelset** | Sectorale significantiedrempels op het NIS2-spoor, versiebeheerd, met de open norm van Cbw art. 25 lid 2 altijd als vangnet erboven |
| **Aanvullende verplichtingen** | Eigen `VERPLICHTING`-sjablonen met eigen frequenties en bewijsvereisten |

Elke regeling krijgt een eigen `NORMBEPALING`-set met versie en consolidatiedatum. De huidige contentpackinhoud kent voor overheidsinstanties een dwingend voorgeschreven kader per **[RF-BIO2-VERPL]**; de exacte versieaanduiding is contentpackinhoud met verificatiestempel.

### 3.6 DORA — correctie op de vorige versie

Herziening 1.0 stelde dat de Cbw-verplichting "vervalt" zodra DORA geldt. Dat is als algemene regel onjuist.

**Correctie.** De sectorspecifieke uitsluiting werkt per onderwerp en alleen voor zover de sectorale regels ten minste gelijkwaardig zijn. De identificatie- en registratiesystematiek blijft van toepassing. De regime-resolver werkt daarom **per verplichtingssoort**, niet per entiteit:

| Verplichtingssoort | Uitkomst bij DORA-entiteit |
|---|---|
| Risicobeheermaatregelen | Beoordeling van gelijkwaardigheid; uitkomst vastgelegd met motivering |
| Incidentmelding | Idem; bij gelijkwaardigheid vervalt de Cbw-meldketen, met vastlegging van de grondslag |
| Toezicht op die onderwerpen | Volgt de uitkomst van de voorgaande twee |
| **Identificatie en registratie** | **Blijft van toepassing** — de entiteit blijft in het register en de registratieklokken blijven lopen |

Voor gemengde concerns toont de tool beide regimes naast elkaar, met de stapelingswaarschuwing voor de entiteit die tevens kritieke ICT-derde partij is. De tool ondersteunt de DORA-verplichtingen zelf niet (§12.4).

### 3.7 Kennisnamebegrip — nuancering

Herziening 1.0 hanteerde één formulering voor beide regimes. Dat is voor de AVG te stellig.

| Regime | Aanvang van de klok | Vastlegging |
|---|---|---|
| **Cbw/NIS2** | Kennisname van het significante incident. Niet het optreden, niet de bevestigde diagnose | `tijdstip_kennisname` + onderbouwing |
| **AVG** | Kennisname veronderstelt een redelijke mate van zekerheid dat een inbreuk heeft plaatsgevonden. Een korte eerste verificatie is toegestaan, mits die zelf wordt gedocumenteerd en niet als uitstelmechanisme wordt gebruikt | `tijdstip_signaal`, `verificatieperiode_van/tot`, `verificatie_onderbouwing`, `tijdstip_kennisname` |

De interface toont beide ankers naast elkaar en waarschuwt zodra de verificatieperiode langer duurt dan de in het contentpack vastgelegde signaalwaarde. Verlenging van de verificatieperiode vereist een motivering met naam en tijdstip.

---

## 4. Functioneel overzicht: de modules

| # | Module | Kern | Eerste fase |
|---|---|---|---|
| 1 | Toegang, rollen en compartimenten | Rol- en rechtenmodel, cryptografisch afgedwongen compartimenten, onuitschakelbaar auditlogboek | 0 / 1 |
| 2 | FG-cockpit en werkbak | Eén werkvoorraad over alle regimes heen, gesorteerd op harde termijn en onherstelbaarheid | 1 |
| 3 | Scope-, regime- en autoriteitenresolver | Classificatiebesluit-artefact, omvangstoets, jurisdictie, CSIRT- en autoriteitsbepaling | 1 |
| 4 | Termijnenmotor | Getypeerde termijnen, ankers, pauzes, escalatie, herberekening, rekenregelverantwoording | 1 |
| 5 | Bewijskluis en verankering | Chain of custody, geldigheidsvensters, houdbaarheidsmotor, ketenankers | 0 / 1 |
| 6 | Normen- en kennisbank | Contentpackformaat, normbepalingen, crosswalkqueries, wetteksten met vindplaats | −1 / 1 |
| 7 | Verwerkingsregister | Beide schema's (art. 30 lid 1 en lid 2), hygiënecontroles, tijdmachine, doorgiften, bijzondere categorieën | 1 |
| 8 | Incident- en meldcockpit | Vijf klokken, twee sporen, significantie-engine, meldformulieren, verwerkersmeldketen | 1 |
| 9 | Advies- en onafhankelijkheidsregister | Comply-or-explain, escalatiestappen, persoonlijk FG-dossier | 1 |
| 10 | Dossier, rapportage en verstrekking | Bundel met manifest, kwaliteitspoort, verstrekkingslogboek, sjabloonbeheer | 1 / 4 |
| 11 | Aantoonbaarheidsscore en **vervalprognose** | Driefactorscore, gap-analyse, correctieplicht, prognose op 30/90/365 dagen | 3 |
| 12 | Zorgplichtcontrolset | Varianten A, B en C; motiveringsplicht bij niet-toepassing | 3 |
| 13 | Betrokkenenverzoeken, Woo en art. 19 | Termijnlogica in maanden, zoekorkestratie, kennisgeving aan ontvangers, redactieregie | 2 |
| 14 | DPIA, LIA, TIA en art. 36 | Pre-scan, belangenafweging, doorgiftebeoordeling, raadplegingsklok met opschorting | 2 |
| 15 | Leveranciers- en ketenregister | Art. 28-checklist met vindplaats, Cbb-leverancierstoetsing, advisory-inbox, weringsbesluiten | 3 |
| 15b | **Ketenbewijs** | Ondertekende, extern verankerde bewijspakketten tussen organisaties | 4 |
| 16 | Toezichtdossier | Correspondentie, bevindingen, maatregelen, handhavingsladder | 4 |
| 16b | **Bestuursrechtelijk spoor** | Procesfasen, zienswijze-, bezwaar- en beroepsklokken, cautiesignalering, verstrekkingslogboek | 4 |
| 17 | Opleiding en bewustwording (versoberd) | Bestuurdersopleidingsspoor met certificaatvalidatie; deelnamebewijs op groepsniveau | 4 |
| 18a | Intakepoortje (voorwaardelijk) | Aparte binary met publieke-sleutel-drop | 5, onder criterium |
| 18b | Multi-entiteit en uitcheckmodel | Meerdere entiteiten, uitcheck, dossieroverdracht | 5 |

### 4.1 De vijf onderscheidende functies

Deze vijf bepalen of het product zich onderscheidt. Zij zijn in deze herziening toegevoegd of aangescherpt.

| # | Functie | Waarom onderscheidend | Sectie |
|---|---|---|---|
| O1 | **Onafhankelijk verifieerbare tijdsverankering met open bewijsformaat** | Beantwoordt de eerste vraag van elke toezichthouder: hoe weet ik dat dit er gisteren ook al zo stond | §6.4 |
| O2 | **Bestuursrechtelijk spoor** | Op het moment van de hoogste tijdsdruk heeft de FG nu geen enkel hulpmiddel | §7.7, module 16b |
| O3 | **Vervalprognose** | Vertaalt informatiebeveiliging naar de enige eenheid die een bestuur begrijpt: een datum | §7.8, module 11 |
| O4 | **Ketenbewijs** | De enige functie met een netwerkeffect; elke klant zet tien leveranciers onder druk | Module 15b |
| O5 | **Persoonlijk onafhankelijkheidsdossier van de FG** | AVG art. 38 lid 3 is waardeloos als het bewijs uitsluitend berust bij degene tegen wie de bescherming is gericht | §7.4 |

### 4.2 De werkbak (module 2)

De werkbak is één lijst, geen dashboard. Sortering is vast en niet door de gebruiker om te draaien:

1. **Onherstelbaar en vandaag** — klokken in uren die vandaag aflopen.
2. **Onherstelbaar deze week** — meldtermijnen, bezwaartermijnen, zienswijzetermijnen.
3. **Achterstallig sinds** — verplichtingen waarvan het anker vóór de inwerkingtredingsdatum ligt, gemarkeerd als *verplicht sinds*.
4. **Vervalprognose 30 dagen** — eisen die binnen een maand onbewijsbaar worden.
5. **Overig, op termijn.**

Bij elke regel staat de grondslag als citeerbare tekst, de toegepaste rekenregel en de naam van de eigenaar.

---

## 5. Systeemarchitectuur en techniekkeuzen

### 5.1 Proces- en cratesindeling

```
┌─────────────────────────────────────────────────────────────┐
│  Schil (desktop, webview)  — rendert, bevat geen sleutels    │
└──────────────┬──────────────────────────────────────────────┘
               │ smalle, getypeerde brug, allowlist per commando
┌──────────────▼──────────────────────────────────────────────┐
│  Kernproces                                                  │
│   dpofg-domain   pure domeinlogica, geen I/O                 │
│   dpofg-terms    termijnenmotor (getypeerde termijnen)       │
│   dpofg-crypto   sleutelafleiding, envelope, shares          │
│   dpofg-store    opslag, migraties, blob store               │
│   dpofg-audit    ketenlogboek, ankers, verificatie           │
│   dpofg-content  contentpacks: laden, handtekening, versie   │
│   dpofg-report   documentopbouw, manifest, ondertekening     │
└──────┬───────────────────────────────┬──────────────────────┘
       │ pipe, geen gedeeld geheugen   │
┌──────▼─────────────┐   ┌─────────────▼──────────────────────┐
│ Parserproces        │   │ Beeld-/documentproces              │
│ CSV, XLSX, EML,     │   │ rendering, vergelijking            │
│ MSG, PDF, JSON      │   │                                    │
│ sandbox, geen net,  │   │ sandbox, geen net,                 │
│ geen schrijfrechten │   │ geen schrijfrechten                │
└─────────────────────┘   └────────────────────────────────────┘
       │
┌──────▼──────────────────────────────────────────────────────┐
│  dpofg-verify — losse binary, alleen lezen, apart onder-     │
│  tekend, draait bij de toezichthouder zonder de hoofdapp     │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 Techniekkeuzen en de correcties daarop

| Onderdeel | Herziening 1.0 | Herziening 2.0 | Reden |
|---|---|---|---|
| Taal kern | Rust | Rust, met **beslispunt aan het einde van week 3** na een spike op CTAP2 `hmac-secret`, versleutelde opslag en de hashketen op drie platforms | Het oude beslispunt lag aan het einde van fase 0; overstappen betekende dan het moeilijkste deel weggooien |
| Opslag | SQLCipher | **Gewone SQLite met envelopeversleuteling per compartiment**, tenzij de spike aantoont dat versleuteling op databaseniveau daar bovenop iets toevoegt dat een tweede sleutelhiërarchie rechtvaardigt | Twee overlappende versleutelingslagen betekenen twee sleutelmodellen, twee migratiepaden en een C-afhankelijkheid in het hart. De keuze is architectonisch, niet juridisch |
| Schil | Webview | Webview met **harde ondergrens op de engineversie**, uitgeschakelde reputatie-, spellings- en crashdiensten, en een startcontrole die weigert te starten onder die ondergrens | Op één platform loopt de engine structureel achter op beveiligingsupdates; dat is een kwetsbaarheidsvraag, geen compatibiliteitsvraag |
| Documentgeneratie | Bibliotheek met byte-identieke uitvoer als criterium | Bibliotheek achter een trait met terugvalimplementatie; **byte-gelijkheid vervalt als criterium**. Het criterium is: is de handtekening onder de uitvoer verifieerbaar met standaardgereedschap dat een toezichthouder al heeft | Het echte probleem is de verifieerbaarheid van de handtekening, niet de determinismegraad van de opmaak |
| Ondertekening uitvoer | Losse Ed25519-handtekening | **Handtekening op het document conform de Europese eIDAS-systematiek**, met de losse Ed25519-handtekening als aanvulling en niet als vervanging | Een losse handtekening kan een toezichthouder niet controleren zonder leverancierstools |
| Zoekindex | Eén index | **Eén index per compartiment**, versleuteld met de compartimentsleutel | Een gedeelde index over gecompartimenteerde inhoud lekt tokens |

### 5.3 Geheugenveiligheid — eerlijke afbakening

De keuze voor een geheugenveilige taal in de kern is echt, maar dekt niet de componenten waar historisch de meeste fouten zitten. Elke afhankelijkheid met een niet-geheugenveilige implementatie draait daarom **buiten het kernproces**, in een sandbox zonder netwerkcapability en zonder schrijfrechten buiten een tijdelijke map, met een harde geheugen- en tijdlimiet.

| Component | Implementatietaal | Plaatsing |
|---|---|---|
| Documentparsers (PDF, XLSX, EML, MSG) | Gemengd | Parserproces, gesandboxd |
| Beeldcodecs en vergelijking | Gemengd | Beeldproces, gesandboxd |
| Databasemotor | C | Kernproces — daarom is de omvang van die afhankelijkheid een expliciet beslispunt (§5.2) |
| Crypto | Geheugenveilig, gepind, `cargo-vet`-geaudit | Kernproces |

De marketingtekst mag niet zeggen "geschreven in een geheugenveilige taal, dus veilig". Zij zegt: "de kern is geheugenveilig; alles wat dat niet is, draait in een sandbox zonder netwerk en zonder schrijfrechten, en dat is in het dreigingsmodel uitgewerkt."

---

## 6. Beveiligings- en bewijsarchitectuur

### 6.1 Dreigingsmodel — wie is de tegenstander

| # | Tegenstander | Vermogen | Belangrijkste maatregel |
|---|---|---|---|
| D1 | Dief van de laptop | Bezit het bestand, niet de sleutel | Sleutelafleiding met harde ondergrens, hardwaretoken verplicht |
| D2 | Malware op de host | Draait als de gebruiker | Sandboxing, geen persistent ontsleutelde staat, automatische vergrendeling, klembordbeleid |
| D3 | **Insider met volledige toegang** | Bezit wachtwoordzin, token én bestand; beheert de systeemklok | **Externe verankering** (§6.4) — dit is de tegenstander die de vorige versie niet adresseerde |
| D4 | De organisatie als tegenpartij van de FG | Beheert de infrastructuur en de kluis | Persoonlijk FG-dossier met eigen sleutel (§7.4) |
| D5 | Onbedoelde verspreiding | Synchronisatiedienst, zoekindex, back-up, antivirus-inzending | Detectie en blokkering (§6.7) |
| D6 | Redactiefout | De tool wordt zelf de bron van een datalek | Redactieregie met verplichte terugleescontrole (§9, fase 2) |

### 6.2 Sleutelhiërarchie en ontgrendelpaden — versoberd

Herziening 1.0 kende vier ontgrendelpaden. Dat is teruggebracht tot twee. Het TOTP-noodpad is volledig vervallen: een code van zes cijfers kan geen betekenisvolle entropie aan een sleutelafleiding bijdragen en verificatie vereist opslag van het gedeelde geheim. Als dat pad een kluis kan openen, is de kluis te openen met een geheim van lage entropie — en dat staat haaks op de belofte dat er geen achterdeur is. Het terugvalpad op een tweede kaarttechnologie is eveneens vervallen; vier paden betekent vier keer testen en het zwakste pad bepaalt het niveau.

```
Wachtwoordzin ──▶ sleutelafleiding (parameters als geauthenticeerde
                  bijkomende gegevens gebonden aan de afleiding,
                  harde ondergrens in code)
                        │
Hardwaretoken (CTAP2 hmac-secret) ──▶ tokengeheim
                        │
                        ▼
                  Hoofdsleutel (KEK)
                        │
        ┌───────────────┼──────────────────────────────┐
        ▼               ▼                              ▼
  Dataversleutelings-  Compartimentsleutel        Blob-sleutels
  sleutel (DEK)        per compartiment           per bewijsstuk
        │                    │
        │                    └─▶ eigen versleutelde zoekindex
        ▼
  Herstelpad: Shamir-shares over de KEK, SLIP-0039-compatibel,
  met per-share- en totaalauthenticatie
```

| Maatregel | Invulling |
|---|---|
| Parameters van de sleutelafleiding | Harde ondergrens in code; de opgeslagen parameters zijn gebonden aan de afleiding als geauthenticeerde bijkomende gegevens, zodat verlaging in de header niet leidt tot een zwakkere afleiding maar tot een verificatiefout |
| **Twee authenticators verplicht** | De kluis accepteert geen productiegegevens voordat ten minste twee tokens zijn geregistreerd. Het credential-identificatiegegeven staat versleuteld, niet in de leesbare header |
| Shamir | **SLIP-0039-compatibel**, met integriteitscontrole per share en over het geheel. Klassieke deling over GF(256) is kneedbaar: gemanipuleerde shares leveren stilzwijgend een verkeerd geheim op |
| **Shares hebben een levenscyclus** | `SHARE (houder, uitgiftedatum, laatste_bevestiging, vervaldatum, ingetrokken_op)` met een halfjaarlijkse bevestigingsverplichting in de werkbak en verplichte herverdeling bij personeelswissel |
| **Sleutelrotatie** | Verplicht bij elke wisseling van een sleutelhouder en ten minste jaarlijks. Herversleuteling op de achtergrond, met een rotatiebewijsstuk in het auditdossier. Zonder rotatie is offboarding onmogelijk: een vertrokken FG heeft de dataversleutelingssleutel gezien |

### 6.3 Compartimenten — cryptografisch, niet als applicatieregel

Herziening 1.0 suggereerde eigen sleutels maar plaatste alle gegevens in één op paginaniveau versleutelde database. Wie de kluis kon openen, kon met een standaard databaseshell alles lezen; de onzichtbaarheidsregel was daarmee een applicatieregel en geen garantie.

**Correctie.**

| Aspect | Invulling |
|---|---|
| Gegevens | Elk record met een `compartiment_id` is op veldniveau versleuteld met de compartimentsleutel, bovenop de kluisversleuteling |
| Zoekindex | Elk compartiment heeft een eigen index, versleuteld met de compartimentsleutel. De hoofdindex bevat geen tokens uit gecompartimenteerde inhoud |
| Journaal en tijdelijke bestanden | Journaalmodus en tijdelijke opslag staan in het geheugen of in een versleutelde tijdelijke map binnen de kluislocatie; nooit in een systeemtijdelijke map |
| Tellingen en aggregaten | Een gebruiker zonder compartimenttoegang ziet geen afwijkend totaal. Aggregaten worden per toegangsbereik berekend, nooit achteraf gefilterd |
| Bewijs | De testmatrix rol × objecttype × compartiment verifieert dat een object in geen enkel codepad verschijnt: query, index, export, rapport, telling, foutmelding |

### 6.4 Bewijskracht en de grenzen daarvan

> **Wat de kluis wel en niet bewijst.** De hashketen bewijst dat de inhoud van de kluis na vastlegging niet ongemerkt is gewijzigd door iemand zónder de sleutels. Zij bewijst uit zichzelf niet dat een record is vastgelegd op het moment dat het claimt, want de houder van de sleutels beheert ook de systeemklok en het bestand. Voor bewijskracht tegenover een toezichthouder of een rechter is daarom externe verankering nodig.

Dit is de belangrijkste correctie in deze herziening. Zonder haar is de verifier van de leverancier een verifier die het formaat van de leverancier controleert, en dat is geen onafhankelijke verificatie.

**1. Verplicht dagelijks anker.** De kluis produceert per dag een `KETENANKER (dagdigest, volgnummerbereik, tijdstip, apparaat_id, epoch)`. Het anker verlaat de kluis als los, klein bestand zonder persoonsgegevens.

**2. Drie ankerkanalen, ten minste één verplicht ingericht vóór productiegebruik.**

| Kanaal | Invulling | Bewijskracht |
|---|---|---|
| **(a) Gekwalificeerde tijdstempel** | Tijdstempeltoken conform RFC 3161 van een gekwalificeerde tijdstempeldienst, offline geïmporteerd via een handmatige verzoek-en-antwoordstroom (de tool opent zelf geen verbinding) | **Standaard.** Het enige kanaal dat bewijskracht levert tegenover een derde |
| (b) Postbus buiten beheer van de FG | Verzending van de dagdigest naar bijvoorbeeld de bestuurssecretaris of de externe accountant | Getuigenbewijs; zwakker, maar toetsbaar |
| (c) Afdruk met paraaf | Periodieke afdruk van de dagdigest, geparafeerd en gearchiveerd | Zwakste; alleen als noodvoorziening |

**3. Klokdiscipline.** Elk klokgevoelig record legt naast de wandkloktijd een **monotone teller**, de tijdzone-offset en de laatst bekende afwijking ten opzichte van het laatste externe anker vast. Een terugsprong van de systeemklok is een auditgebeurtenis die niet kan worden onderdrukt en die in elk daarna geproduceerd dossier zichtbaar is.

**4. Terminologie.** De term "WORM" wordt in alle uitvoer, documentatie en veldnamen vervangen door **append-only met ketenverificatie en externe verankering**. Op een lokaal bestand is "WORM" feitelijk onjuist, en een onjuist woord in een document dat aan een toezichthouder wordt getoond kost geloofwaardigheid die niet terugkomt.

**5. Ketensplitsing bij uitchecken.** Het uitcheckmodel vertakt de keten. Daarom: één keten per apparaat, met een monotone epoch-teller en **kruisondertekening bij inname**. Het samenvoegingsrecord bevat de eindankers van beide ketens en is zelf verankerd. `dpofg-verify` toont de vertakking expliciet in plaats van haar glad te strijken.

**6. Gepubliceerde specificatie.** Het bestands- en bundelformaat, de hashketen, de manifestopbouw, de ankerstructuur en de handtekeningen worden gespecificeerd in een openbaar, versienummerd document met testvectoren, zodat een derde een onafhankelijke verifier kan bouwen.

### 6.5 Onveilige standaarden die expliciet zijn omgekeerd

| Standaardinstelling | Waarde |
|---|---|
| Noodontgrendelpad met eenmalige code | **Vervalt volledig.** Twee ontgrendelpaden: wachtwoordzin plus geregistreerde authenticator, en Shamir-herstel |
| Tweede authenticator | Verplicht geregistreerd vóór de kluis productiegegevens accepteert |
| Vergrendeling | Automatisch bij schermvergrendeling, slaapstand, gebruikerswissel en na tien minuten inactiviteit; sleutelmateriaal verlaat het geheugen. Geen optie om de wachtwoordzin te onthouden |
| Klembord | Wissen na dertig seconden; een gesynchroniseerd klembord wordt gedetecteerd en geblokkeerd met blokkerende waarschuwing |
| Export | **Standaard versleuteld** naar een ontvangersleutel. Onversleutelde export vereist een gelogde overrule met motivering en verschijnt in het auditdossier en in het verstrekkingslogboek |
| Compartimenten | Envelopeversleuteling per compartiment bovenop de kluissleutel; eigen versleutelde index. De databasesleutel alleen geeft geen toegang tot compartimentsinhoud |
| Parser-, beeld- en documentprocessen | Eigen sandbox per platform, zonder netwerkcapability en zonder schrijfrechten buiten een tijdelijke map, met harde geheugen- en tijdlimiet |
| Webview | Ontwikkelgereedschap en foutopsporing op afstand uit in release; crashrapportage, spellingcontrole en reputatiediensten uitgeschakeld via het installatiebeleid; startcontrole weigert te starten onder de gepubliceerde ondergrens en meldt dat expliciet |
| Ingebouwde inhoud in de schil | Geen enkele constructie die ongefilterde inhoud als opmaak invoegt; geïmporteerde tekst (e-mailbodies, bestandsnamen, leveranciersnamen) wordt altijd als tekst weergegeven; strikte inhoudsbeleidsregels; de brug kent een allowlist per commando met getypeerde argumenten |
| Redactie | Uitvoer wordt na redactie opnieuw opgebouwd en automatisch teruggelezen: tekstextractie, metadata- en annotatiescan, beeldvergelijking op de geredigeerde gebieden. Vindt de controle een geredigeerde term terug, dan is verstrekking geblokkeerd |
| Sleutelrotatie | Verplicht bij elke wisseling van een sleutelhouder en ten minste jaarlijks, met rotatiebewijsstuk |
| Herstelshares | SLIP-0039-compatibel met integriteitscontrole; levenscyclus met halfjaarlijkse bevestigingsverplichting |
| Auditlogboek | Kan door geen enkele rol worden uitgeschakeld, ook niet door de beheerder. Het uitschakelbaar maken is geen configuratieoptie en komt er niet |

### 6.6 Sandboxspecificatie per platform

"Gescheiden proces" is geen sandbox. Per platform is de invulling vastgelegd en getest:

| Platform | Invulling |
|---|---|
| Linux | Namespaces (mount, pid, net, user) plus een restrictief systeemaanroepfilter; geen netwerknamespace met route; alleen-lezen wortelbestandssysteem behalve één tijdelijke map |
| macOS | App Sandbox met expliciet geweigerde netwerkrechten; toegang uitsluitend tot de aangeboden bestandsdescriptor |
| Windows | Laag-integriteitscontainer met een taakobject dat geheugen, processen en tijd begrenst; geen netwerkcapability |

Een integratietest verifieert per platform dat een netwerkverbinding vanuit het parserproces daadwerkelijk faalt en dat een crash het kernproces niet meeneemt.

### 6.7 Het echte back-up- en lekoppervlak

Herziening 1.0 adresseerde alleen netwerkpaden. Het oppervlak is breder:

| Kanaal | Maatregel |
|---|---|
| Synchronisatiemappen en automatische mapverplaatsing | Detectie bij het openen én bij elke start; blokkerende waarschuwing met de gevonden padoorsprong |
| Zoekindex van het besturingssysteem | Kluislocatie wordt bij ingebruikname uitgesloten; de zelftest controleert dit en meldt bij afwijking |
| Systeemback-up en momentopnamen | De hardeningshandleiding schrijft uitsluiting voor; de zelftest detecteert bekende momentopnamemechanismen op het volume |
| Automatische monsterinzending door antivirus | Uitsluiting van het kluispad wordt in de hardeningshandleiding voorgeschreven en bij ingebruikname bevestigd, met een uitleg waarom: het kluisbestand kan anders als verdacht bestand naar een leverancier worden geüpload |
| Wisselbestand, sluimerbestand en crashdumps | Gevoelig materiaal wordt vastgezet in het geheugen waar het platform dat toelaat, met de expliciete vermelding dat dit op één platform quotabeperkt is. De hardeningshandleiding schrijft versleutelde schijf en uitgeschakelde sluimerstand voor; de zelftest controleert de schijfversleuteling en meldt de sluimerstatus |
| Roaming profielen | Detectie en blokkerende waarschuwing |

### 6.8 Beveiliging van de leverancier zelf

Zie §10.7. Kort: een product dat klanten helpt aan de Cbw te voldoen terwijl de maker zijn eigen productverplichtingen niet naleeft, is bij de eerste inspectie onverkoopbaar. Vóór de eerste release bestaan er een gepubliceerd kwetsbaarhedenbeleid met contactpunt, een vastgelegde ondersteuningsperiode in de licentievoorwaarden, een kwetsbaarhedenafhandelingsproces met termijnen, en een ingerichte meldprocedure richting het CSIRT en de Europese instantie vóór **[RF-CRA-MELD]**.

---

## 7. Datamodel

### 7.1 Modelleerprincipes

| Principe | Invulling |
|---|---|
| **Bitemporaliteit beperkt** | Alleen `VERWERKING`, `CONTROL`, `BEWIJSSTUK`, `BELEIDSDOCUMENT` en `MAPPING` zijn bitemporeel. Elders volstaat een gewoon wijzigingslogboek met wie, wanneer en waarom. Bitemporaliteit op alles verdubbelt de complexiteit van elke query en elke migratie zonder dat de tijdmachine er beter van wordt |
| **Append-only met keten** | Auditrelevante velden worden nooit overschreven maar gecorrigeerd via een correctierecord |
| **Geen datums in het model** | Alle wettelijke datums verwijzen naar `RECHTSFEIT.code` (§3.2) |
| **Compartiment is een eigenschap van het object** | Niet van de weergave |
| **Elke motiveringsplicht is een verplicht veld** | Niet een tekstvak dat leeg mag blijven |

### 7.2 Kern

**ENTITEIT**
`id, naam, kvk_nummer, rechtsvorm, sectoren[], entiteitstype, classificatie (essentieel|belangrijk|niet_in_scope), classificatiebesluit_id, omvangstoets_id, jurisdictie_lidstaat, hoofdvestiging_motivering, vertegenwoordiger_unie_id, is_verwerkingsverantwoordelijke, is_verwerker, fg_plicht_oordeel, fg_aanmelding_ref, regimes[]`

**OMVANGSTOETS**
`id, entiteit_id, boekjaar_1, boekjaar_2, werkzame_personen, jaaromzet, balanstotaal, partnerondernemingen_json, verbonden_ondernemingen_json, geconsolideerde_waarden, uitkomst, tweejaarsregel_toegepast, vastgesteld_door, vastgesteld_op`

**VESTIGING** `id, entiteit_id, adres, land, is_hoofdvestiging`

**SYSTEEM**
`id, entiteit_id, naam, soort, eigenaar_id, leverancier_id, ip_bereiken[], domeinnamen[], bevat_persoonsgegevens, kritikaliteit, compartiment_id`

**VERPLICHTING** — het centrale object van de termijnenmotor
`id, sjabloon_id, soort (wettelijk_vast|zelf_vastgesteld|contractueel|sectoraal), grondslag_ref, anker_type, anker_object_id, anker_tijdstip, duur_waarde, duur_eenheid (uur|dag|week|maand|jaar), rekenregel_id, deadline_utc, deadline_lokaal, toegepaste_regel_toelichting, verlengbaar, verlenging_max, opschortbaar, status, eigenaar_id, escalatiepad_id, frequentie_vastgesteld_door, frequentie_vastgesteld_op, herberekend_van, compartiment_id`

**BEWIJSSTUK**
`id, titel, soort, bestandshash, bestandsgrootte, herkomst (aangeleverd|zelf_opgesteld|extern_geverifieerd), aanleverketen_json, geldig_van, geldig_tot, bewijskracht (zelfgerapporteerd|geverifieerd), compartiment_id, verankerd_in_anker_id`

**KOPPELING** `id, van_type, van_id, naar_type, naar_id, rol, motivering`

### 7.3 AVG-kant — sterk uitgebreid

**VERWERKING** — beide schema's van art. 30
`id, naam, rol (verantwoordelijke|verwerker|gezamenlijk), doeleinden[], grondslag (art6_a..f), grondslag_motivering, categorieen_betrokkenen[], categorieen_gegevens[], bijzondere_gegevens (ja|nee), uitzondering_art9_10, strafrechtelijke_gegevens (ja|nee), uitzondering_uavg_31_33, bsn_gebruik (ja|nee), ontvangers[], bewaartermijn, bewaartermijn_grondslag, archiefselectielijst_ref, beveiligingsmaatregelen_ref, systemen[], verwerkers[], compartiment_id`

**DOORGIFTE** *(nieuw)*
`id, verwerking_id, ontvanger, ontvangerland, rol_ontvanger (verwerker|verantwoordelijke|gezamenlijk), instrument (adequaatheidsbesluit|modelbepalingen|bcr|gedragscode|certificering|art49_uitzondering|geen), instrument_ref, instrument_versie, geldig_van, geldig_tot, herbeoordelingsdatum, tia_id, aanvullende_maatregelen[], art49_grond, art49_vastlegging_in_register, informatieplicht_betrokkene_uitgevoerd_op`
Relaties: 1—0..1 `TIA (datum, uitvoerder, rechtsontwikkelingen_geraadpleegd_op, uitkomst, restrisico, besluit_door)`; 1—N `VERPLICHTING` (herbeoordeling bij wijziging van het adequaatheidsbesluit of van het recht van het ontvangstland).

**ADEQUAATHEIDSBESLUIT** *(contentpackobject)*
`id, land_of_gebied, besluit_ref, vastgesteld_op, geldig_tot, status (geldig|onder_toetsing|ingetrokken|vernieuwd), laatste_wijziging, wijzigingstoelichting, bron, geverifieerd_op`
Dit is de meest volatiele juridische inhoud die er is: adequaatheidsbesluiten worden vernieuwd, betwist, opgeschort en ingetrokken, en het toezichtsstelsel in het ontvangstland kan wijzigen zonder dat het besluit zelf wijzigt. Een statuswijziging in dit object zet automatisch alle betrokken `DOORGIFTE`-records op *herbeoordelen* en genereert per record een verplichting.

**GEZAMENLIJKE_VERANTWOORDELIJKHEID** *(nieuw)*
`id, verwerking_id, partijen[], regeling_bewijsstuk_id, verdeling_verplichtingen_json, contactpunt, wezenlijke_inhoud_publicatie_ref, vastgesteld_op, herbeoordelingsdatum`

**GEAUTOMATISEERDE_BESLUITVORMING** *(nieuw)*
`id, verwerking_id, is_uitsluitend_geautomatiseerd, rechtsgevolg_of_aanmerkelijke_treffing, grondslag_art22_lid2 (a|b|c), onderliggende_logica_omschrijving, belang_en_gevolgen, menselijke_tussenkomst_procedure, betwistingsprocedure, algoritmeregister_ref, verordening_2024_1689_classificatie (verboden|hoogrisico|transparantieplicht|geen), transparantiemaatregelen, overheidsuitzondering_uavg_ref`

**BSN_GEBRUIK** *(nieuw)*
`id, verwerking_id, wettelijke_grondslag_ref, doel, toets_uitgevoerd_door, toets_datum`

**TOESTEMMING** *(nieuw)*
`id, verwerking_id, doelgroep, wijze_van_verkrijgen, tekst_versie, bewijsvorm, bewijsstuk_id, intrekkingsprocedure_ref, aantal_intrekkingen_periode, minderjarigen (ja|nee), leeftijdsverificatie_methode`
De bewijslast voor toestemming ligt expliciet bij de verwerkingsverantwoordelijke; zonder `bewijsvorm` en `bewijsstuk_id` is de verwerking niet volledig.

**BELANGENAFWEGING (LIA)** *(nieuw)*
`id, verwerking_id, gerechtvaardigd_belang, noodzakelijkheidstoets, afweging_belangen_betrokkene, redelijke_verwachtingen, waarborgen, uitkomst, uitgevoerd_door, datum, herbeoordelingsdatum`
Verplicht bij elke verwerking met grondslag art. 6 lid 1 onder f. Zonder LIA blijft de verwerking *onvolledig*.

**BETROKKENENVERZOEK**
`id, soort (inzage|rectificatie|wissing|beperking|overdraagbaarheid|bezwaar|art22), ontvangen_op, kanaal, identiteit_geverifieerd_op, opschorting_van, opschorting_tot, opschorting_lezing (gekozen_lezing, motivering), verlenging_medegedeeld_op, verlenging_motivering, afgehandeld_op, uitkomst (voldaan|deels|geweigerd), weigeringsgrond, bericht_art12_lid4_verzonden_op, redactieprofiel_id, verstrekte_bundel_id`

**ONTVANGERKENNISGEVING** *(nieuw — AVG art. 19)*
`id, betrokkenenverzoek_id, ontvanger_id, kennisgeving_verzonden_op, wijze, onmogelijk_of_onevenredig (ja|nee), motivering, medegedeeld_aan_betrokkene_op`
Bij elke gehonoreerde rectificatie, wissing of beperking ontstaat per ontvanger één verplichting. De betrokkene heeft daarnaast recht op mededeling *welke* ontvangers zijn geïnformeerd; dat is een aparte verplichting die pas vervalt na een expliciet verzoek-of-niet.

**WOO_VERZOEK** *(nieuw)*
`id, ontvangen_op, onderwerp, beslistermijn_verplichting_id, verdaging_medegedeeld_op, zienswijze_derden_verplichting_id[], besluit_op, uitkomst, weigeringsgronden[], gerelateerd_betrokkenenverzoek_id`
De tool onderscheidt het informatieverzoek van het inzageverzoek scherp: andere termijn, andere weigeringsgronden, andere rechtsbescherming. Bevat één binnengekomen bericht beide, dan ontstaan twee dossiers met twee klokken en een onderlinge verwijzing.

**VERTEGENWOORDIGER_UNIE** *(nieuw)*
`id, entiteit_id, regime (avg|nis2), naam, vestigingsland, aanwijzingsbewijsstuk_id, contactgegevens, aangewezen_op, beëindigd_op`

**WPG_SPOOR** *(nieuw)*
`id, entiteit_id, van_toepassing_motivering, interne_controle_frequentie, laatste_interne_controle, externe_audit_frequentie (vierjaarlijks), laatste_externe_audit, auditrapport_bewijsstuk_id, verbeterplan_id, volgende_audit_verplichting_id`

**DPIA**
`id, verwerking_id, prescan_uitkomst, methode, uitgevoerd_door, datum, restrisico, art36_raadpleging_nodig, art36_ingediend_op, art36_termijn_verplichting_id, art36_opschorting_van, art36_opschorting_tot, art36_verlenging_toegepast, advies_ap_ref, besluit_door, herbeoordelingsdatum`

### 7.4 FG-positie, advies en onafhankelijkheid

**ADVIES**
`id, vraag, vraagsteller_id, afdeling, herkomst, urgentie, adviestekst, uitgebracht_aan_id, uitgebracht_op, tijdig_betrokken (ja|nee|deels), tijdig_toelichting`
Relaties: 1—1 `BESTUURSREACTIE (status: overgenomen|deels|niet|geen_reactie, motivering, beslisser_id, datum)`; 1—N `ESCALATIESTAP (niveau, datum, uitkomst)`; N—N `VERWERKING`, `SYSTEEM`, `LEVERANCIER`; 0—N `RISICO`.

**ONAFHANKELIJKHEIDSINCIDENT** (AVG art. 38 lid 3 en lid 6)
`id, soort (instructie_gegeven|toegang_geweigerd|capaciteit_geweigerd|belangenconflict|sanctie_gedreigd|beoordeling_gekoppeld_aan_advies), datum, omschrijving, betrokken_functionaris, opvolging, bewijsstuk_id`

**Persoonlijk FG-dossier** *(nieuw)*

De functionaris voor gegevensbescherming kan een tweede, kleine kluis aanmaken die uitsluitend met zijn eigen sleutel te openen is en waarvan de organisatie de inhoud niet kan lezen, exporteren of verwijderen. Daarin staan uitsluitend: `ONAFHANKELIJKHEIDSINCIDENT`, uitgebrachte adviezen met hun bestuursreactie, escalatiestappen en de daarbij horende bewijsstukken.

Records worden bij vastlegging in de organisatiekluis onzichtbaar gespiegeld als hash, zodat later kan worden aangetoond *dát* een advies op een bepaald moment bestond zonder de inhoud prijs te geven. Beide kluizen ankeren extern volgens §6.4, zodat de tijdstippen onafhankelijk toetsbaar zijn. Bij beëindiging van de aanstelling neemt de FG dit dossier mee; de organisatie behoudt de hashes en het gewone adviesregister.

Grondslag en bestaansreden: AVG art. 38 lid 3, dat verbiedt de FG te ontslaan of te straffen voor de uitvoering van zijn taken — een bescherming die waardeloos is als het bewijs ervan uitsluitend berust bij degene tegen wie zij is gericht. De juridische houdbaarheid van deze constructie tegenover eigendoms- en archiefaanspraken van de organisatie wordt vóór de eerste levering met een externe jurist vastgelegd, inclusief een modelbepaling voor de aanstellingsovereenkomst (zie risico R26).

### 7.5 Incident — het scharnierpunt

**INCIDENT**
`id, omschrijving, tijdstip_optreden, tijdstip_signaal, verificatieperiode_van, verificatieperiode_tot, verificatie_onderbouwing, tijdstip_kennisname (append-only), kennisname_onderbouwing (append-only), kennisname_bevestigd_door, monotone_teller, tijdzone_offset, ankerafwijking, oorzaakcategorie, status (lopend|afgehandeld), afgehandeld_op, gepland_onderhoud (ja|nee), onderhoudsvenster_id, rol_entiteit (verantwoordelijke|verwerker|beide)`

**DATALEKSPOOR (AVG)** — uitgebreid met de wettelijk voorgeschreven documentatievelden
`risicobeoordeling, waarschijnlijke_gevolgen (verplicht), genomen_of_voorgestelde_maatregelen (verplicht), categorieen_betrokkenen[], categorieen_persoonsgegevens[], aantal_betrokkenen, aantal_registraties, meldplicht_oordeel (melden|niet_melden), oordeel_motivering (verplicht), oordeel_door, oordeel_op, vertraging_bij_melding (ja|nee), vertraging_motivering (verplicht indien vertraging = ja), art34_oordeel, art34_uitzonderingsgrond (lid3_a_versleuteling|lid3_b_latere_maatregelen|lid3_c_onevenredige_inspanning|geen), art34_motivering, openbare_mededeling_ref (verplicht bij lid3_c), ap_verplichting_alsnog_informeren, betrokkenen_geinformeerd_op, melding_aan_verantwoordelijken[]`

De drie uitzonderingsgronden van art. 34 lid 3 zijn een gesloten keuzelijst, geen vrij tekstveld. Bij de grond "onevenredige inspanning" is de openbare mededeling verplicht en wordt zij als apart artefact met bewijsstuk vastgelegd.

**VERWERKERSMELDING** *(nieuw)*
`id, incident_id, richting (ontvangen|verzonden), tegenpartij_id, contractuele_termijn_uren, ontvangen_op, verzonden_op, inhoud_ref, bewijsstuk_id, achterwege_motivering`
De contractuele termijn wordt overgenomen uit `VERWERKERSOVEREENKOMST.art28_lid3_checklist_json` onderdeel f en levert een eigen `VERPLICHTING` op. Voor een entiteit die als verwerker optreedt, is dit de belangrijkste klok van allemaal — en de vorige versie kende hem niet.

**NIS2SPOOR (Cbw)**
`significantie_laag1_oordeel, laag1_motivering (verplicht), laag2_drempels_geraakt[], laag3_aggregatiegroep_id, omzetdrempel_toegepast, omzetdrempel_waarde, eindoordeel (significant|niet_significant), oordeel_door, oordeel_op, afnemers_informatieplicht_oordeel, afnemers_geinformeerd_op, afnemers_motivering_bij_niet`

**MELDING**
`id, incident_id, ontvanger_autoriteit_id, ontvanger_soort_resolved, soort (vroegtijdige_waarschuwing|melding|tussentijds_verslag|voortgangsverslag|eindverslag|aanvullende_melding_avg), verzendtijdstip, kanaal, referentienummer, ontvangstbevestiging_bewijsstuk_id, eerste_feedback_bewijsstuk_id, formuliersnapshot_id`

De soort `aanvullende_melding_avg` dekt de gefaseerde melding: informatie die niet gelijktijdig kon worden verstrekt, wordt zonder onnodige vertraging in fasen aangevuld. Elke aanvulling is een eigen record met eigen tijdstip en eigen bewijsstuk, gekoppeld aan de oorspronkelijke melding.

**AGGREGATIEGROEP**
`id, grondoorzaak, venster_start, venster_eind (rollend 6 maanden), aantal_incidenten, cumulatieve_schade_eur, absolute_drempel_eur, omzetdrempel_eur, toegepaste_drempel, drempel_bereikt, alarm_op`

> **Correctie op herziening 1.0.** De aggregatiedrempel is niet één vast bedrag. De drempel is het **laagste** van (a) het absolute bedrag uit de uitvoeringsverordening en (b) een percentage van de totale jaaromzet in het voorafgaande boekjaar. Bij een entiteit met een jaaromzet van 4 miljoen euro ligt de drempel daarmee aanzienlijk lager dan het absolute bedrag, en zou de oude implementatie een meldplichtig incident hebben gemist. Daarnaast geldt de eis van ten minste twee incidenten binnen zes maanden met dezelfde kennelijke oorzaak. Beide waarden staan in het contentpack, niet in de code.

**CYBERDREIGING** *(nieuw)*
`id, entiteit_id, omschrijving, bron, ontvangen_op, beoordeling_significant (ja|nee), beoordeling_motivering (verplicht), beoordelaar_id, afnemers_informatieplicht (ja|nee), te_treffen_maatregelen_omschrijving, afnemers_geinformeerd_op, motivering_bij_niet_informeren, gekoppelde_advisories[]`
Een significante cyberdreiging is geen incident en kent geen meldklok naar het CSIRT, maar wel een zelfstandige informatieplicht richting afnemers, inclusief de maatregelen die zij kunnen treffen. Dit ontbrak volledig in herziening 1.0.

**VRIJWILLIGE_MELDING** *(nieuw)*
`id, soort (bijna_incident|dreiging|kwetsbaarheid|niet_significant_incident), ontvanger_autoriteit_id, verzonden_op, inhoud_ref, bewijsstuk_id`
Vrijwillige meldingen zijn expliciet als vrijwillig gemarkeerd, genereren geen verplichtingen en worden in het auditdossier apart getoond, zodat zij nooit als bewijs van naleving van een meldplicht kunnen worden gelezen.

**Relaties op `INCIDENT`:** 1—0..1 `DATALEKSPOOR`; 1—0..1 `NIS2SPOOR`; 1—N `MELDING`; 1—N `VERWERKERSMELDING`; 1—N `VERPLICHTING`; N—N `SYSTEEM`, `VERWERKING`, `LEVERANCIER`; N—N `AGGREGATIEGROEP`.

### 7.6 NIS2-kant

**CONTROL**
`id, entiteit_id, raamwerkvariant (A|B|C), variant_c_regeling_ref, cbw_art21_lid3_onder (a..j), cbb_artikel, uv_bijlagepunt, naam, status (aantoonbaar|vastgesteld_niet_aantoonbaar|niet_ingericht|menselijk_oordeel_vereist), eigenaar_id, motivering_bij_niet_toepassing, volwassenheidsniveau, score_vaststelling, score_uitvoering, score_actualiteit, vervaldatum_aantoonbaarheid`

**RISICOBEOORDELING** *(nieuw)*
`id, entiteit_id, scope_omschrijving, methode, methode_bron, uitgevoerd_door, uitvoerdatum, geraadpleegde_bronnen[], geïdentificeerde_risicos[], restrisico_aanvaarding_door, bestuursvaststelling_besluit_id, geldig_tot, volgende_beoordeling_verplichting_id`
Herziening 1.0 kende wel `RISICO` maar niet de beoordeling zelf. Zonder het beoordelingsartefact — met methode, scope, uitvoerder, datum en bestuursvaststelling — is de zorgplicht niet aantoonbaar.

**BELEIDSDOCUMENT** — bitemporeel
`id, control_id, titel, versie, vaststeller_id, vaststellingsdatum, goedkeuringsbesluit_id, bewijsstuk_id`

**NORMBEPALING**
`id, raamwerk, versie, deelaanduiding, consolidatiedatum, aanduiding, titel, tekst_ref, bron, geverifieerd_door, geverifieerd_op`
Het veld `deelaanduiding` is toegevoegd omdat een norm uit meerdere delen kan bestaan met verschillende eisen; het door elkaar gebruiken van deel 1 en deel 2 in herziening 1.0 was een slordigheid met bewijsgevolg.

**MAPPING**
`id, van_normbepaling_id, naar_normbepaling_id, relatietype (gelijkwaardig|dekt_gedeeltelijk|breder|smaller|geen_dekking), dekkingsgewicht, motivering, bron, reviewer, reviewdatum, review_geldig_tot, contentpack_versie`

**LEVERANCIER**
`id, naam, is_rechtstreeks, kritikaliteit, kvk_nummer, land, cra_relevant, weringsbesluit_ref, weringsbesluit_vervangingstermijn, weringsbesluit_uitgevoerd_op`
Relaties: 1—N `CONTRACT` (0..1 `VERWERKERSOVEREENKOMST` met `art28_lid3_checklist_json` inclusief vindplaats per onderdeel a-h en de contractuele meldtermijn uit onderdeel f); 1—N `SUBVERWERKER` (boom); 1—N `LEVERANCIERSTOETS`; 1—N `CERTIFICAAT`; 0..1 `BEWIJSPAKKET` (module 15b).

**ADVISORY** (Cbb art. 17)
`id, bron (csirt|bevoegde_autoriteit|leverancier|overig), bron_naam, ontvangstdatum, cves[], omschrijving`
Relaties: 1—1 `BEOORDELING (verplicht schriftelijk: aanpassing_nodig ja|nee, motivering, beoordelaar_id, datum, bewijsstuk_id)`; 1—N `MAATREGEL`; 0—N `CYBERDREIGING`.

**BESTUURDER**
`id, entiteit_id, naam, functie, benoemingsdatum, rechtsvorm_variant, is_uitvoerend, einddatum`
Relaties: 1—N `TRAININGSCERTIFICAAT (naam_bestuurslid, trainingsdata[], behandelde_onderwerpen[], trainingsaanbieder, taal, bewijsstuk_id, alle_vier_gevalideerd)`; 1—N `VERPLICHTING`.

> **Deadlineregel.** De individuele deadline is `benoemingsdatum` plus de in het contentpack vastgelegde termijn. De collectieve backstop is **[RF-BEST-BACKSTOP]**. De geldende deadline is **de eerdere van de twee**. Voor een bestuurder die kort ná de inwerkingtreding is benoemd, ligt de individuele deadline ná de backstop en wint de backstop — de vorige versie behandelde de individuele deadline ten onrechte als altijd leidend.

**BESTUURSBESLUIT** `id, entiteit_id, datum, aanwezigen[], besluittekst, goedgekeurde_pakketversie, goedgekeurde_risicobeoordeling_id, bewijsstuk_id` · N—N `CONTROL`

**REGISTRATIEGEGEVEN**
`id, entiteit_id, register (nationaal|europees), veldnaam, huidige_waarde, initieel_gemeld_op, laatst_gemeld_op`
Relaties: 1 `INITIELE_REGISTRATIE (verplichting_id, uitgevoerd_op, referentie, bevestiging_bewijsstuk_id)`; 1—N `REGISTRATIEWIJZIGING (oude_waarde, nieuwe_waarde, wijzigingsdatum)`; elke wijziging genereert de van toepassing zijnde klok(ken) → 0..1 `REGISTRATIEMELDING`.

> **Correctie op herziening 1.0.** De registratieklokken waren intern inconsistent (op de ene plaats veertien dagen en drie maanden, op de andere één maand en drie maanden). Vastgesteld model: de nationale wijzigingstermijn is **twee weken**; voor de entiteitstypen met een Europese registratieplicht geldt daarnaast een wijzigingstermijn van **drie maanden**. De niet-onderbouwde termijn van één maand is geschrapt. De **initiële** registratie is een zelfstandige verplichting met een eigen termijn uit het contentpack — herziening 1.0 modelleerde alleen wijzigingen, wat betekende dat een nieuwe entiteit geen enkele registratieklok kreeg.

### 7.7 Toezicht, bestuursrecht, output en toegang

**TOEZICHTDOSSIER** `id, entiteit_id, autoriteit_id, geopend_op, procesfase (informeel_contact|toezicht|voornemen|handhaving|rechtsbescherming), fase_gewijzigd_op, status`

Bij elke faseovergang toont de tool wat er verandert:

| Overgang | Wat de tool doet |
|---|---|
| naar **voornemen** | Start de zienswijzeklok (Awb art. 4:8) en stelt een dossierbundel samen die precies het voornemen dekt |
| naar **handhaving** (besluit genomen) | Start de bezwaartermijn van zes weken vanaf de dag ná bekendmaking (Awb art. 6:7 en 6:8) als onverlengbare, rood gemarkeerde klok, met de beroepstermijn erachter |
| naar een **punitief traject** | Toont een niet-uitschakelbare melding over de verhouding tussen de medewerkingsplicht (Awb art. 5:20) en het zwijgrecht bij boeteoplegging (Awb art. 5:10a), met de aanbeveling juridische bijstand in te schakelen vóórdat verder wordt geantwoord |

**CORRESPONDENTIE** `soort, richting, datum, onderwerp, bewijsstuk_id, reactietermijn_verplichting_id`

**VERSTREKKINGSLOGBOEK** *(nieuw)*
`id, toezichtdossier_id, verstrekt_op, ontvanger, aanleiding, bundel_id, bundelhash, versie, redactieprofiel_id, verstrekt_door, ondertekening_ref`
Bij een latere uitvraag toont de tool wat de toezichthouder al heeft en waar de eerdere en de huidige verklaring van elkaar afwijken. Dat verschil zien vóór verzending is het verschil tussen een correctie en een tegenstrijdigheid in een dossier.

**BEVINDING** `herkomst, ernst, omschrijving, termijn, boetecategorie, status`
**MAATREGEL** `omschrijving, eigenaar_id, termijn, verificatiebewijs_id, status`
**HANDHAVINGSSTAP** `id, entiteit_id, stap, datum, termijn, besluit_ref, rechtsmiddel_verplichting_id, bewijsstuk_id`

**RAPPORT**
`id, soort (jaarverslag|bestuursrapportage|auditdossier|inspectie_export|deltarapport|vervalprognose), peildatum, sjabloonversie, contentpack_versies[], inhoudshash, handtekening_eidas_ref, handtekening_ed25519, ankerbestanden[], redactieprofiel, redactielogboek, terugleescontrole_uitkomst, aangeboden_aan_id, aanbiedingsdatum, bestuursreactie, definitief_verklaard_door`
Een rapport is zelf een onwijzigbaar bewijsstuk met een manifest van alle opgenomen bestanden en de bijbehorende ankerbestanden.

**TOEGANGSMODEL**
`GEBRUIKER` N—N `ROL` N—N `RECHT`, met daarnaast `COMPARTIMENT (id, naam, sleutel_id, acl_json, export_uitsluiting)`.
Compartimenttoegang is **additief** en wordt apart gelogd. Objecten in een compartiment zijn **onzichtbaar** — niet "zichtbaar maar geblokkeerd" — voor wie er geen toegang toe heeft, óók in zoekresultaten, exports, tellingen en rapportages, en die onzichtbaarheid berust op versleuteling en niet op een filterregel.

### 7.8 De vervalprognose

De tool weet van elk bewijsstuk de geldigheid, van elke control de frequentie, van elke mapping de reviewdatum, van elk certificaat de vervaldatum en van elk doorgifte-instrument de houdbaarheid. In herziening 1.0 werd die kennis alleen achteraf gebruikt.

**Vervalprognose.** Voor elk gekozen horizonpunt — dertig, negentig en driehonderdvijfenzestig dagen — toont de tool welke **wettelijke eisen** op dat moment niet langer aantoonbaar zijn en waarom:

| Oorzaak | Voorbeeld |
|---|---|
| Verlopen bewijsstuk | Het reviewrapport van de autorisatiereview verloopt over 41 dagen |
| Verstreken frequentie | De zelf vastgestelde halfjaarlijkse hersteltest is over 12 dagen te laat |
| Verlopen certificaat | Het certificaat van een kritieke leverancier verloopt over 63 dagen |
| Verlopen mandaat of contract | De verwerkersovereenkomst loopt af zonder verlenging |
| Verlopen mappingreview | De crosswalkrand is ouder dan de reviewgeldigheid en telt vanaf dan met gewicht nul |
| Vervallen doorgifte-instrument | Het instrument verloopt of de status van het adequaatheidsbesluit is gewijzigd |
| Verlopen bestuurdersopleiding | De actualiteitsteller van een bestuurslid loopt af |

De uitvoer is **geen lijst met taken maar een lijst met eisen die onbewijsbaar worden**, met de eigenaar en de benodigde doorlooptijd erbij. Dit is een eigen exporteerbaar rapport voor de bestuursvergadering, want dit is de enige vorm waarin een bestuur een informatiebeveiligingsrisico begrijpt: niet als kleur, maar als datum.

### 7.9 Invarianten die de kern afdwingt

| # | Invariant |
|---|---|
| I1 | Een `CONTROL` kan de status *aantoonbaar* niet bereiken zonder ten minste één `BEWIJSSTUK` met bewijsrol *uitvoering* waarvan `geldig_tot` in de toekomst ligt. |
| I2 | Een `VERPLICHTING` van soort *zelf_vastgesteld* is niet activeerbaar zonder `frequentie_vastgesteld_door` én `frequentie_vastgesteld_op`. |
| I3 | `INCIDENT.tijdstip_kennisname` en `.kennisname_onderbouwing` zijn na eerste vastlegging alleen te wijzigen via een correctierecord met vier-ogenakkoord. |
| I4 | Een `CONTROL` in variant B met een als "waar passend", "indien van toepassing" of "voor zover haalbaar" geformuleerde eis die niet wordt toegepast, blijft *onvolledig* zolang `motivering_bij_niet_toepassing` leeg is. |
| I5 | Een `NIS2SPOOR` kan niet worden afgesloten zonder `laag1_motivering`, ook niet wanneer geen enkele kwantitatieve drempel is geraakt. |
| I6 | Een `RAPPORT` kan niet *definitief* worden zonder expliciete ondertekening én zonder dat ontbrekend of verlopen bewijs in de bundel is benoemd. |
| I7 | Elk object met een `compartiment_id` is uitgesloten van elke query, export, telling en zoekindex van een gebruiker zonder de bijbehorende sleutel. |
| I8 | Een `VERWERKING` met bijzondere gegevens zonder `uitzondering_art9_10` levert een blokkerende hygiënebevinding op. |
| I9 | Een wijziging in `SYSTEEM.ip_bereiken` of `.domeinnamen` creëert automatisch een `REGISTRATIEWIJZIGING` met de bijbehorende klok(ken). |
| I10 | Een `VERWERKING` met een `DOORGIFTE` naar een derde land bereikt niet de status *volledig* zonder een geldig `instrument` waarvan `geldig_tot` in de toekomst ligt, dan wel een vastgelegde `art49_grond` met vastlegging in het register. |
| I11 | Een `DATALEKSPOOR` met `meldplicht_oordeel = melden` en een verzendtijdstip later dan de wettelijke termijn na `tijdstip_kennisname` kan niet worden afgesloten zonder `vertraging_motivering`. |
| I12 | Een `CONTROL` waarvan de `eigenaar_id` de aangemelde functionaris voor gegevensbescherming is, levert een blokkerende bevinding op wegens strijd met AVG art. 38 lid 6: de FG kan geen eigenaar zijn van een maatregel waarop hij toezicht houdt. |
| I13 | Een `VERWERKING` met `bsn_gebruik = ja` zonder `wettelijke_grondslag_ref` levert een blokkerende hygiënebevinding op. |
| I14 | Een `INCIDENT` waarbij de entiteit optreedt als verwerker kan niet worden afgesloten zonder ten minste één `VERWERKERSMELDING` met richting *verzonden*, dan wel een vastgelegde motivering waarom die achterwege bleef. |
| I15 | Een `NIS2SPOOR` met `eindoordeel = significant` kan niet worden afgesloten zonder een expliciet oordeel over de informatieplicht richting afnemers, inclusief motivering bij niet-informeren. |
| I16 | Een `VERWERKING` met grondslag art. 6 lid 1 onder f zonder afgeronde `BELANGENAFWEGING` blijft *onvolledig*. |
| I17 | Een `TOESTEMMING` zonder `bewijsvorm` en `bewijsstuk_id` blijft *onvolledig*; de bewijslast ligt bij de verwerkingsverantwoordelijke. |
| I18 | Een gehonoreerde rectificatie, wissing of beperking genereert per ontvanger één `ONTVANGERKENNISGEVING`; het verzoek kan niet worden afgesloten zolang er een openstaat zonder verzenddatum of zonder motivering van onmogelijkheid. |
| I19 | De kluis accepteert geen productiegegevens zonder twee geregistreerde authenticators, een geslaagde hersteltest en ten minste één ingericht ankerkanaal. |
| I20 | Elk klokgevoelig record draagt een monotone teller, een tijdzone-offset en de afwijking ten opzichte van het laatste externe anker. Een terugsprong van de systeemklok is een auditgebeurtenis die niet kan worden onderdrukt. |
| I21 | Een onversleutelde export is alleen mogelijk via een gelogde overrule met motivering; die overrule verschijnt in het auditdossier en in het verstrekkingslogboek. |
| I22 | Een `MELDING` kan niet als *verzonden* worden gemarkeerd zonder een op de peildatum geresolvede `AUTORITEIT` per ontvangersoort. |
| I23 | Een `BESTUURDER` zonder `TRAININGSCERTIFICAAT` met alle vier gevalideerde elementen houdt een openstaande verplichting met deadline `min(individuele deadline, collectieve backstop)`. |
| I24 | Een statuswijziging van een `ADEQUAATHEIDSBESLUIT` zet alle betrokken `DOORGIFTE`-records op *herbeoordelen* en genereert per record een verplichting met termijn. |
| I25 | Elke verstrekking aan een toezichthouder wordt vastgelegd in het `VERSTREKKINGSLOGBOEK` met bundelhash; bij een volgende verstrekking toont de tool de verschillen ten opzichte van de vorige vóórdat verzending mogelijk is. |
| I26 | Een contentpack met een `NORMBEPALING` zonder `geverifieerd_op` kan niet worden ondertekend en niet worden uitgeleverd. |
| I27 | Een sleutelhouder die is afgevoerd terwijl zijn sleutel nog actief is, levert een blokkerende bevinding op tot de rotatie is afgerond en het rotatiebewijsstuk is vastgelegd. |
| I28 | Een `RAPPORT` met redactie kan niet worden verstrekt zonder een geslaagde `terugleescontrole_uitkomst`. |

### 7.10 Bewaartermijnen van de tool zelf

De vorige versie bewaakte de bewaartermijnen van anderen zonder er zelf te hebben. Toegevoegd:

| Objecttype | Standaardbewaartermijn | Grondslag |
|---|---|---|
| Incidentdossier inclusief melding en bijlagen | Vijf jaar na afhandeling, plus één jaar marge | Verjaring van de bevoegdheid tot boeteoplegging (Awb art. 5:45) |
| Betrokkenenverzoek inclusief verstrekte bundel | Twee jaar na afhandeling; de verstrekte bundel zelf één jaar | Verantwoordingsplicht tegenover minimale gegevensverwerking |
| Zoekresultaten en tussenbestanden bij een verzoek | Verwijderd bij afsluiting van het verzoek, tenzij expliciet bewaard met motivering | Art. 5 lid 1 onder e |
| Auditlogboek en ketenankers | Tien jaar | Bewijsfunctie |
| Toezichtdossier | Tien jaar na afsluiting | Bewijsfunctie en rechtsbescherming |
| Persoonsgegevens in opleidingsregistratie | Duur van de aanstelling plus twee jaar | Minimale gegevensverwerking |

De opschoning draait als verplichting in de werkbak met een voorstel, nooit als stille automatische verwijdering: elk voorstel wordt bevestigd door een mens en het besluit wordt vastgelegd.

---

## 8. Compliance-mapping

### 8.1 Hoe de koppeling technisch werkt

De mapping is geen kruisjestabel maar een **graaf met onderbouwde randen**.

```
NORMBEPALING ──(MAPPING: relatietype, gewicht, motivering, bron, reviewer)──▶ NORMBEPALING
      │
      │ (KOPPELING: "wordt ingevuld door")
      ▼
   CONTROL ──(KOPPELING: bewijsrol)──▶ BEWIJSSTUK ──▶ geldigheidsvenster
```

| Relatietype | Betekenis | Gevolg voor de score |
|---|---|---|
| `gelijkwaardig` | De eisen dekken elkaar volledig | Bewijs telt 1:1 mee in beide raamwerken |
| `dekt_gedeeltelijk` | De norm dekt een deel van de wettelijke eis | Bewijs telt mee met het dekkingsgewicht; het restant blijft zichtbaar als gap |
| `breder` | De norm eist méér dan de wet | Bewijs telt volledig mee; overschot telt niet als tekort |
| `smaller` | De wet eist méér dan de norm | Verplicht aanvullend bewijs; expliciet zichtbaar in het delta-rapport |
| `geen_dekking` | De norm dekt deze wettelijke eis niet | **Verschijnt altijd in het delta-rapport**, ook als het raamwerk verder volledig is |

De reviewstatus is verplicht en heeft nu ook een **houdbaarheid** (`review_geldig_tot`). Een mapping zonder menselijke review, of met een verlopen review, draagt in de score met gewicht nul bij, wordt rood gemarkeerd en verschijnt in de vervalprognose.

### 8.2 Hoofdmapping: Cbw art. 21 lid 3 a-j

> **Let op de artikelverwijzing.** De tien categorieën staan in NIS2 art. 21 lid 2 onder a-j, maar in de Cyberbeveiligingswet in **art. 21 lid 3 onder a-j**. Alle Nederlandse output verwijst naar het derde lid, met NIS2 art. 21 lid 2 als EU-referentie.

De onderstaande tabel is de **doelstand**. In v1 wordt de crosswalk beperkt (§8.5); de overige kolommen komen als contentpack ná v1.

| Cbw art. 21 lid 3 | Cbb (variant A) | UV 2024/2690 bijlage (variant B) | ISO/IEC 27001:2022 | NEN 7510-2:2024 | Voorgeschreven overheidskader (variant C) | AVG |
|---|---|---|---|---|---|---|
| **a.** beleid risicoanalyse en beveiliging informatiesystemen | art. 6, art. 7 | punt 1, punt 2 | cl. 4-6, cl. 6.1.2-6.1.3, A.5.1, A.5.2, A.5.4, A.5.36 | §5.1, §5.2, §5.4 + zorgspecifieke aanvullingen | Beleidshoofdstuk + overheidsmaatregelen op A.5.1/A.5.2 | art. 24, art. 32 lid 1, art. 5 lid 2 |
| **b.** incidentenbehandeling | art. 8 | punt 3 | A.5.24-A.5.28, A.8.15, A.8.16 | §5.24-§5.28 | Overheidsmaatregelen op logging en detectie | art. 33, art. 34, art. 32 lid 1 onder b |
| **c.** bedrijfscontinuïteit, back-up, herstel, crisisbeheer | art. 9 | punt 4 | A.5.29, A.5.30, A.8.13, A.8.14 | §5.29, §5.30 + continuïteit van zorg | Overheidsmaatregelen op A.5.29/A.5.30 | art. 32 lid 1 onder b en c |
| **d.** beveiliging van de toeleveringsketen | art. 10 | punt 5 | A.5.19-A.5.23 | §5.19-§5.23 | Overheidsmaatregelen op leveranciersbeheer | art. 28, art. 29 |
| **e.** verwerving, ontwikkeling, onderhoud; kwetsbaarhedenrespons en -bekendmaking | art. 11, art. 17 | punt 6, punt 7 | A.8.8, A.8.9, A.8.25-A.8.34, A.5.37 | §8.8, §8.9, §8.25 e.v. | Overheidsmaatregelen op patch- en wijzigingsbeheer | art. 25, art. 32 |
| **f.** beleid en procedures om effectiviteit te beoordelen | art. 18 | punt 8 | cl. 9.1-9.3, A.5.35, A.5.36 | §5.35, §5.36 | Overheidsmaatregelen op evaluatie | art. 32 lid 1 onder d |
| **g.** cyberhygiëne en opleiding | art. 12 | punt 9 | A.6.3, A.5.4, A.8.7 | §6.3 + zorgspecifieke bewustwording | Overheidsmaatregelen op bewustwording | art. 39 lid 1 onder b, art. 32 lid 4 |
| **h.** cryptografie en encryptie | art. 13 | punt 10 | A.8.24 | §8.24 | Overheidsmaatregelen op cryptografie en sleutelbeheer | art. 32 lid 1 onder a |
| **i.** personeel, toegangsbeleid, assetbeheer | art. 14, 15, 16 | punt 11 | A.5.9-A.5.11, A.5.15-A.5.18, A.6.1-A.6.6, A.7.1-A.7.14, A.8.1-A.8.5 | §5.15-§5.18, §6.1-§6.6, §7.x | Overheidsmaatregelen op toegangsbeheer en fysieke beveiliging | art. 5 lid 1 onder f, art. 32 |
| **j.** meerfactorauthenticatie, beveiligde spraak-, video-, tekst- en noodcommunicatie | art. 9 lid 5, art. 15 | punt 12 | A.8.5, A.5.14, A.8.20, A.8.21 | §8.5, §5.14 | Overheidsmaatregelen op sterke authenticatie | art. 32 lid 1 |

> **Normaanduiding.** Waar in herziening 1.0 nu eens "NEN 7510-2:2024" en dan weer "NEN 7510:2024" stond, geldt vanaf deze versie: elke normverwijzing draagt raamwerk, **deelaanduiding**, versie en consolidatiedatum. Deel 1 en deel 2 zijn verschillende documenten met verschillende eisen; een bewijsstuk dat aan het verkeerde deel hangt, is in een audit een aangrijpingspunt. Dezelfde discipline geldt voor de puntversie van een technisch raamwerk.

### 8.3 Wat géén enkele norm dekt — het delta-rapport

Dit is de kern van de commerciële waarde en tegelijk de belangrijkste waarschuwing aan gecertificeerde organisaties.

| Verplichting | Grondslag | Waarom niet gedekt |
|---|---|---|
| Registratieplicht nationaal register, initiële registratie én wijziging binnen twee weken | Cbw art. 43-46; Cbb art. 27 | Administratieve verplichting jegens de overheid; geen onderwerp van een managementsysteem |
| Europese registratie met eigen wijzigingstermijn van drie maanden | Cbw art. 47-48 | Idem |
| Meldplicht in vier trappen aan CSIRT én bevoegde autoriteit, met per sector een ander CSIRT | Cbw art. 25-29; Cbb art. 23-25 | Normen kennen incidentbeheer, niet deze externe meldketen, ontvangers en termijnen |
| Informatieplicht richting afnemers bij een significant incident | Cbw art. 30 | Geen equivalent |
| **Informatieplicht bij een significante cyberdreiging**, inclusief de te treffen maatregelen | NIS2 art. 23 lid 2 en Cbw-pendant | Geen normequivalent |
| Bestuursgoedkeuring van het maatregelenpakket per pakketversie | Cbw art. 24 lid 1 | Leiderschapseisen zijn geen formeel goedkeuringsbesluit |
| Trainingscertificaat per bestuurslid met vier verplichte elementen | Cbw art. 24 lid 2-5; Cbb art. 21-22 | Volledig nationaal; geen norm kent deze eis |
| Schriftelijke beoordeling van elke ontvangen advisory | Cbb art. 17 | Normen kennen dreigingsinformatie, niet de documentatieplicht per attendering |
| Weringsbesluiten met vervangingstermijn | Cbw art. 21a | Uniek nationaal instrument |
| Vertrouwelijke, bij besluit opgelegde significantiedrempels | Cbb art. 23 lid 4 | Geen normequivalent |
| Motiveringsplicht bij niet-toepassing in variant B | UV 2024/2690 art. 2 lid 2 | Verwant aan de verklaring van toepasselijkheid, maar andere reikwijdte en bewijslast |
| Beoordeling FG-plicht en aanmelding van de FG | AVG art. 37; UAVG | Buiten het bereik van een managementsysteem |
| **Doorgiftebeoordeling en instrumentbeheer met vervaldatum** | AVG art. 44-49 | Normen kennen geen juridisch doorgifteregime |
| **Kennisgeving aan ontvangers bij rectificatie, wissing of beperking** | AVG art. 19 | Geen equivalent |
| **Melding aan de verwerkingsverantwoordelijke door de verwerker** | AVG art. 33 lid 2 | Normen kennen incidentbeheer, niet deze contractuele meldketen |
| **Gezamenlijke verwerkingsverantwoordelijkheid: regeling en wezenlijke inhoud** | AVG art. 26 | Geen equivalent |
| **Register geautomatiseerde besluitvorming met beschrijving van de onderliggende logica** | AVG art. 13-15, art. 22; UAVG art. 40 | Geen equivalent |
| **Privacyaudit politiegegevens** | Wpg art. 33 en het bijbehorende besluit | Volledig buiten het managementsysteem |
| **Bestuursrechtelijke rechtsbescherming: zienswijze, bezwaar, beroep** | Awb art. 4:8, 6:7, 6:8 | Procesrecht, geen normenkader |
| **Transparantieplichten voor algoritmische systemen** | Verordening (EU) 2024/1689 art. 50 | Ander regime, andere toezichthouder |
| **Verplichtingen als fabrikant van een product met digitale elementen** | Verordening (EU) 2024/2847 | Productregime, geen organisatienorm |

Vaste vermelding in elk delta-rapport: zie bijlage A.

### 8.4 Sectorale en aanvullende raamwerken in dezelfde graaf

| Raamwerk | Positie in het model |
|---|---|
| **UV (EU) 2024/2690** bijlage | Volwaardig raamwerk (variant B); vervangt Cbb art. 6-18 voor de entiteitstypen van Cbb art. 4 |
| **Sectorale ministeriële regelingen** | Drie afzonderlijke koppelingen: voorgeschreven normenkader (variant C), drempelset op het NIS2-spoor, en aanvullende verplichtingen. Elke regeling krijgt een eigen `NORMBEPALING`-set met versie en consolidatiedatum, met de open norm van Cbw art. 25 lid 2 altijd als vangnet erboven |
| **Technische controlraamwerken** (bijvoorbeeld een puntversie van een internationale controllijst) | **Ná v1.** Uitsluitend als hulpmiddel voor technische invulling, nooit als juridische referentie, en altijd met puntversie |
| **DORA** | Geen mapping maar een **regime-verhouding per verplichtingssoort** (§3.6). De identificatie- en registratiesystematiek blijft van toepassing |
| **CRA** | Complementair productraamwerk met eigen tijdlijn (**[RF-CRA-MELD]**, **[RF-CRA-VOL]**); aan de inkoopkant een toetsingscriterium in het leveranciersregister, aan de leverancierskant een eigen verplichting (§10.7) |
| **Verordening (EU) 2024/1689** | Eigen spoor via `GEAUTOMATISEERDE_BESLUITVORMING`; geen crosswalk naar de zorgplicht |
| **Archiefwet en selectielijsten** | Gekoppeld aan `VERWERKING.bewaartermijn` met conflictsignalering tegen de AVG-bewaartermijn |

### 8.5 Crosswalkafbakening voor v1

Een crosswalk naar tien raamwerken betekent tien keer een jaarlijkse hercontrole, tien keer een aansprakelijkheidsvraag en tien keer een reviewbudget. Voor v1 geldt daarom:

| In v1 | Ná v1, als contentpack |
|---|---|
| Cbw, Cbb, UV 2024/2690, AVG/UAVG | Aanvullende sectornormen |
| **Eén** sectornorm naar keuze van de klantsector: ISO/IEC 27001:2022, NEN 7510-2:2024 of het voorgeschreven overheidskader | Technische controlraamwerken |
| — | Elk raamwerk zonder wettelijke werking |

Deze afbakening staat in de verkoopdocumentatie. Een klant die een tweede sectornorm wil, krijgt die als contentpackuitbreiding met eigen review en eigen prijs — niet als gratis toevoeging waarvan het onderhoud onzichtbaar bij de ontwikkelaar blijft liggen.

---

## 9. Roadmap en inspanningsschatting

### 9.1 Herziene uitgangspunten van de schatting

Eén ontwikkelaar, dertig productieve uren per week. Op de ontwikkeltijd per werkpakket wordt een **opslag van veertig procent** gelegd voor integratie, foutherstel en releasewerk. Vanaf de eerste betalende klant wordt bovendien een substantieel deel van de beschikbare tijd gereserveerd voor support, implementatie en contentonderhoud. De onderstaande weken zijn ontwikkeltijd vóór die opslag.

> **Realistische doorlooptijd tot een volwaardige v1: 33 tot 45 maanden.** De oorspronkelijke schatting van 14 tot 17 maanden ontbeerde support, foutherstel, implementatie bij de eerste klanten, documentatie, afhankelijkheidsonderhoud en releasewerk per platform. Wie deze doorlooptijd niet accepteert, moet de scope halveren en niet de schatting — zie §9.9 voor de uitgewerkte gehalveerde variant.

### 9.2 Fase −1 — Inhoud en marktbewijs

**Duur: 6-8 weken** · *Resultaat: het juridische fundament bestaat en drie organisaties hebben ervoor getekend, vóórdat er één regel productiecode is geschreven.*

Het onderscheidend vermogen zit in de contentpacks. Die in fase 4 maken, nadat de machine er staat, is de omgekeerde volgorde voor risico én voor omzet.

| Werkpakket | Weken |
|---|---|
| Contentpackformaat als platte, versienummerde tekstbestanden in versiebeheer; normbepalingen Cbw, Cbb, UV 2024/2690 en AVG met vindplaats, consolidatiedatum en verificatiestempel | 2,0 |
| Termijnencatalogus: elke verplichting met anker, duur, eenheid, rekenregel, grondslag en tegenvoorbeeld — eerst op papier, dan pas in code | 1,5 |
| Rechtsfeitencatalogus (§3.2) en autoriteiten-/CSIRT-tabel | 0,5 |
| Crosswalk Cbw naar één sectornorm, met relatietype en motivering per rand | 1,5 |
| Handmatige pilot bij drie organisaties met uitsluitend deze inhoud en bestaande hulpmiddelen | 2,0 |

*Mijlpaal:* drie organisaties bevestigen schriftelijk dat zij voor het onderhoud van deze inhoud willen betalen. Wordt die mijlpaal niet gehaald, dan is het contentpack-abonnement geen prijsmodel en vervalt de zakelijke basis onder het hele plan.

### 9.3 Fase 0 — Fundering en kluiskern

**Duur: 13-16 weken** · *Resultaat: een lege kluis die je kunt openen, sluiten, back-uppen, herstellen, verankeren en verifiëren.*

| Werkpakket | Weken |
|---|---|
| **Spike (weken 1-3): CTAP2 `hmac-secret`, versleutelde opslag en hashketen op drie platforms** — hierna het stackbeslispunt | 3,0 |
| Werkruimte, cratesindeling, CI-pijplijn, gepinde toolchain | 1,0 |
| `dpofg-crypto`: sleutelafleiding met gebonden parameters, envelope, domeinseparatie, geheugenhygiëne | 1,5 |
| CTAP2-integratie met verplichte tweede authenticator | 1,0 |
| `dpofg-store`: opslaglaag conform het beslispunt, migratieraamwerk, blob store, beperkte bitemporaliteit | 2,0 |
| **Envelopeversleuteling per compartiment inclusief aparte versleutelde zoekindex** | 1,5 |
| `dpofg-audit`: hashketen, append-only mechaniek, ketenankers, klokdiscipline met monotone teller | 1,5 |
| **Ankerkanalen: offline tijdstempelstroom, digest-export, ankerimport en -verificatie** | 1,5 |
| `dpofg-verify` CLI + verificatietestsuite + **gepubliceerde formaatspecificatie met testvectoren** | 1,0 |
| Shamir-herstel (SLIP-0039-compatibel), shares met levenscyclus, recovery kit, verplichte hersteltest | 1,0 |
| **Sandboxspecificatie en -implementatie per platform, met faaltests** | 1,0 |
| Schil, allowlist op de brug, inhoudsbeleid, ontgrendelscherm, zelftest bij start | 1,5 |

*Mijlpaal:* een externe partij kan met `dpofg-verify` en de gepubliceerde specificatie aantonen dat een testkluis niet is gemanipuleerd, en kan de ankerbestanden onafhankelijk controleren.

### 9.4 Fase 1 — MVP "Bewijs & Klok"

**Duur: 26-32 weken** · *Resultaat: verkoopbaar aan de eerste drie klanten.*

Het bestuurdersregister en de registratieplicht zijn uit fase 3 naar voren gehaald: dat zijn verplichtingen die vanaf **[RF-CBW-IWT]** lopen, waarvan er één op **[RF-BEST-BACKSTOP]** afloopt, en de bestuurdersopleiding is de eerste verplichting waarop naar verwachting wordt gehandhaafd.

| Werkpakket | Weken |
|---|---|
| Module 4 — termijnenmotor met **getypeerde termijnen**, kalenders uit het contentpack, escalatie, golden-testsuite | 4,0 |
| Module 3 — scope-, regime- en **autoriteiten-/CSIRT-resolver**, omvangstoets met tweejaarsregel en consolidatie, classificatiebesluit-artefact | 3,5 |
| Module 6 — normen-/kennisbank, contentpackformaat in code, handtekening- en versiecontrole, crosswalkqueries | 3,0 |
| Module 5 — bewijskluis, chain of custody, houdbaarheidsmotor, gesandboxte import | 2,5 |
| Module 7 — verwerkingsregister (beide schema's), **doorgiften, gezamenlijke verantwoordelijkheid, BSN, strafrechtelijke gegevens, toestemming, belangenafweging**, hygiënecontroles, tijdmachine | 4,5 |
| Module 8 — incident- en meldcockpit, vijf klokken, twee sporen, significantie-engine met omzetdrempel, **verwerkersmeldketen**, **cyberdreiging**, **vrijwillige melding**, formulieren per geresolvede autoriteit | 5,0 |
| **Registratieplichtdossier: initiële registratie plus dubbele wijzigingsklok** | 1,5 |
| **Bestuurdersregister, certificaatvalidatie op vier elementen, goedkeuringsworkflow** | 2,0 |
| Module 9 — adviesregister, comply-or-explain, onafhankelijkheidsincidenten, **persoonlijk FG-dossier** | 2,0 |
| Module 2 — FG-cockpit / werkbak | 1,5 |
| Module 10 (kern) — dossierbundel, manifest, ankerbijlage, toezichthoudersviewer, kwaliteitspoort, **verstrekkingslogboek** | 2,5 |
| Module 1 — rollen en compartimenten volledig doorgevoerd door alle modules, inclusief de volledige testmatrix | 2,0 |
| Startbibliotheken gemeente / zorg / onderwijs / middenbedrijf | 2,0 |
| Installers, hardeningshandleiding, dreigingsmodel, DPIA op de tool, modelovereenkomst | 1,5 |

**Extern:** circa 80-100 uur juridisch review op de contentpacks (Cbb-controlset, meldformuliervelden, significantiedrempels en -formules, termijnencatalogus, doorgifte-instrumenten) vóór de eerste betalende klant.

*Mijlpaal:* een gemeente kan een volledig incidentdossier voeren over beide regimes, het register bijhouden inclusief doorgiften, en binnen tien minuten een geverifieerd, extern verankerd auditdossier voor de bevoegde autoriteit produceren.

### 9.5 Fase 2 — v1a "Register verdiept en verzoeken"

**Duur: 13-16 weken**

| Werkpakket | Weken |
|---|---|
| Module 13 — betrokkenenverzoeken met **maandtermijnlogica**, zoekorkestratie, **art. 19-kennisgeving aan ontvangers**, art. 12 lid 4-bericht | 3,5 |
| **Woo-spoor** met eigen termijnen, weigeringsgronden en het onderscheid met het inzageverzoek | 1,5 |
| **Redactieregie (2,0)** — zie §9.5.1 | 2,0 |
| Module 14 — DPIA, pre-scan, **LIA**, **TIA**, art. 36-raadpleging met opschorting en verlenging, doorgiftebeoordeling | 3,5 |
| **Eén generieke veldmappingtool** met bewaarde profielen en verschilrapport (vervangt drie afzonderlijke driftimporters) | 1,5 |
| Gedistribueerde registerreviewcyclus | 1,0 |
| **Wpg-spoor** met auditverplichting en verbeterplan | 1,0 |

#### 9.5.1 Redactieregie in plaats van een eigen redactiepijplijn

De tool bepaalt *wat* moet worden geredigeerd en bewijst *dat* het is gebeurd; zij voert de redactie op de beeld- en tekstlaag niet zelf uit. Concreet: identificatie van te redigeren passages op basis van het register en van patroonlijsten, een uitvoerpakket naar een aangewezen extern redactiehulpmiddel, en daarna een **verplichte terugleescontrole** op het teruggeleverde bestand: tekstextractie, metadata- en annotatiescan, en een pixelvergelijking op de geredigeerde gebieden. Slaagt de controle niet, dan blokkeert de tool verstrekking (invariant I28).

Deze keuze is bewust. Het zelf bouwen van een redactiepijplijn met tekstlaagbewerking, tekenherkenning op scans en patroonherkenning is een zelfstandig product van twaalf tot twintig weken om het veilig te doen; de begroting van 3,5 week in herziening 1.0 was een orde van grootte mis. Belangrijker: het zou de meest waarschijnlijke oorzaak van een datalek *dóór* de tool in eigen beheer nemen, en dat past niet bij een product dat door één persoon wordt onderhouden. Het besluit wordt herzien wanneer er een tweede ontwikkelaar is.

### 9.6 Fase 3 — v1b "Zorgplicht en keten"

**Duur: 13-16 weken**

| Werkpakket | Weken |
|---|---|
| Module 12 — volledige Cbb-controlset variant A + variant B (UV 2024/2690) + **variant C (voorgeschreven kader)** | 4,5 |
| **Risicobeoordeling als artefact** met methode, scope, bestuursvaststelling en herbeoordelingsklok | 1,5 |
| Module 11 — driefactorscore, gap-analyse, correctieplicht, **vervalprognose op 30/90/365 dagen als eigen rapport** | 3,5 |
| Module 15 — leveranciers, art. 28-checklist met vindplaats per onderdeel, Cbb-leverancierstoetsing, advisory-inbox met verplichte schriftelijke beoordeling, weringsbesluiten | 3,5 |
| Handhavingsescalatieladder + boete-exposure | 1,0 |
| Kwetsbaarheden- en meldingsregister voor gecoördineerde bekendmaking + CRA-toetsingscriterium in de inkoopkant | 1,5 |

### 9.7 Fase 4 — v1c "Toezicht, rapportage en keten"

**Duur: 12-15 weken**

| Werkpakket | Weken |
|---|---|
| Module 16 — toezichtdossier, correspondentie, bevindingenregister, maatregelen | 2,0 |
| **Module 16b — bestuursrechtelijk spoor**: procesfasen, zienswijze-, bezwaar- en beroepsklokken, cautiesignalering, verstrekkingslogboek met verschilweergave | 2,5 |
| Module 10 (uitbreiding) — jaarverslag, bestuursrapportage, delta-rapport, sjabloonbeheer, **handtekening conform de Europese systematiek** | 3,5 |
| **Module 15b — ketenbewijs**: bewijspakketformaat, gratis hulpbinary, import met handtekening- en ankerverificatie, geldigheidsbewaking | 3,0 |
| Module 17 (versoberd) — bestuurdersopleidingsspoor met certificaatvalidatie; deelnamebewijs op groepsniveau | 1,0 |
| Toegankelijkheid: toetsenbordbediening, contrast en labels als bouwvoorschrift, plus een herstelronde | 1,0 |

**Extern:** externe penetratietest en code-audit op `dpofg-crypto`, `dpofg-store` en `dpofg-audit`, plus een gerichte review van de ankerimplementatie (circa 5-7 dagen extern werk plus 2 weken herstel).

### 9.8 Fase 5 — v2 "Meerdere entiteiten en samenwerking"

**Duur: 6-8 weken**

| Werkpakket | Weken |
|---|---|
| Module 18b — multi-entiteit, uitcheck-/synchronisatiemodel met **keten per apparaat, epoch-teller en kruisondertekening** | 4,5 |
| Overdrachtsdossier bij FG-wisseling, inclusief de scheiding met het persoonlijke FG-dossier | 1,5 |
| Module 18a — intakepoortje als aparte binary | **Uit de basisplanning; pas begroten wanneer het criterium is gehaald** |
| On-premise servervariant | **Vervalt uit de planning** — zie bijlage B |
| Benchmarkkengetallen | **Vervalt** — vereist gegevens over klanten heen en botst frontaal met het uitgangspunt dat er geen egress is. Eventueel later als statisch contentpack op basis van openbare bronnen |

### 9.9 Samenvatting en de gehalveerde variant

| Fase | Ontwikkelweken | Met opslag 40% |
|---|---|---|
| −1 Inhoud en marktbewijs | 6-8 | 8-11 |
| 0 Fundering en kluiskern | 13-16 | 18-22 |
| 1 MVP | 26-32 | 36-45 |
| 2 Register en verzoeken | 13-16 | 18-22 |
| 3 Zorgplicht en keten | 13-16 | 18-22 |
| 4 Toezicht en rapportage | 12-15 | 17-21 |
| **Totaal tot v1** | **83-103** | **115-144** |
| 5 Multi-entiteit | 6-8 | 8-11 |

Met volledige capaciteit tot en met fase 1 en een gereduceerde capaciteit daarna komt de kalenderdoorlooptijd tot v1 uit op **33 tot 45 maanden**.

**Gehalveerde variant (v1-kern, circa 18-22 maanden).** Wanneer die doorlooptijd onaanvaardbaar is, is dit de scope die overblijft:

| Wel in v1-kern | Niet in v1-kern |
|---|---|
| Fasen −1, 0 en 1 volledig | Module 12 controlset variant B en C |
| Verwerkingsregister inclusief doorgiften en bijzondere categorieën | Module 11 vervalprognose |
| Incident- en meldcockpit volledig | Module 13 en 14 (verzoeken, DPIA, redactieregie) |
| Registratie- en bestuurdersspoor | Module 15b ketenbewijs |
| Dossierbundel met externe verankering | Module 16b bestuursrechtelijk spoor |
| Eén crosswalk: Cbw/Cbb/UV naar AVG | Wpg- en Woo-spoor |

De keuze tussen beide varianten is een prijs- en positioneringskeuze en wordt vóór de eerste offerte gemaakt.

### 9.10 Doorlopend — contentpackonderhoud

| Activiteit | Inspanning |
|---|---|
| Bijhouden wetteksten, toezichthouderstandpunten, Europese richtsnoeren | 6-8 uur per week |
| Statusbewaking adequaatheidsbesluiten en doorgifte-instrumenten | 2 uur per week |
| Nieuwe sectorale ministeriële regeling verwerken (drie koppelingen, §3.5) | 2-3 weken per regeling |
| Grote wetswijziging, bijvoorbeeld de periodieke evaluatie van de significantiecriteria | 2-3 weken plus juridisch review |
| Jaarlijkse hercontrole van alle crosswalkmappings | 2 weken plus 20 uur juridisch review |
| Onderhoud van de rechtsfeitencatalogus en de feestdagenkalenders | 4 uur per jaar per jurisdictie |

> **Waarschuwing bij de planning.** Zonder een geloofwaardige, volgehouden updatecadans op de contentpacks is het product binnen een jaar een *risico* in plaats van een hulpmiddel. Het contentonderhoud is geen bijzaak naast de ontwikkeling; het is de duurzame kostenpost en moet in de prijsstelling terugkomen.

### 9.11 Volgorde-argumentatie

De volgorde is bepaald door juridische hardheid en dagelijkse pijn, niet door technische gemakkelijkheid:

1. **Inhoud vóór machine.** Het onderscheidend vermogen zit in de contentpacks; die worden nu eerst gemaakt en handmatig bij drie klanten beproefd.
2. **Kluiskern vóór alles**, inclusief externe verankering — zonder verankering is niets bewijs.
3. **De stackbeslissing in week 3**, niet in week 44. Aan het eind van fase 0 is de kluiskern juist het enige wat af is; overstappen zou dan betekenen dat precies het moeilijkste deel wordt weggegooid.
4. **De termijnenmotor vóór alle inhoudelijke modules**, omdat elke module er verplichtingen in hangt.
5. **De incidentcockpit vóór de zorgplichtcontrolset**: een gemiste meldtermijn is onherstelbaar, een half ingevulde controlset niet.
6. **Bestuurders- en registratiespoor in fase 1**, omdat dit de verplichtingen zijn die nu al lopen en waarop het eerst wordt gehandhaafd.
7. **Het verwerkingsregister vóór de betrokkenenverzoeken**, omdat de zoekorkestratie het register nodig heeft.
8. **Geen verstrekking zonder redactiecontrole.** Herziening 1.0 kon in fase 1 al een auditdossier exporteren terwijl de redactiecontrole pas in fase 2 kwam. Correctie: in fase 1 is verstrekking uitsluitend mogelijk via een profiel dat gecompartimenteerde en als gevoelig gemarkeerde inhoud volledig uitsluit; vrije redactie wordt pas ontgrendeld met de terugleescontrole in fase 2.

---

## 10. Kwaliteitsborging

### 10.1 Teststrategie

| Laag | Aanpak |
|---|---|
| **Domeininvarianten** | Unittests op `dpofg-domain` (puur, geen I/O). Elke invariant uit §7.9 heeft ten minste één positieve en één negatieve test |
| **Termijnenmotor — golden tests** | Circa 250 uitgeschreven casussen als tekstfixtures met invoer, verwacht anker, verwachte deadline, **verwachte toegepaste rekenregel** en verwachte statusovergangen. Zie §10.2 |
| **Termijnenmotor — eigenschapstests** | Een deadline ligt nooit vóór het anker; een pauze verlengt de resterende termijn met precies de pauzeduur; herberekening na ankerwijziging is idempotent; een maandtermijn wordt nooit intern in dagen uitgedrukt |
| **Contentpack** | Tests dat elke rechtsfeitcode wordt opgelost, dat elke normbepaling een verificatiestempel heeft, dat de handtekening en de volgordecontrole werken, en dat een testcontentpack met verschoven datums identieke code doet slagen |
| **Crypto** | Known-answer tests tegen de officiële testvectoren; roundtrip wrap/unwrap; **downgradetests op de parameters van de sleutelafleiding**; Shamir-reconstructie bij alle geldige en ongeldige sharecombinaties, inclusief gemanipuleerde shares die moeten worden geweigerd in plaats van een verkeerd geheim op te leveren |
| **Auditketen en ankers** | Tests die bewust een record wijzigen, verwijderen of invoegen en verifiëren dat `dpofg-verify` exact het juiste volgnummer als breukpunt aanwijst; tests op klokterugsprong; tests op ketensplitsing en -samenvoeging bij uitchecken |
| **Opslag** | Migratietests voorwaarts over alle schemaversies; corruptiesimulatie op blob store en database; **tests dat journaal- en tijdelijke bestanden geen leesbare inhoud bevatten** |
| **Importparsers** | Fuzzing van alle parsers; **verificatie per platform dat het parserproces daadwerkelijk zonder netwerk en zonder schrijfrechten draait en dat een crash het kernproces niet meeneemt** |
| **Autorisatie en compartimenten** | Testmatrix rol × objecttype × compartiment die verifieert dat een object in geen enkel codepad verschijnt: query, zoekindex, export, rapport, telling, foutmelding, aggregaat |
| **Export, redactie en verstrekking** | Golden tests op bundelstructuur en manifest; een testcase die een vertrouwelijk compartiment vult en verifieert dat elk redactieprofiel het uitsluit; **negatieve terugleestests waarin een geredigeerde term bewust in de tekstlaag, in metadata en in een annotatie achterblijft en verstrekking moet worden geblokkeerd** |
| **Geïmporteerde inhoud** | Tests met vijandige bestandsnamen, e-mailbodies en leveranciersnamen die verifiëren dat niets als opmaak wordt geïnterpreteerd |
| **Schil** | Componenttests en scenario's van begin tot eind: kluis openen, incident aanmaken, vijf klokken zien lopen, dossier exporteren, anker verifiëren |
| **Toegankelijkheid** | Geautomatiseerde controle op elke route in CI, plus een handmatige toetsenbord- en schermlezerronde per release |
| **Platformmatrix** | Elke release op de drie ondersteunde platforms; de compatibiliteitsmatrix inclusief de ondergrens van de engineversie wordt bij de release gepubliceerd |

### 10.2 Termijnrekenkundig uitgangspunt en de verplichte randgevallen

Dit onderdeel is in herziening 1.0 fout gegaan op de kern van het product en is volledig herschreven.

> **Termijnrekenkundig uitgangspunt.** De motor rekent termijnen in uren, dagen, weken, maanden en jaren als afzonderlijke typen en zet een maandtermijn nooit om in dagen.
>
> 1. Termijnen uitgedrukt in **uren** (24 en 72 uur) lopen in kalendertijd door weekenden en feestdagen heen en worden nooit verlengd. Grondslag: Verordening (EEG, Euratom) nr. 1182/71, art. 3 lid 1 en 2.
> 2. Termijnen uitgedrukt in **dagen, weken, maanden of jaren** die voortvloeien uit Unierecht en eindigen op een zaterdag, zondag of feestdag, lopen af aan het einde van de eerstvolgende werkdag. Grondslag: Verordening 1182/71, art. 3 lid 4.
> 3. Termijnen uit **nationale wetgeving en algemeen verbindende voorschriften** volgen de Algemene termijnenwet, tenzij de betreffende regeling anders bepaalt.
> 4. De **feestdagenkalender** is contentpackinhoud met een jurisdictie- en jaaraanduiding, geen code.
> 5. Bij elke berekende deadline toont de interface **welke van deze regels is toegepast en op welke bepaling zij berust**.
> 6. Bij een omstreden interpretatie — zoals opschorting bij identiteitsverificatie — biedt de tool beide lezingen aan, met bronvermelding, en legt de gekozen lezing met motivering vast in het dossier. Een interpretatie uit richtsnoeren wordt nooit hard in de motor gebakken.

| # | Casus | Verwacht gedrag |
|---|---|---|
| T-01 | Kennisname vrijdag 16:40; 72-uursklok | Deadline maandag 16:40, kalendertijd, geen verlenging |
| T-02 | Kennisname op een feestdag; urentermijn | Idem; urentermijnen lopen door feestdagen heen |
| T-03 | Overgang zomertijd binnen het venster | Berekend in UTC, weergegeven in lokale tijd; venster blijft exact 72 uur |
| T-04 | Schrikkeljaar, kennisname 29 februari, eindverslag "één maand later" | 29 maart; in een niet-schrikkeljaar wordt 29 februari niet als anker geaccepteerd |
| T-05 | Eindverslag ná de melding, niet ná het incident | Anker is `MELDING.verzendtijdstip` van de betreffende melding, niet `INCIDENT.tijdstip_kennisname` |
| T-06 | Incident duurt voort op de eindverslagdatum | Voortgangsverslagtaak nú + nieuw eindverslag één maand ná `afgehandeld_op` |
| T-07 | Entiteitstype met verkorte meldtermijn | De klok is 24 uur, niet 72; waarde uit het contentpack per entiteitstype |
| T-08 | Kennisname vóór optreden (invoerfout) | Blokkerende validatie met toelichting |
| T-09 | Correctie van het kennisnametijdstip | Alle vijf klokken herberekend; oude waarden zichtbaar; auditrecord met vier-ogenakkoord |
| T-10 | Registratiewijziging van een IP-bereik bij een entiteit met Europese registratieplicht | Twee verplichtingen: twee weken (nationaal) én drie maanden (Europees) |
| T-11 | Registratiewijziging bij een entiteit zonder Europese registratieplicht | Alleen de tweewekenklok |
| T-12 | Inzageverzoek ontvangen 15 januari, verlenging medegedeeld 20 februari | Verlenging geweigerd: de mededeling moest binnen de eerste maand, dus uiterlijk 15 februari, of de eerstvolgende werkdag. De motor rekent in maanden, niet in dagen |
| T-13 | Identiteitsverificatie: opschorting van de termijn | Twee lezingen aangeboden met bronvermelding; de gekozen lezing wordt met motivering vastgelegd; geen harde regel in de motor |
| T-14 | Bestuurslid benoemd twee jaar ná inwerkingtreding | Individuele deadline = benoemingsdatum + contentpacktermijn; collectieve backstop is dan verstreken en geldt niet voor deze persoon |
| T-15 | Bestuurslid benoemd vóór inwerkingtreding | Deadline is de collectieve backstop |
| T-16 | Zelf vastgestelde frequentie ontbreekt | Verplichting is niet activeerbaar; control blijft *menselijk oordeel vereist* |
| T-17 | Twee incidenten, dezelfde grondoorzaak, binnen zes maanden, entiteit met hoge omzet | Absolute drempel is van toepassing omdat die lager is dan het omzetpercentage; aggregatiegroep bereikt de drempel; alarm |
| T-18 | Idem, maar zeven maanden uit elkaar | Geen aggregatie |
| T-19 | Onbeschikbaarheid tijdens een geregistreerd onderhoudsvenster | Uitgesloten van de significantietoets |
| T-20 | Verplichting met ankerdatum vóór de inwerkingtreding | Achterstand berekend vanaf **[RF-CBW-IWT]**, gemarkeerd als *verplicht sinds* |
| T-21 | Inzageverzoek ontvangen 15 januari; maandtermijn | Deadline 15 februari; valt die op zaterdag, zondag of feestdag, dan de eerstvolgende werkdag |
| T-22 | Inzageverzoek ontvangen 31 januari | Deadline 28 februari, respectievelijk 29 februari in een schrikkeljaar |
| T-23 | Registratiewijziging waarvan de tweewekentermijn eindigt op een algemeen erkende feestdag | Verlengd tot de eerstvolgende werkdag; de 24- en 72-uurstermijnen worden níet verlengd |
| T-24 | Twee incidenten, dezelfde grondoorzaak, binnen zes maanden, ja| T-24 | Twee incidenten, dezelfde grondoorzaak, binnen zes maanden, jaaromzet 4 miljoen euro, schade tweemaal 120.000 euro | De drempel is het percentage van de omzet (200.000 euro), niet het absolute bedrag; aggregatiegroep bereikt de drempel en slaat alarm |
| T-25 | Entiteitstype met verkorte meldtermijn, kennisname 10:00 | Vroegtijdige waarschuwing én melding vallen samen op 24 uur; de tool toont één klok met twee verplichtingen en waarschuwt tegen dubbeltelling |
| T-26 | Datalek bij een verwerker, contractueel 24 uur | De klok naar de verwerkingsverantwoordelijke start bij kennisname van de verwerker, onafhankelijk van de 72-uursklok van de verantwoordelijke |
| T-27 | Art. 36-raadpleging ingediend, aanvullende informatie opgevraagd op dag 20 | Termijn van acht weken opgeschort tot ontvangst; de verlenging met zes weken is apart zichtbaar en apart te motiveren |
| T-28 | Besluit van de toezichthouder bekendgemaakt op 3 september | Bezwaartermijn zes weken vanaf de dag ná bekendmaking; harde, niet-verlengbare klok met eigen escalatie, gevolgd door de beroepstermijn |
| T-29 | Adequaatheidsbesluit voor het ontvangstland wordt ingetrokken | Alle `DOORGIFTE`-records met dat instrument gaan naar *ongeldig*; per record ontstaat een herbeoordelingsverplichting met termijn; de betrokken verwerkingen verliezen de status *volledig* |
| T-30 | Bestuurslid benoemd kort ná inwerkingtreding | Individuele deadline ligt ná de collectieve backstop; de backstop is dan de eerdere van de twee en wint |
| T-31 | Melding ván een verwerker ontvangen | De 72-uursklok van de verwerkingsverantwoordelijke start bij ontvangst van die melding, niet bij het optreden van het incident bij de verwerker; beide tijdstippen worden vastgelegd |
| T-32 | Gehonoreerde rectificatie met vier ontvangers | Vier `ONTVANGERKENNISGEVING`-verplichtingen; bij één ontvanger onevenredige inspanning met motivering; het verzoek is pas af te sluiten als alle vier een uitkomst hebben |
| T-33 | Eén bericht bevat zowel een informatieverzoek als een inzageverzoek | Twee dossiers, twee klokken (vier weken met eenmalige verdaging van twee weken, respectievelijk één maand met verlengingsmogelijkheid), met onderlinge verwijzing |
| T-34 | Verzoek wordt niet gehonoreerd | Bericht binnen één maand met vermelding van de redenen, het klachtrecht bij de toezichthouder en het rechtsmiddel; dat bericht is zelf een verplichting met eigen klok |
| T-35 | Doorgifte op grond van een uitzondering | Vastlegging in het register is verplicht en blokkerend; de informatieplicht richting de betrokkene is een aparte verplichting |
| T-36 | Systeemklok springt terug tussen twee records | Auditgebeurtenis die niet kan worden onderdrukt; de keten blijft geldig, maar elk daarna geproduceerd dossier vermeldt de afwijking en de laatst bekende ankerpositie |
| T-37 | Omvangsdrempel in één boekjaar overschreden | Geen statuswijziging; pas bij twee opeenvolgende boekjaren wijzigt de classificatie, met een nieuw classificatiebesluit |
| T-38 | Deelneming van 30 procent en een tweede van 60 procent | De eerste telt naar rato mee, de tweede volledig; de rekenketen is zichtbaar in het classificatiebesluit |
| T-39 | Vrijwillige melding van een bijna-incident | Geen verplichtingen, wel dossier; expliciet als vrijwillig gemarkeerd en in het auditdossier apart getoond |
| T-40 | Significante cyberdreiging vastgesteld | Informatieplicht richting afnemers met de te treffen maatregelen; geen meldklok naar het CSIRT tenzij het contentpack die voorschrijft; motivering verplicht bij niet-informeren |

### 10.3 CI/CD

Elke commit op de hoofdlijn en elke wijzigingsaanvraag doorloopt:

```
opmaak → statische analyse (waarschuwingen zijn fouten) → licentie- en
afhankelijkheidscontrole → kwetsbaarhedencontrole → handmatige-auditcontrole
  → workspace-tests → eigenschapstests → fuzz-rookproef (60 s per doel)
  → contentpackvalidatie (rechtsfeitcodes, verificatiestempels, handtekening)
  → front-endcontroles → toegankelijkheidscontrole per route
  → build (3 platforms) → scenario's van begin tot eind
  → sandboxfaaltests per platform
  → materiaallijst (gecombineerd voor beide afhankelijkheidsbomen)
```

Bij een release komen daar bovenop: codesigning, notarisatie, hashsommen, releasehandtekening, publicatie van de materiaallijst en van de compatibiliteitsmatrix inclusief de ondergrens van de engineversie, en het aanmaken van de contentpackversie die bij de release hoort.

**Geen automatische publicatie.** Releases worden handmatig vrijgegeven na een handmatige rookproef op alle drie de platforms. Er is geen releasepijplijn die zonder menselijke tussenkomst een binary bij klanten kan krijgen — dat zou hetzelfde risico introduceren dat het productontwerp juist vermijdt.

### 10.4 Materiaallijst, licenties en toeleveringsketen

| Maatregel | Invulling |
|---|---|
| Materiaallijst | Gecombineerd voor beide afhankelijkheidsbomen, ondertekend en gepubliceerd bij elke release |
| Licentiecontrole | Expliciete allowlist; elke nieuwe licentie vereist handmatige goedkeuring |
| Kwetsbaarheden | Geautomatiseerde controle in CI én wekelijks buiten de releasecyclus |
| Handmatige audits | Vastgelegde audits op de crypto- en opslagafhankelijkheden, opgeslagen in de repository |
| **Reproduceerbaarheid — versoberd** | Build in een gepinde containerimage met een gepubliceerd bouwrecept en gepubliceerde hashes. **Byte-identieke uitvoer over drie platforms vervalt als criterium en als CI-stap**: het is weken werk aan een claim die vrijwel geen klant narekent, en de zekerheid die het toevoegt boven een ondertekende materiaallijst met een gepubliceerd bouwrecept is klein |
| Duplicaten | Meerdere versies van dezelfde afhankelijkheid worden geblokkeerd, om onduidelijke crypto-afhankelijkheden te voorkomen |
| Front-end | Bewust minimale afhankelijkheidsboom; elke nieuwe afhankelijkheid vereist een geschreven motivering in de wijzigingsaanvraag |

### 10.5 Release-signing en verifieerbaarheid van de uitvoer

| Artefact | Ondertekening |
|---|---|
| Installer per platform | Codesigningcertificaat op hardwaretoken; notarisatie waar het platform dat vereist |
| Linux-pakketten | Losse handtekening + gepubliceerde hashsom |
| `dpofg-verify` | Apart ondertekend — dit is het artefact dat toezichthouders draaien |
| Contentpacks | Ondertekend tegen een **aparte** uitgeverssleutel, ingebakken in de binary; versie- en volgordecontrole tegen terugrollen |
| **Exportbundels van klanten** | **Handtekening conform de Europese systematiek op het document**, zodat een toezichthouder de handtekening kan controleren met gereedschap dat hij al heeft, plus de losse handtekening en de ankerbestanden als aanvulling |

Sleutelbeheer aan de uitgeverskant: de contentpack-ondertekensleutel en de codesigningsleutel liggen op verschillende hardwaretokens, met een gedocumenteerde intrekkingsprocedure en een in de binary opgenomen tweede, nog niet gebruikte uitgeverssleutel als opvolgingspad.

### 10.6 Security testing

| Activiteit | Frequentie |
|---|---|
| Dreigingsmodel actualiseren per component | Bij elke nieuwe module, minimaal jaarlijks |
| Externe penetratietest op de applicatie | Jaarlijks; rapport beschikbaar voor klanten |
| Externe code-audit op `dpofg-crypto`, `dpofg-store`, `dpofg-audit` en de ankerimplementatie | Bij de eerste v1-release en daarna bij elke ingrijpende wijziging in die crates |
| Fuzzing van importparsers | Continu in CI (rookproef) + langdurige runs bij elke release |
| Herstel- en back-uptest van een productiekluis bij een pilotklant | Halfjaarlijks, met verslag |
| **Ankerverificatie-oefening**: een derde controleert een dossier zonder hulp van de leverancier | Halfjaarlijks bij ten minste één klant |
| Zelftest bij elke start bij de klant | Schijfversleuteling actief, back-uplocatie buiten synchronisatiemappen, zoekindex uitgesloten, geen roaming profiel, geen netwerkpad voor de kluis, engineversie boven de ondergrens, ankerkanaal ingericht en niet achterstallig |

De **hardeningshandleiding**, het **dreigingsmodel**, de **formaatspecificatie** en de **DPIA op de tool zelf** worden als product meegeleverd — de FG moet de eigen tool aan het eigen bestuur kunnen verantwoorden, en het is inconsequent om dat werk bij de klant te leggen.

### 10.7 Verplichtingen van de leverancier zelf

`dpo-fg-tool` is een product met digitale elementen. De maker is daarmee fabrikant in de zin van Verordening (EU) 2024/2847. Een product dat klanten helpt aan de Cbw te voldoen terwijl de maker zijn eigen verordening niet naleeft, is bij de eerste inspectie onverkoopbaar.

| Verplichting | Invulling | Uiterlijk |
|---|---|---|
| Beleid voor gecoördineerde bekendmaking van kwetsbaarheden | Gepubliceerd beleid met contactpunt en `security.txt` op het publicatiedomein | Vóór de eerste release |
| Kwetsbaarhedenafhandeling met termijnen | Vastgelegd proces: ontvangst, triage, hersteltermijn per ernst, bulletin | Vóór de eerste release |
| Ondersteuningsperiode | Vastgelegd in de licentievoorwaarden en op de productpagina | Vóór de eerste release |
| Materiaallijst per release | Zie §10.4 | Vanaf de eerste release |
| Meldprocedure richting het CSIRT en de Europese instantie | Ingericht, beproefd, met contactgegevens en sjablonen | Vóór **[RF-CRA-MELD]** |
| Volledige conformiteit inclusief technische documentatie | Gepland traject met externe toets | Vóór **[RF-CRA-VOL]** |

De tool bewaakt deze verplichtingen in de eigen installatie van de ontwikkelaar, in dezelfde vorm waarin een klant zijn verplichtingen bewaakt. Dat is geen aardigheidje: het is de goedkoopste manier om te merken dat een onderdeel in de praktijk niet werkt.

---

## 11. Risico's en mitigaties

| # | Risico | Ernst | Mitigatie |
|---|---|---|---|
| R1 | **Juridische drift.** Sectorale regelingen, toezichthouderstandpunten, Europese richtsnoeren en de periodieke evaluatie van de significantiecriteria veranderen de inhoud sneller dan de software | Hoog | Alle juridische inhoud én alle datums strikt buiten de binary in ondertekende contentpacks (§3.2); wijzigingsjournaal met impactanalyse op de eigen dossiers; vast onderhoudsbudget van 6-8 uur per week; expliciete onderhoudsverplichting in het contract |
| R2 | **Bus factor één.** Eén ontwikkelaar is tegelijk het verkoopargument en het grootste risico voor een organisatie die er haar wettelijke bewijsvoering aan ophangt | Hoog | Broncode-escrow of open-sourcing van `dpofg-crypto`, `dpofg-store`, `dpofg-audit` en `dpofg-verify`; gepubliceerd databaseschema; **gepubliceerde formaatspecificatie met testvectoren**; volledige niet-proprietaire export. Expliciet in de verkoopdocumentatie, want elke FG stelt deze vraag terecht |
| R3 | **Sleutelverlies is definitief.** Geen leveranciersreset betekent dat een verloren wachtwoordzin plus verloren shares het complete dossier vernietigt | Hoog | Verplichte hersteltest vóór de eerste productiegegevens; twee verplichte authenticators; shares met levenscyclus en halfjaarlijkse bevestiging; fysieke shares bij drie bewaarders; herhaalde waarschuwing. Accepteren dat dit klanten kost en het inhoudelijk verdedigen |
| R4 | **Verkeerde termijnimplementatie** is een aansprakelijkheidsrisico van het product zelf | Hoog | Getypeerde termijnen met expliciete rekenregels (§10.2); 250 golden tests; de wettelijke grondslag én de toegepaste rekenregel staan naast elke klok; omstreden interpretaties worden als keuze voorgelegd, niet als feit; beperking van aansprakelijkheid in de licentievoorwaarden plus beroepsaansprakelijkheidsverzekering |
| R5 | **Schijnzekerheid.** Een groen dashboard verleidt tot "wij voldoen" | Hoog | Geen enkele status heet "compliant"; vier statussen waarvan er twee expliciet een tekort of een openstaand oordeel benoemen; overal onderscheid tussen zelfgerapporteerd en geverifieerd bewijs; vaste waarschuwingsteksten (bijlage A) |
| R6 | **Meergebruikerswerken over een netwerkschijf.** Een databasebestand op een gedeelde schijf raakt vroeg of laat corrupt, en de doelgroep zal dit toch proberen | Hoog | Actieve detectie van netwerkpaden met blokkerende waarschuwing bij het openen; expliciet uitcheckmodel met keten per apparaat; pas in v2 een echt synchronisatiepad. Dit blijft de meest waarschijnlijke oorzaak van dataverlies in het veld |
| R7 | **Migratie en instap.** Een FG die al drie jaar in werkbladen werkt, stapt niet over als de instap een maand kost | Hoog | Generieke veldmappingtool met bewaarde profielen; sectorale startbibliotheken. De eerste dag moet eindigen met een gevuld register, niet met een leeg scherm |
| R8 | **Scopecreep richting een brede beheersuite.** Elke aangrenzende behoefte trekt aan het ontwerp. Voor één ontwikkelaar is dat fataal | Hoog | Harde afbakening op §1.2; alles daarbuiten via import/export ontsluiten in plaats van namaken. §12 is bindend |
| R9 | **De tool wordt zelf een hoogrisicoverwerking**: bijzondere persoonsgegevens uit inzageverzoeken, de identiteit van klagers, het complete kwetsbaarhedenbeeld | Hoog | Meegeleverde DPIA; **eigen bewaartermijnen met bevestigde opschoning (§7.10)**; compartimentering als standaard en cryptografisch afgedwongen; aanbeveling de tool op een aparte, versleutelde en niet-roaming werkplek te draaien |
| R10 | **Onnauwkeurige informatie aan de toezichthouder** door een achteloze export | Hoog | Blokkerende volledigheidspoort; ontbrekend of verlopen bewijs wordt benoemd in plaats van weggelaten; verplichte menselijke ondertekening met verklaring; **verstrekkingslogboek met verschilweergave tegenover eerdere verstrekkingen**; gelogde overrule als enige uitweg |
| R11 | **Redactiefout in een antwoord op een verzoek** waardoor de tool zelf de bron van een datalek wordt | Hoog | **De tool voert de redactie niet zelf uit** (§9.5.1); verplichte terugleescontrole op tekstlaag, metadata, annotaties en beeld; blokkering bij een gevonden term; vier-ogenoptie op verstrekking |
| R12 | **Vertrouwelijke drempels lekken in een gedeelde rapportage** | Middel | Apart compartiment met eigen sleutel en exportuitsluiting; expliciete testcase in de kwaliteitspoort; plaatsaanduiding in de bundel die het bestaan meldt zonder de inhoud |
| R13 | **Platformafhankelijkheden buiten eigen controle**: uiteenlopende engineversies op beheerde werkplekken, één engine die structureel achterloopt op beveiligingsupdates | Hoog *(was: middel)* | Dit is een kwetsbaarheidsvraag, geen compatibiliteitsvraag. Harde ondergrens op de engineversie met startweigering; uitgeschakelde crash-, spellings- en reputatiediensten; conservatieve front-endkeuzes; CTAP2 rechtstreeks in de kern; compatibiliteitsmatrix per release; offline installatiebootstrap |
| R14 | **Licentie- en afhankelijkheidsrisico** op de opslaglaag | Laag *(was: middel)* | De licentievraag is minder scherp dan herziening 1.0 suggereerde; de echte reden om te kiezen is architectonisch. Beslispunt in fase 0 (bijlage B) met envelopeversleuteling per compartiment als voorkeurspad |
| R15 | **Instabiliteit van de documentgeneratiebibliotheek** raakt de documentuitvoer | Laag/Middel | Versie gepind; laag achter een trait met terugvalimplementatie; **byte-gelijkheid vervalt als criterium** ten gunste van verifieerbaarheid van de handtekening |
| R16 | **Offline werken kost functionaliteit die gebruikers gewend zijn** | Middel | Uitstekende conceptvorming en klembordstromen (met wisbeleid); agenda- en berichtexport als bestand; een eerlijke uitleg waarom automatische indiening bewust ontbreekt |
| R17 | **Verwachtingsmanagement rond toezicht.** Toezichthouders gedragen zich het eerste jaar verkennend, wat klanten kan verleiden tot uitstel | Middel | De tool toont het verschil expliciet: *verplicht sinds* **[RF-CBW-IWT]**, met achterstand als risico en niet als toekomstige opgave. **Genuanceerd ten opzichte van herziening 1.0**: de stelling dat er geen overgangsrecht bestaat wordt niet als harde juridische bewering gevoerd; de tekst luidt dat verkennend toezicht beleid is en geen recht, en dat evenredigheid en de beginselen van behoorlijk bestuur de handhaving sturen. De precieze formulering en de vindplaats zijn contentpackinhoud met verificatiestempel |
| R18 | **Overschatting van de crosswalk.** Klanten kunnen de mappings lezen als een garantie dat certificeringsbewijs volstaat | Middel | Mappings dragen altijd relatietype, dekkingsgewicht, motivering en reviewhoudbaarheid; `geen_dekking`-randen verschijnen dwingend in het delta-rapport; vaste waarschuwingstekst; **crosswalk in v1 beperkt tot vijf raamwerken (§8.5)** |
| R19 | **Leercurve op de gekozen stack vertraagt fase 0 en 1** | Middel | **Beslispunt naar week 3** na een gerichte spike op de drie moeilijkste onderdelen; de tweede-keuzestack is uitgewerkt en het besluit valt vóórdat er iets is gebouwd dat weggegooid zou moeten worden |
| R20 | **Contentpackfouten** verspreiden zich naar alle klanten tegelijk | Middel | Juridisch review vóór ondertekening; **verificatiestempel per normbepaling, blokkerend bij ondertekening (I26)**; contentpacks per entiteit te pinnen; wijzigingsjournaal met impactanalyse; terugrolpad |
| R21 | **Prijsdruk.** Een lokaal product zonder abonnementsdwang is lastiger te verkopen dan een abonnementsdienst | Laag/Middel | Prijsstelling gekoppeld aan het contentpack-abonnement (het echte doorlopende werk), niet aan de licentie; de licentie blijft eeuwigdurend zodat een klant bij beëindiging zijn dossier houdt |
| **R22** | **De leverancier is zelf fabrikant onder de CRA.** Verplichtingen inzake kwetsbaarhedenbeheer, ondersteuningsperiode en melding; de meldplicht geldt vanaf **[RF-CRA-MELD]** | Hoog | Eigen beleid voor gecoördineerde bekendmaking en `security.txt` vóór de eerste release; kwetsbaarhedenafhandelingsproces met termijnen; materiaallijst per release; vastgelegde ondersteuningsperiode; meldprocedure ingericht en beproefd — zie §10.7 |
| **R23** | **Geen updatekanaal betekent geen herstelvermogen.** Zonder automatische update én zonder telemetrie is onbekend wie een versie met een bekend lek draait | Hoog | Elk contentpack draagt `minimaal_aanbevolen_applicatieversie` en een ondertekend beveiligingsbulletin; de tool waarschuwt blokkerend bij een te oude versie **zonder ooit zelf contact te zoeken**; een gepubliceerde adviesmailinglijst waarop elke klant bij ingebruikname wordt ingeschreven, contractueel vastgelegd; de zelftest meldt hoe oud het geïnstalleerde contentpack is |
| **R24** | **Externe verankering wordt bij de klant niet ingericht**, waarmee de bewijskracht vervalt terwijl het dossier er wel officieel uitziet | Hoog | Blokkerende poort: de kluis accepteert geen productiegegevens zonder ingericht ankerkanaal (I19); achterstallige verankering is een zichtbare, niet te onderdrukken melding in elk dossier; de vaste tekst in bijlage A benoemt het ontbreken expliciet in de bundel zelf |
| **R25** | **Verificatieschuld in het contentpack**: normbepalingen en vindplaatsen die zijn overgenomen zonder bronverificatie, met een onjuiste artikelverwijzing in een auditdossier als gevolg | Hoog | `geverifieerd_door` en `geverifieerd_op` per normbepaling, blokkerend bij ondertekening (I26); jaarlijkse hercontrole met budget; **alle vaste waarschuwingsteksten met een artikelverwijzing zijn contentpackinhoud, niet code**, zodat een fout te corrigeren is zonder release |
| **R26** | **Het persoonlijke FG-dossier wordt juridisch betwist** door de organisatie, die stelt dat het om organisatiegegevens gaat | Middel | Vóór de eerste levering een juridische notitie plus modelbepaling voor de aanstellingsovereenkomst; in de organisatiekluis staan uitsluitend hashes, zodat het bestaan aantoonbaar is zonder de inhoud; de constructie wordt in de documentatie expliciet als omstreden benoemd, met de keuze bij de FG |
| **R27** | **Het ketenbewijsformaat wordt niet geadopteerd** en blijft een functie die niemand gebruikt | Middel | Hulpbinary gratis en apart ondertekend; het formaat is onderdeel van de openbare specificatie; het criterium voor doorontwikkeling is dat ten minste tien leveranciers van bestaande klanten een pakket hebben geleverd; zonder die adoptie blijft het bij import van gewone bewijsstukken |

---

## 12. Wat de tool niet doet

Deze afbakening is bindend. Elke functie hieronder is bewust afgewezen, met reden. Verzoeken om ze alsnog te bouwen worden getoetst aan §1.2 en aan risico R8.

### 12.1 Geen handelingen namens de organisatie

- **Geen automatische indiening bij enig meldportaal of meldformulier van een toezichthouder.** De tool stelt de melding samen tot op het veld nauwkeurig, resolvet het juiste CSIRT en de juiste bevoegde autoriteit, en levert kopieerbare tekst plus een ondertekend document. De verzending is een menselijke handeling. Reden: een tool die namens een organisatie met een toezichthouder communiceert, kan een onbedoelde of onvolledige melding versturen — en dat is zelf een verzwaringsgrond.
- **Geen automatische berichtverzending.** De tool opent conceptberichten in de standaardclient; verzenden doet de gebruiker.
- **Geen automatische agendakoppeling.** Wel agenda-export als bestand.
- **Geen automatische opvolging bij leveranciers of collega's.** Wel gebundelde conceptherinneringen.

### 12.2 Geen operationele beveiligingsfuncties

- **Geen logcollectie, geen monitoring, geen detectieplatform.** De tool registreert dát er logging is, met welke bewaartermijn en met welke bescherming; zij verzamelt de logs niet.
- **Geen kwetsbaarhedenscanner en geen testgereedschap.** De tool registreert bevindingen en hun opvolging.
- **Geen netwerk- of assetontdekking.** De assetinventaris wordt gevoed door offline import en handmatige invoer.
- **Geen back-upsoftware voor de organisatie.** De tool bewaakt de back-upverificatie als verplichting en bewaart het verificatiebewijs.
- **Geen identiteits- of toegangsbeheer.** De tool bewaakt de autorisatiereviewcyclus en bewaart de reviewlogs; zij kent en wijzigt geen rechten in andere systemen.

### 12.3 Geen aangrenzende bedrijfssoftware

- **Geen ticketsysteem of servicedesk.** De werkbak beheert uitsluitend de eigen zaken van de FG en de security officer.
- **Geen contractmanagementsuite.** Het leveranciersregister bevat wat wettelijk moet worden getoetst en vastgelegd.
- **Geen personeelssysteem.** Trainingsregistratie beperkt zich tot wat de wet vergt.
- **Geen leerplatform en — nieuw in deze herziening — geen campagnebeheer.** Herziening 1.0 kende campagnes met deelnameregistratie per persoon. Dat is geschrapt: elk leerplatform doet het beter, en het trekt personeelsgegevens de kluis in, wat risico R9 verergert. Wat overblijft is het registreren van bewijs van deelname **op groepsniveau** plus het bestuurdersopleidingsspoor, dat wettelijk per persoon moet.
- **Geen documentmanagementsysteem.** De bewijskluis is doelgebonden: alleen wat als bewijs dient, met een geldigheidsvenster.
- **Geen organisatiebrede beheersuite** voor financieel, kwaliteits- of personeelsrisicomanagement.
- **Geen redactiegereedschap.** De tool bepaalt wat geredigeerd moet worden en controleert of het is gebeurd; de bewerking zelf gebeurt in een daarvoor bestemd hulpmiddel (§9.5.1).

### 12.4 Geen juridische functies

- **Geen juridisch advies.** De tool citeert wetteksten met bron, versie en consolidatiedatum, biedt modeladviezen en analogieën uit erkende richtsnoeren, en dwingt bij elke open norm een menselijk oordeel met motivering af. Zij velt geen oordeel.
- **Geen conformiteitsverklaring.** Er is geen scherm, rapport of certificaat waarin staat dat een organisatie voldoet.
- **Geen certificering.** De tool is geen certificerende instelling.
- **Geen automatische beslissing over meldplicht of significantie.** De significantie-engine doet een voorstel met zichtbare redenering; het besluit — inclusief het besluit *niet* te melden — is altijd van een mens, met motivering, tijdstip en naam.
- **Geen uitvoering van het DORA-regime.** De regime-resolver bepaalt per verplichtingssoort welk regime geldt en legt de motivering vast; de DORA-verplichtingen zelf worden niet ondersteund.
- **Geen procesvertegenwoordiging.** Het bestuursrechtelijke spoor bewaakt termijnen, bundelt dossiers en signaleert het cautiepunt; het schrijft geen bezwaarschrift en vervangt geen advocaat.

### 12.5 Geen dienstverlening op afstand

- **Geen gehoste variant, geen meervoudige huisvesting, geen accountsysteem bij de leverancier.**
- **Geen telemetrie, geen gebruiksstatistiek, geen crashrapportage naar buiten.** Dit is nu ook op het niveau van de schil afgedwongen (§6.5), niet alleen als voornemen.
- **Geen automatische updater** — met als tegenwicht de maatregelen bij risico R23.
- **Geen mobiele applicatie.** De dataset hoort niet op een telefoon.
- **Geen webversie.**
- **Geen servervariant.** In herziening 1.0 stond een on-premise servervariant in fase 5 begroot op vier weken. Dat is een tweede product met andere authenticatie, andere back-up, andere hardening en meerdere gelijktijdige schrijvers; de schatting was een orde van grootte mis. De variant vervalt uit de planning en wordt pas opnieuw begroot onder de voorwaarden in bijlage B.
- **Geen realtime samenwerking met meerdere gelijktijdige schrijvers.** Samenwerking loopt via een expliciet uitcheckmodel met keten per apparaat, of via ondertekende dossieroverdracht.
- **Geen sleutelherstel door de leverancier.** Er is geen achterdeur en er komt er geen. Het noodontgrendelpad met een kortcode uit herziening 1.0 is om precies deze reden geschrapt.
- **Geen benchmarkgegevens over klanten heen.**

### 12.6 Wat de tool bewust wél aan de organisatie laat

- Het **vaststellen van de periodieke frequenties** waar de wet er geen noemt. De tool dwingt de keuze af en meet ertegen, maar maakt de keuze niet.
- Het **beoordelen of een incident significant is** en of een datalek meldplichtig is.
- Het **kiezen tussen twee verdedigbare lezingen** van een omstreden termijnregel.
- Het **accepteren van restrisico's**.
- Het **goedkeuren van het maatregelenpakket en de risicobeoordeling** door het bestuur.
- Het **daadwerkelijk uitvoeren** van tests, reviews, opleidingen, redacties en leverancierstoetsingen. De tool bewaakt, registreert en bewijst; zij voert niet uit.

---

## Bijlage A — Vaste waarschuwingsteksten

Deze teksten staan letterlijk in de betreffende uitvoer en zijn niet door de gebruiker uit te zetten. **Alle teksten met een artikelverwijzing zijn contentpackinhoud met verificatiestempel**, zodat een onjuiste vindplaats te corrigeren is zonder release (risico R25).

**A1 — In elk delta-rapport:**
> Een certificaat op grond van een informatiebeveiligingsnorm is geen wettelijke conformiteitsverklaring onder de Cyberbeveiligingswet. Cbb art. 6 lid 4 eist een managementsystematiek om aantoonbaar te kunnen voldoen; het eist geen certificaat, en een certificaat vervangt de meld-, registratie-, informatie- en bestuurderstrainingsverplichtingen niet. De hieronder genoemde verplichtingen worden door geen van de in dit rapport betrokken normen gedekt.

**A2 — Bij elke significantiebeoordeling:**
> De sectorale drempels zijn niet uitputtend. De open norm van Cbw art. 25 lid 2 blijft als vangnet gelden: een incident kan significant zijn ook wanneer geen enkele kwantitatieve drempel wordt geraakt. Waar een drempel is uitgedrukt als een absoluut bedrag én als een percentage van de jaaromzet, geldt het laagste van beide. Een oordeel zonder motivering wordt niet vastgelegd.

**A3 — Bij elke termijn van 24 of 72 uur:**
> "Onverwijld" gaat vóór de maximumtermijn. 24 en 72 uur zijn plafonds, geen streefwaarden. Deze termijnen lopen in kalendertijd door weekenden en feestdagen heen en worden nooit verlengd. Voor de Cyberbeveiligingswet start de klok bij kennisname van het significante incident, niet bij het optreden ervan en niet bij de bevestigde diagnose. Voor de Algemene verordening gegevensbescherming veronderstelt kennisname een redelijke mate van zekerheid dat een inbreuk heeft plaatsgevonden; een korte eerste verificatie is toegestaan, mits die zelf wordt gedocumenteerd en niet als uitstel wordt gebruikt.

**A4 — Bij elke berekende deadline in dagen, weken, maanden of jaren:**
> Deze termijn is berekend volgens [toegepaste regel] op grond van [vindplaats]. Termijnen in maanden worden nooit in dagen omgerekend. Eindigt de termijn op een zaterdag, zondag of algemeen erkende feestdag, dan loopt zij af aan het einde van de eerstvolgende werkdag. De gehanteerde feestdagenkalender is [jurisdictie, jaar, contentpackversie].

**A5 — Bij elk auditdossier dat ontbrekend of verlopen bewijs bevat:**
> Dit dossier bevat onderdelen waarvoor geen geldig bewijsstuk aanwezig is. Deze onderdelen zijn in de index benoemd en niet weggelaten. Het verstrekken van onjuiste of zeer onnauwkeurige informatie over de wettelijke verplichtingen kan zelfstandig als ernstige tekortkoming worden aangemerkt en de handhaving verzwaren; de toepasselijke bepaling en vindplaats staan in het bijgevoegde grondslagenoverzicht van contentpackversie [versie].

**A6 — Bij elke export en elk auditdossier, direct onder de ondertekening:**
> De integriteit van dit dossier is te controleren met de gepubliceerde formaatspecificatie en de bijgevoegde ankerbestanden. De ketenverificatie toont aan dat de inhoud na vastlegging niet ongemerkt is gewijzigd. Zij toont uit zichzelf niet aan op welk moment een record is vastgelegd; die vaststelling berust op de bijgevoegde externe tijdstempels. Ontbreken die, dan berust het tijdstip uitsluitend op de opgave van de organisatie zelf, en is dat in dit dossier als zodanig aangemerkt.

**A7 — Bij ingebruikname van de kluis:**
> Er is geen herstelmogelijkheid via de leverancier. Bij verlies van zowel de wachtwoordzin als de herstelshares is de inhoud van deze kluis definitief onherstelbaar. Vóór het invoeren van productiegegevens moeten zijn afgerond: registratie van twee authenticators, een geslaagde hersteltest, en de inrichting van ten minste één ankerkanaal.

**A8 — Bij elke doorgiftebeoordeling:**
> Een doorgifte-instrument is geen eenmalige vaststelling. De geldigheid van een adequaatheidsbesluit, van modelbepalingen en van aanvullende maatregelen hangt af van de rechtsontwikkeling in het ontvangstland en van besluiten van de Europese Commissie en het Hof van Justitie. Dit instrument is beoordeeld op [datum] op basis van contentpackversie [versie]. Bij een statuswijziging vervalt de status *volledig* van de betrokken verwerkingen en ontstaat per doorgifte een herbeoordelingsverplichting.

**A9 — Bij overgang van een toezichtdossier naar een punitief traject:**
> Vanaf dit punt kan het traject leiden tot een bestuurlijke boete. De medewerkingsplicht en het zwijgrecht bij boeteoplegging bestaan naast elkaar en sluiten elkaar gedeeltelijk uit. Het verdient aanbeveling juridische bijstand in te schakelen vóórdat verder wordt geantwoord. Deze melding kan niet worden uitgezet.

**A10 — Bij elke crosswalkweergave met een niet-gereviewde of verlopen mapping:**
> Deze mapping is niet, of niet actueel, door een mens beoordeeld en telt daarom met gewicht nul mee in de score. Zij mag niet worden gebruikt als onderbouwing dat bewijs uit het ene raamwerk een verplichting uit het andere afdekt.

**A11 — Bij een verstrekking die afwijkt van een eerdere verstrekking aan dezelfde ontvanger:**
> Deze bundel wijkt op [aantal] punten af van de bundel die op [datum] aan dezelfde ontvanger is verstrekt. De verschillen zijn hieronder opgesomd. Verstrekking zonder kennisname van deze lijst is niet mogelijk.

---

## Bijlage B — Beslispunten met een vaste datum

| Beslispunt | Moment | Criterium |
|---|---|---|
| Rust of tweede-keuzestack | **Einde week 3 van fase 0**, na een spike op de drie moeilijkste onderdelen: CTAP2 `hmac-secret`, versleutelde opslag en de hashketen | Werken alle drie de onderdelen aantoonbaar op alle drie de platforms? |
| Opslaglaag | **Einde fase 0** | Envelopeversleuteling per compartiment op een gewone databasemotor, tenzij aantoonbaar is dat versleuteling op databaseniveau daar bovenop iets toevoegt dat een tweede sleutelhiërarchie rechtvaardigt |
| Ankerkanaal als standaard | **Einde fase 0** | Werkt de offline verzoek-en-antwoordstroom naar een gekwalificeerde tijdstempeldienst zonder dat de applicatie zelf een verbinding opent, en is het token door een derde te verifiëren? |
| Handtekening onder de uitvoer | **Einde fase 1** | Is het exportbestand verifieerbaar met standaardgereedschap dat een toezichthouder al heeft, zonder installatie van software van de leverancier? Zo nee, dan een handtekening conform de Europese systematiek op het document, met de losse handtekening als aanvulling en niet als vervanging |
| Documentgenerator | Einde fase 1 | Gepinde versie, eigen ondersteuningslaag, en een schriftelijke inschatting van het onderhoud per jaar. **Byte-gelijkheid vervalt als criterium** |
| Contentpack-abonnement als prijsmodel | **Einde fase −1**, vóór de eerste offerte | Hebben drie organisaties schriftelijk bevestigd dat zij voor het onderhoud van de inhoud willen betalen? |
| Scope: volledige v1 of gehalveerde v1-kern | **Vóór de eerste offerte** | Welke variant past bij de betalingsbereidheid uit fase −1 en bij de doorlooptijd die de eerste klanten accepteren? |
| Crosswalkuitbreiding voorbij de vijf raamwerken van v1 | Na de eerste jaarlijkse hercontrole | Is de hercontrole van de bestaande mappings binnen het begrote budget afgerond? |
| Bouw van het intakepoortje | Start fase 5 | Hebben ten minste drie klanten er expliciet om gevraagd? |
| Doorontwikkeling ketenbewijs | Zes maanden na oplevering van module 15b | Hebben ten minste tien leveranciers van bestaande klanten een bewijspakket geleverd? |
| Eigen redactiepijplijn in plaats van redactieregie | Niet vóór er een tweede ontwikkelaar is | Is er capaciteit om een redactiepijplijn met een eigen testronde te bezitten en te onderhouden? |
| On-premise servervariant | **Vervalt uit de planning** | Wordt pas opnieuw begroot wanneer drie klanten er schriftelijk om vragen én er een tweede ontwikkelaar is |

---

## Bijlage C — Verwerking van de kritische review

| Bevinding | Oordeel | Verwerkt in |
|---|---|---|
| A1 Doorgifteregister en instrumenten | Overgenomen | §7.3 `DOORGIFTE`, `TIA`, `ADEQUAATHEIDSBESLUIT`; I10, I24; T-29, T-35; A8 |
| A2 Gezamenlijke verantwoordelijkheid | Overgenomen | §7.3 `GEZAMENLIJKE_VERANTWOORDELIJKHEID` |
| A3 Meldketen als verwerker | Overgenomen | §7.5 `VERWERKERSMELDING`; I14; T-26, T-31 |
| A4 Kennisgeving aan ontvangers | Overgenomen | §7.3 `ONTVANGERKENNISGEVING`; I18; T-32 |
| A5 Vertragingsmotivering | Overgenomen | §7.5 `DATALEKSPOOR`; I11 |
| A6 Gefaseerde melding | Overgenomen | §7.5 `MELDING.soort = aanvullende_melding_avg` |
| A7 Verplichte inhoud datalekregister | Overgenomen | §7.5 `DATALEKSPOOR` |
| A8 Uitzonderingsgronden art. 34 | Overgenomen | §7.5, gesloten keuzelijst + openbare mededeling |
| A9 Art. 36-klok | Overgenomen | §7.3 `DPIA`; T-27; fase 2 |
| A10 Bericht bij niet-handelen | Overgenomen | §7.3 `BETROKKENENVERZOEK`; T-34 |
| A11 Toestemmingsbewijs | Overgenomen | §7.3 `TOESTEMMING`; I17 |
| A12 Belangenafweging | Overgenomen | §7.3 `BELANGENAFWEGING`; I16 |
| A13 Geautomatiseerde besluitvorming | Overgenomen | §7.3 `GEAUTOMATISEERDE_BESLUITVORMING` |
| A14 Vertegenwoordiger in de Unie | Overgenomen | §7.3 `VERTEGENWOORDIGER_UNIE`; §3.3 |
| A15 Bewaartermijn van het bewijs zelf | Overgenomen | §7.10 |
| B1 BSN | Overgenomen | §7.3 `BSN_GEBRUIK`; I13 |
| B2 Strafrechtelijke gegevens | Overgenomen | §7.3 `VERWERKING` |
| B3 Wpg-spoor | Overgenomen | §7.3 `WPG_SPOOR`; fase 2; §8.3 |
| B4 Woo-spoor | Overgenomen | §7.3 `WOO_VERZOEK`; T-33; fase 2 |
| B5 Bestuursrechtelijke klokken | Overgenomen | §7.7 module 16b; T-28; fase 4 |
| B6 Cautie en medewerkingsplicht | Overgenomen | §7.7; A9 |
| B7 Transparantieplichten algoritmische systemen | Overgenomen | §3.1, §7.3, §8.3 |
| C1 Significante cyberdreiging | Overgenomen | §7.5 `CYBERDREIGING`; T-40; §8.3 |
| C2 Vrijwillige melding | Overgenomen | §7.5 `VRIJWILLIGE_MELDING`; T-39 |
| C3 Initiële registratie | Overgenomen | §7.6 `INITIELE_REGISTRATIE`; fase 1 |
| C4 Scoperesolver met omvangsdrempels | Overgenomen | §3.3 `OMVANGSTOETS`; T-37, T-38 |
| C5 Jurisdictie en hoofdvestiging | Overgenomen | §3.3; §7.2 |
| C6 CSIRT-resolver | Overgenomen | §3.4 `AUTORITEIT`; I22 |
| C7 Raamwerkvariant C | Overgenomen | §3.5, §7.6, §8.2, §8.4; fase 3 |
| C8 Risicobeoordeling als artefact | Overgenomen | §7.6 `RISICOBEOORDELING`; fase 3 |
| C9 Eigen positie als fabrikant | Overgenomen | §10.7; R22 |
| F1 Inwerkingtredingsdatum | Bevestigd, maar uit de code gehaald | §3.2 rechtsfeitencatalogus |
| F2 Autoriteit resolven, niet hard-coderen | Overgenomen | §3.4 |
| F3 Eén meldkanaal onjuist | Overgenomen als correctie | §3.4; §8.3 |
| F4 Sectorale regeling is geen drempelset | Overgenomen als architectuurcorrectie | §3.5, §8.4 |
| F5 Aggregatiedrempel | Overgenomen als correctie | §7.5 `AGGREGATIEGROEP`; T-17, T-24; A2 |
| F6 Termijnrekenkunde te absoluut | Overgenomen als correctie | §10.2 uitgangspunt; A3, A4 |
| F7 Dagen en maanden gemengd; interpretatie hard gecodeerd | Overgenomen als correctie | §10.2 regel 6; T-12, T-13 |
| F8 Registratietermijn inconsistent | Overgenomen als correctie | §7.6; T-10, T-11 |
| F9 Datums in het plan in plaats van in het contentpack | Overgenomen | §3.2 |
| F10 DORA-uitsluiting te absoluut | Overgenomen als correctie | §3.6 |
| F11 Normaanduidingen inconsistent | Overgenomen | §7.6 `deelaanduiding`; §8.2 |
| F12 Onverifieerbare vindplaats in een vaste tekst | Overgenomen | A5 herschreven; alle vindplaatsen naar het contentpack; R25 |
| F13 Kennisnamebegrip te stellig voor de AVG | Overgenomen als nuancering | §3.7; A3 |
| F14 Adequaatheidsbesluiten ontbreken | Overgenomen | §7.3 `ADEQUAATHEIDSBESLUIT`; §9.10 |
| F15 "Geen overgangsrecht" onvoldoende onderbouwd | Overgenomen als nuancering | R17 |
| S1 Hashketen bewijst geen antedatering | Overgenomen — kernwijziging | §6.4; R24; A6 |
| S2 Noodpad met kortcode | Overgenomen — geschrapt | §6.2, §6.5 |
| S3 Compartimenten niet cryptografisch | Overgenomen | §6.3; I7 |
| S4 Schil als zachtste plek | Overgenomen | §6.5, §5.2; R13 opgehoogd naar hoog |
| S5 Geen sleutelrotatie | Overgenomen | §6.2; I27 |
| S6 Shamir zonder integriteitsbescherming | Overgenomen | §6.2 |
| S7 Parameters ongeauthenticeerd | Overgenomen | §6.2; §10.1 downgradetests |
| S8 Eén authenticator, leesbare header | Overgenomen | §6.2; I19 |
| S9 Terugvalpad zonder beleid | Overgenomen — pad geschrapt | §6.2 |
| S10 Geen sandboxspecificatie | Overgenomen | §6.6; §10.1 |
| S11 Geheugenveiligheid stopt bij de grens | Overgenomen | §5.3 |
| S12 Wisselbestand, sluimeren, momentopnamen | Overgenomen | §6.7 |
| S13 Back-upoppervlak breder | Overgenomen | §6.7 |
| S14 Klembord | Overgenomen | §6.5 |
| S15 Export onversleuteld | Overgenomen | §6.5; I21 |
| S16 Geïmporteerde inhoud als opmaak | Overgenomen | §6.5; §10.1 |
| S17 Redactie zonder verificatie | Overgenomen | §9.5.1; I28 |
| S18 Uitchecken vertakt de keten | Overgenomen | §6.4 punt 5; fase 5 |
| S19 Geen updatekanaal én geen telemetrie | Overgenomen | R23 |
| S20 Shares zonder levenscyclus | Overgenomen | §6.2 |
| S21 Term "WORM" | Overgenomen | §6.4 punt 4; §7.5 |
| K1 Redactiewerkbank | Overgenomen — vervangen door redactieregie | §9.5.1; §12.3 |
| K2 Servervariant | Overgenomen — vervalt | §9.8; §12.5; bijlage B |
| K3 Twee extra sleutelpaden | Overgenomen — geschrapt | §6.2 |
| K4 Tien raamwerken | Overgenomen — v1 beperkt tot vijf | §8.5 |
| K5 Byte-identieke builds | Overgenomen — versoberd | §10.4; bijlage B |
| K6 Bitemporaliteit op alles | Overgenomen — beperkt tot vijf entiteiten | §7.1 |
| K7 Campagnebeheer | Overgenomen — geschrapt | §9.7; §12.3 |
| K8 Drie driftimporters | Overgenomen — één generieke veldmappingtool | §9.5 |
| K9 Uitvraagsimulatie | Overgenomen — geschrapt; capaciteit naar de tegenspraakbibliotheek | §9.7 |
| K10 Benchmarkkengetallen | Overgenomen — geschrapt | §9.8; §12.5 |
| K11 Volledige toegankelijkheidsaudit in één week | Overgenomen — bouwvoorschrift vanaf dag één | §9.7; §10.1 |
| K12 Intakepoortje in de begroting | Overgenomen — uit de basisplanning | §9.8; bijlage B |
| O1 Tijdsverankering met open bewijsformaat | Overgenomen | §6.4; fase 0 |
| O2 Bestuursrechtelijke procesmodule | Overgenomen | §7.7 module 16b; fase 4 |
| O3 Aantoonbaarheidsradar / vervalprognose | Overgenomen | §7.8; fase 3 |
| O4 Ketenbewijs | Overgenomen | Module 15b; fase 4; R27 |
| O5 Persoonlijk onafhankelijkheidsdossier | Overgenomen | §7.4; R26 |
| Tegenspraakbibliotheek | Overgenomen als contentpack ná v1 | §9.10, §9.7 |
| P1 Schatting factor 2 à 3 mis | Overgenomen | §9.1, §9.9 |
| P2 Stackbeslispunt te laat | Overgenomen | §9.3; bijlage B |
| P3 Opslaglaag overbodig naast compartimentversleuteling | Overgenomen | §5.2; bijlage B; R14 verlaagd |
| P4 Engines lopen achter | Overgenomen | §5.2, §6.5; R13 opgehoogd |
| P5 Verkeerd probleem bij documentgeneratie | Overgenomen | §5.2, §10.5; bijlage B |
| P6 Faseringsconflict | Overgenomen | §9.4, §9.11 punt 8 |
| P7 Inhoud na de machine | Overgenomen | Fase −1 (§9.2) |

**Niet overgenomen of afwijkend verwerkt:** geen. Twee bevindingen zijn genuanceerd in plaats van integraal overgenomen: F15 (de stelling over overgangsrecht is niet vervangen door een tegenovergestelde stelling maar uit de productteksten gehaald en naar contentpack met verificatiestempel verplaatst) en F12 (de onjuiste vindplaats is verwijderd; de strekking blijft, maar de vindplaats komt uit het contentpack en wordt vóór de eerste levering juridisch geverifieerd).

---

*Einde document. Opgesteld door en namens WimLee115, 18 augustus 2026. Herziening 2.0, vervangt herziening 1.0 van dezelfde datum. Wijzigingen op dit ontwerp lopen via een genummerde herziening met vermelding van datum en reden; de vorige versie blijft bewaard. Alle in dit document genoemde wettelijke datums, termijnen, drempels, artikelverwijzingen en autoriteiten zijn contentpackinhoud en dragen een verificatiestempel; waar dit document ze noemt, doet het dat als huidige contentpackwaarde en niet als vaststelling.*