# Kritische review projectplan `dpo-fg-tool`

**Reviewer:** privacyjurist/FG (achtergrond toezicht) en security-architect
**Peildatum toetsing:** 18 augustus 2026 — drie dagen na inwerkingtreding van de Cyberbeveiligingswet en het Cyberbeveiligingsbesluit

---

## Kernoordeel

Het plan is inhoudelijk verder dan vrijwel alles wat op de Nederlandse markt te koop is, en §12 (wat de tool níet doet) is het sterkste hoofdstuk. Maar er zitten drie categorieën problemen in die het product in zijn huidige vorm onverkoopbaar of zelfs schadelijk maken:

1. **De AVG-kant is aantoonbaar dunner dan de NIS2-kant.** Doorgiften, gezamenlijke verwerkingsverantwoordelijkheid, art. 19, art. 22, BSN, Wpg, toestemmingsbewijs en de verwerkersmeldketen ontbreken volledig. Voor een product dat "FG-tool" heet is dat een gat in het hart.
2. **De integriteitsclaim is circulair.** Een hashketen in een bestand dat de gebruiker zelf beheert, geverifieerd door een verifier van dezelfde leverancier, bewijst tegenover een toezichthouder niets over antedatering. Dat is exact de claim waarop het hele product rust.
3. **De planning is met een factor 2 à 3 te optimistisch** en de faseringsvolgorde bevat minstens één harde tegenstrijdigheid (inzageverzoeken vóór redactie).

Hieronder per vraag de bevindingen en de letterlijk in te voegen tekst.

---

# 1. Ontbrekende wettelijke verplichtingen, registers, termijnen en artefacten

## 1.1 AVG — ontbrekend

| # | Ontbreekt | Grondslag | Waarom fataal |
|---|---|---|---|
| A1 | **Doorgifteregister en doorgifte-instrumenten**: geen entiteit voor adequaatheidsbesluit, modelbepalingen, BCR, art. 49-uitzondering, TIA | art. 44-49, art. 46 lid 2 onder c/d, art. 47, art. 49 lid 1 en lid 6, art. 30 lid 1 onder e | De meest volatiele juridische inhoud die er is, en er is geen datamodel voor. Zie §2 voor de actuele status |
| A2 | **Register van gezamenlijke verwerkingsverantwoordelijkheid** | art. 26 lid 1 en 2 | Elke gemeentelijke ketensamenwerking is art. 26; de "wezenlijke inhoud" moet beschikbaar zijn voor de betrokkene |
| A3 | **Meldketen als verwérker**: geen klok voor "melden aan de verwerkingsverantwoordelijke" en geen inbox voor meldingen ván verwerkers | art. 33 lid 2, art. 28 lid 3 onder f | Contractuele termijnen van 24/48 uur zijn de praktijk; de tool kent alleen de klok naar de AP |
| A4 | **Kennisgeving aan ontvangers** bij rectificatie, wissing of beperking + mededeling wélke ontvangers | art. 19 | Klassiek vergeten verplichting, wordt in handhaving standaard uitgevraagd |
| A5 | **Vertragingsmotivering bij te late datalekmelding** | art. 33 lid 1, tweede volzin | Ontbreekt als veld in `DATALEKSPOOR`; zonder dit veld kan de tool een te late melding niet correct produceren |
| A6 | **Gefaseerde melding** | art. 33 lid 4 | Geen `MELDING`-soort voor aanvulling op een eerdere AP-melding |
| A7 | **Verplichte inhoud datalekregister**: waarschijnlijke gevolgen en genomen/voorgestelde maatregelen | art. 33 lid 3 onder c en d, art. 33 lid 5 | `DATALEKSPOOR` heeft wel oordeelvelden maar niet de wettelijk voorgeschreven documentatievelden |
| A8 | **Uitzonderingsgrond art. 34 lid 3** en de openbare mededeling ex art. 34 lid 3 onder c; plus de bevoegdheid van de AP om alsnog te verplichten (art. 34 lid 4) | art. 34 lid 3-4 | `art34_oordeel` is een vrij veld zonder de drie limitatieve gronden |
| A9 | **Art. 36-klok**: 8 weken + verlenging 6 weken + opschorting | art. 36 lid 2 en 3 | Module 14 noemt art. 36 maar de termijn staat niet in §10.2 |
| A10 | **Art. 12 lid 4**: bericht binnen één maand bij niet-handelen, inclusief klachtrecht AP en rechtsmiddel | art. 12 lid 4 | Weigering is óók een handeling met termijn |
| A11 | **Toestemmingsbewijs en intrekking** | art. 7 lid 1 en 3, art. 8 | Geen artefact, terwijl de bewijslast expliciet bij de verantwoordelijke ligt |
| A12 | **Belangenafweging (LIA)** bij art. 6 lid 1 onder f | art. 6 lid 1 onder f, overweging 47 | Ontbreekt terwijl dit ná HvJ EU 4 oktober 2024, C-621/22 (KNLTB) juist het meest betwiste onderwerp is |
| A13 | **Register geautomatiseerde besluitvorming en profilering** met beschrijving van de onderliggende logica | art. 13 lid 2 onder f, art. 14 lid 2 onder g, art. 15 lid 1 onder h, art. 22, UAVG art. 40 | Voor overheidsklanten de meest gestelde uitvraag |
| A14 | **Vertegenwoordiger in de Unie** | art. 27 | Relevant zodra een klant een niet-EU-verwerker inschakelt die zelf verantwoordelijke is |
| A15 | **Bewaartermijn van het bewijs zelf** | art. 5 lid 2 juncto Awb art. 5:45 (verjaring bestuurlijke boete: vijf jaar) | De tool bewaakt bewaartermijnen van anderen maar heeft er zelf geen |

## 1.2 UAVG en aanpalend nationaal recht — ontbrekend

| # | Ontbreekt | Grondslag |
|---|---|---|
| B1 | **BSN-vlag op `VERWERKING`** met verplichte wettelijke grondslag voor het gebruik | art. 87 AVG, UAVG art. 46, Wet algemene bepalingen burgerservicenummer |
| B2 | **Strafrechtelijke gegevens** als eigen categorie, apart van bijzondere gegevens, met de UAVG-uitzonderingen | art. 10 AVG, UAVG art. 31-33 |
| B3 | **Wpg-spoor voor gemeenten met boa's**: interne controle en de vierjaarlijkse externe privacyaudit, met eigen verplichtingen en auditrapport als bewijsstuk | Wpg art. 33, Besluit politiegegevens |
| B4 | **Woo-spoor**: beslistermijn vier weken met eenmalige verdaging van twee weken, plus het onderscheid tussen een Woo-verzoek en een inzageverzoek | Woo art. 4.4, art. 5.1 lid 2 onder e |
| B5 | **Bestuursrechtelijke klokken** rond het `TOEZICHTDOSSIER`: zienswijzetermijn, bezwaartermijn zes weken, beroepstermijn zes weken | Awb art. 4:8, art. 6:7, art. 6:8 |
| B6 | **Cautie en medewerkingsplicht** als expliciet signaleringspunt zodra toezicht overgaat in punitieve handhaving | Awb art. 5:10a tegenover art. 5:20 |
| B7 | **AI-verordening en algoritmeregister**: transparantieplichten zijn sinds 2 augustus 2026 van toepassing | Verordening (EU) 2024/1689 art. 50, zoals gewijzigd bij Verordening (EU) 2026/1744 |

## 1.3 Cbw/NIS2 — ontbrekend

| # | Ontbreekt | Grondslag |
|---|---|---|
| C1 | **Significante cyberdreiging** als zelfstandig object met eigen informatieplicht richting afnemers, inclusief de te treffen maatregelen | NIS2 art. 23 lid 2; Cbw-pendant |
| C2 | **Vrijwillige melding** van bijna-incidenten, dreigingen en kwetsbaarheden | NIS2 art. 30 |
| C3 | **Initiële registratie** als verplichting met eigen klok; het plan modelleert alleen wijzigingen | Cbw registratieplicht per 15-08-2026 |
| C4 | **Scoperesolver met KMO-drempels**: het tweejaarsvereiste en de consolidatie van partner- en verbonden ondernemingen | Aanbeveling 2003/361/EG, bijlage art. 4 lid 2 en art. 3; NIS2 art. 2 |
| C5 | **Jurisdictiebepaling en hoofdvestiging** bij grensoverschrijdende entiteiten, plus vertegenwoordiger in de Unie | NIS2 art. 26 lid 1 onder b en lid 3 |
| C6 | **CSIRT-resolver.** De tool hard-codeert MijnNCSC; de CSIRT verschilt per sector | Zie §2, bevinding F3 |
| C7 | **Raamwerkvariant C**: een sectoraal voorgeschreven normenkader. De Cyberbeveiligingsregeling overheid schrijft BIO2 dwingend voor per 15-08-2026 | Zie §2, bevinding F4 |
| C8 | **Risicobeoordeling als artefact** met methode, scope, uitvoerder, datum en bestuursvaststelling; de tool kent `RISICO` maar niet de beoordeling zelf | Cbw art. 21, Cbb-zorgplicht |
| C9 | **Eigen positie als fabrikant onder de CRA.** De meldplicht van art. 14 CRA geldt vanaf 11-09-2026 — over 24 dagen | Verordening (EU) 2024/2847 |

### Invoegtekst — nieuw §7.3a

> **DOORGIFTE**
> `id, verwerking_id, ontvanger, ontvangerland, rol_ontvanger (verwerker|verantwoordelijke|gezamenlijk), instrument (adequaatheidsbesluit|modelbepalingen|bcr|gedragscode|certificering|art49_uitzondering|geen), instrument_ref, instrument_versie, geldig_van, geldig_tot, herbeoordelingsdatum, tia_id, aanvullende_maatregelen[], art49_grond, art49_vastlegging_in_register (verplicht bij art49)`
> Relaties: 1—0..1 `TIA (datum, uitvoerder, rechtsontwikkelingen_geraadpleegd_op, uitkomst, restrisico, besluit_door)`; 1—N `VERPLICHTING` (herbeoordeling bij wijziging van het adequaatheidsbesluit of van het recht van het ontvangstland).
>
> **GEZAMENLIJKE_VERANTWOORDELIJKHEID** (art. 26)
> `id, verwerking_id, partijen[], regeling_bewijsstuk_id, verdeling_verplichtingen_json, contactpunt, wezenlijke_inhoud_publicatie_ref, vastgesteld_op`
>
> **GEAUTOMATISEERDE_BESLUITVORMING** (art. 22, UAVG art. 40)
> `id, verwerking_id, is_uitsluitend_geautomatiseerd, rechtsgevolg_of_aanmerkelijke_treffing, grondslag_art22_lid2 (a|b|c), onderliggende_logica_omschrijving, belang_en_gevolgen, menselijke_tussenkomst_procedure, betwistingsprocedure, algoritmeregister_ref, ai_verordening_classificatie (verboden|hoogrisico|transparantieplicht|geen), ai_art50_maatregelen`
>
> **BSN_GEBRUIK**
> `id, verwerking_id, wettelijke_grondslag_ref, doel, toets_uitgevoerd_door, toets_datum`

### Invoegtekst — uitbreiding §7.4 `DATALEKSPOOR`

> Toe te voegen velden (alle verplicht bij afsluiting):
> `waarschijnlijke_gevolgen`, `genomen_of_voorgestelde_maatregelen`, `categorieen_betrokkenen[]`, `categorieen_persoonsgegevens[]`, `aantal_registraties`, `vertraging_bij_melding (ja|nee)`, `vertraging_motivering (verplicht indien vertraging = ja, AVG art. 33 lid 1 tweede volzin)`, `art34_uitzonderingsgrond (lid3_a_versleuteling|lid3_b_latere_maatregelen|lid3_c_onevenredige_inspanning|geen)`, `openbare_mededeling_ref (verplicht bij lid3_c)`, `ap_verplichting_alsnog_informeren (art. 34 lid 4)`, `rol_entiteit (verantwoordelijke|verwerker)`, `melding_aan_verantwoordelijken[] (bij rol = verwerker)`.
>
> Nieuwe subentiteit **VERWERKERSMELDING**: `id, incident_id, richting (ontvangen|verzonden), tegenpartij_id, contractuele_termijn_uren, ontvangen_op, verzonden_op, bewijsstuk_id`. De contractuele termijn wordt overgenomen uit `VERWERKERSOVEREENKOMST.art28_lid3_checklist_json` onderdeel f en levert een eigen `VERPLICHTING` op.

### Invoegtekst — nieuwe invarianten bij §7.7

| # | Invariant |
|---|---|
| I10 | Een `VERWERKING` met een `DOORGIFTE` naar een derde land kan niet de status *volledig* bereiken zonder een geldig `instrument` waarvan `geldig_tot` in de toekomst ligt, dan wel een vastgelegde `art49_grond` met vastlegging in het register. |
| I11 | Een `DATALEKSPOOR` met `meldplicht_oordeel = melden` en een verzendtijdstip later dan 72 uur na `tijdstip_kennisname` kan niet worden afgesloten zonder `vertraging_motivering`. |
| I12 | Een `CONTROL` waarvan de `eigenaar_id` de aangemelde functionaris voor gegevensbescherming is, levert een blokkerende bevinding op wegens strijd met AVG art. 38 lid 6; de FG kan geen eigenaar zijn van een maatregel waarop hij toezicht houdt. |
| I13 | Een `VERWERKING` met `bsn_gebruik` zonder `wettelijke_grondslag_ref` levert een blokkerende hygiënebevinding op (UAVG art. 46). |
| I14 | Een `INCIDENT` waarbij de entiteit optreedt als verwerker kan niet worden afgesloten zonder ten minste één `VERWERKERSMELDING` met richting *verzonden* of een vastgelegde motivering waarom die achterwege bleef. |
| I15 | Een `NIS2SPOOR` met `eindoordeel = significant` kan niet worden afgesloten zonder een expliciet oordeel over de informatieplicht richting afnemers, inclusief motivering bij niet-informeren. |

### Invoegtekst — extra randgevallen bij §10.2

| # | Casus | Verwacht gedrag |
|---|---|---|
| T-21 | Inzageverzoek ontvangen 15 januari; termijn van één maand | Deadline 15 februari; valt die op zaterdag, zondag of algemeen erkende feestdag, dan de eerstvolgende werkdag (Verordening (EEG, Euratom) nr. 1182/71, art. 3 lid 4). Termijnen in maanden worden nooit intern in dagen omgerekend |
| T-22 | Inzageverzoek ontvangen 31 januari | Deadline 28 februari, respectievelijk 29 februari in een schrikkeljaar (art. 3 lid 2 laatste alinea Verordening 1182/71) |
| T-23 | Registratiewijziging waarvan de tweewekentermijn eindigt op tweede paasdag | Verlengd tot de eerstvolgende werkdag op grond van de Algemene termijnenwet; de 24- en 72-uurstermijnen worden níet verlengd |
| T-24 | Twee incidenten, dezelfde grondoorzaak, binnen zes maanden, entiteit met een jaaromzet van 4 miljoen euro, schade tweemaal 120.000 euro | Drempel is 5 procent van de omzet (200.000 euro), niet 500.000 euro; aggregatiegroep bereikt de drempel en slaat alarm |
| T-25 | Vertrouwensdienstverlener, kennisname 10:00 | Vroegtijdige waarschuwing én melding vallen samen op 24 uur; de tool toont één klok met twee verplichtingen en waarschuwt tegen dubbeltelling |
| T-26 | Datalek bij een verwerker, contractueel 24 uur | Klok naar de verwerkingsverantwoordelijke start bij kennisname van de verwerker, onafhankelijk van de 72-uursklok van de verantwoordelijke |
| T-27 | Art. 36-raadpleging ingediend, AP vraagt aanvullende informatie op dag 20 | Termijn van 8 weken opgeschort tot ontvangst; verlenging met 6 weken apart zichtbaar |
| T-28 | Besluit van de toezichthouder bekendgemaakt op 3 september | Bezwaartermijn zes weken vanaf de dag ná bekendmaking (Awb art. 6:7 en 6:8); harde, niet-verlengbare klok met eigen escalatie |
| T-29 | Adequaatheidsbesluit voor het ontvangstland wordt ingetrokken | Alle `DOORGIFTE`-records met dat instrument gaan naar *ongeldig*; per record ontstaat een herbeoordelingsverplichting met termijn |
| T-30 | Bestuurslid benoemd op 20 augustus 2026, dus ná inwerkingtreding | Individuele deadline 20 augustus 2028; de collectieve backstop 15-08-2028 is dan de eerdere van de twee en wint |

### Invoegtekst — extra regels in het delta-rapport (§8.3)

| Verplichting | Grondslag | Waarom niet gedekt |
|---|---|---|
| Doorgiftebeoordeling en instrumentbeheer met vervaldatum | AVG art. 44-49 | ISO en NEN kennen geen juridisch doorgifteregime |
| Kennisgeving aan ontvangers bij rectificatie of wissing | AVG art. 19 | Geen equivalent in Annex A |
| Melding aan de verwerkingsverantwoordelijke door de verwerker | AVG art. 33 lid 2 | ISO kent incidentbeheer, niet deze contractuele meldketen |
| Privacyaudit politiegegevens | Wpg art. 33, Besluit politiegegevens | Volledig buiten het ISMS |
| Bestuursrechtelijke rechtsbescherming: zienswijze, bezwaar, beroep | Awb art. 4:8, 6:7, 6:8 | Procesrecht, geen normenkader |
| Transparantieplichten AI-verordening | Verordening (EU) 2024/1689 art. 50 | Ander regime, andere toezichthouder |
| Informatieplicht bij significante cyberdreiging | NIS2 art. 23 lid 2 | Geen normequivalent |

---

# 2. Feitelijke fouten en onnauwkeurigheden

| # | Bevinding | Oordeel |
|---|---|---|
| F1 | **De datum 15 augustus 2026 klopt.** Cbw en Cbb zijn per die datum in werking; de Cbw vervangt de Wbni. De backstop van 15-08-2028 voor bestuurdersopleiding volgt daaruit | Correct — maar zie F9 |
| F2 | **"Auditdossier voor de RDI" bij een gemeente klopt.** De RDI is bevoegde autoriteit voor de sector overheid; voor waterschappen is dat de ILT | Correct, maar de tool moet de autoriteit resolven, niet hard-coderen |
| F3 | **MijnNCSC als enig meldkanaal is onjuist.** Het NCSC is nationaal CSIRT en beheert het entiteitenregister, maar Z-CERT is het aangewezen CSIRT voor de zorg; voor gemeenten en gemeenschappelijke regelingen is de IBD het sectorale CSIRT, waarbij het NCSC voorlopig CSIRT-dienstverlening levert. De zorgklant die via MijnNCSC meldt, meldt bij het verkeerde CSIRT | **Fout** |
| F4 | **Sectorale ministeriële regelingen zijn géén drempelset.** §8.4 modelleert ze uitsluitend als "drempelset op het `NIS2SPOOR`". De Cyberbeveiligingsregeling overheid schrijft BIO2 als normenkader dwingend voor aan overheidsinstanties per 15-08-2026 (BIO2 v1.3, Stcrt. 5 maart 2026). Dat raakt het raamwerkvariantmodel, niet de significantietoets | **Architectuurfout** |
| F5 | **De aggregatiedrempel is niet 500.000 euro.** Uitvoeringsverordening (EU) 2024/2690 art. 3 lid 1 onder a: meer dan 500.000 euro **of meer dan 5 procent van de totale jaaromzet in het voorgaande boekjaar, indien dat lager is**. T-17 hard-codeert het absolute bedrag en zou bij een mkb-entiteit een meldplichtig incident missen. Art. 4 vereist bovendien ten minste tweemaal binnen zes maanden met dezelfde kennelijke oorzaak | **Fout, met meldplichtgevolg** |
| F6 | **"Kalendertijd, geen werkdagcorrectie" is te absoluut.** Verordening (EEG, Euratom) nr. 1182/71 art. 3 lid 4 verlengt termijnen die níet in uren zijn uitgedrukt tot de eerstvolgende werkdag als de laatste dag een zaterdag, zondag of feestdag is. Dat geldt dus wél voor de maandtermijn van AVG art. 12 lid 3 en voor het eindverslag van één maand, en níet voor 24 en 72 uur. Voor nationale termijnen in dagen, weken en maanden werkt daarnaast de Algemene termijnenwet door | **Fout in de kern van het product** |
| F7 | **T-12 en T-13 mengen dagen en maanden.** "Verlenging aangevraagd op dag 32" en "resterende termijn 18 dagen" zijn rekenkundig onjuiste vertalingen van een maandtermijn. Bovendien is de opschortingsleer bij identiteitsverificatie een interpretatie (EDPB Guidelines 01/2022) en geen wettekst; die mag niet hard in de motor zitten | **Fout plus onterechte zekerheid** |
| F8 | **ENISA-termijn inconsistent.** §8.3 noemt "1 maand / 3 maanden", §7.5 noemt "14 dagen respectievelijk 3 maanden". De nationale wijzigingstermijn is twee weken. De 1-maandstermijn wordt nergens onderbouwd | **Interne inconsistentie** |
| F9 | **Datums staan in het plan, niet in het contentpack.** 15-08-2026, 15-08-2028, 11-09-2026 en 11-12-2027 staan verspreid in tekst, tests en risicotabel, terwijl R1 zegt dat alle juridische inhoud buiten de binary blijft. Eén verschoven datum betekent nu een release | **Ontwerpinconsistentie** |
| F10 | **DORA-uitsluiting te absoluut.** NIS2 art. 4 sluit de bepalingen over risicobeheer, melding én het bijbehorende toezicht uit voor zover sectorspecifieke regels ten minste gelijkwaardig zijn. De identificatie- en registratiesystematiek blijft van toepassing. "De Cbw-verplichting vervalt" is dus onjuist als algemene regel | **Fout** |
| F11 | **Normaanduidingen inconsistent.** In §8.2 "NEN 7510-2:2024", in bijlage A "NEN 7510:2024". Deel 1 en deel 2 zijn verschillende documenten met verschillende eisen. Idem CIS Controls v8 tegenover v8.1 | **Slordigheid met bewijsgevolg** |
| F12 | **De vaste waarschuwingstekst citeert NIS2 art. 32 lid 7 onder a.** Die vindplaats is niet verifieerbaar met de strekking die het plan eraan geeft. Een niet-uitschakelbare waarschuwing met een onjuiste vindplaats is zelf een geloofwaardigheidsrisico in een auditdossier | **Te verifiëren, waarschijnlijk fout** |
| F13 | **"Kennisname, niet de bevestigde diagnose" is voor de AVG te stellig.** Kennisname veronderstelt een redelijke mate van zekerheid dat een inbreuk heeft plaatsgevonden; een korte eerste verificatie is toegestaan en moet zelf worden gedocumenteerd. Voor NIS2 klopt de formulering wel | **Nuancefout** |
| F14 | **Adequaatheidsbesluiten ontbreken volledig**, terwijl juist daar de status het snelst verandert: het Britse besluit is op 19 december 2025 vernieuwd voor zes jaar tot 27 december 2031; het EU-VS Data Privacy Framework staat sinds september 2025 overeind na de uitspraak van het Gerecht, maar het toezichtsorgaan PCLOB is sinds januari 2025 zonder quorum en de herautorisatie van Section 702 FISA is in april 2026 verlopen. Dit is precies het type feit dat versiebeheerde inhoud vereist | **Materieel gat** |
| F15 | **"Het Cbb kent geen overgangsrecht" is een harde juridische stelling zonder vindplaats.** Ook zonder overgangsrecht sturen evenredigheid en de beginselen van behoorlijk bestuur de handhaving | **Onvoldoende onderbouwd** |

### Invoegtekst — vervangt de eerste regels van §10.2

> **Termijnrekenkundig uitgangspunt.** De motor rekent termijnen in uren, dagen, weken, maanden en jaren als afzonderlijke typen en zet een maandtermijn nooit om in dagen.
>
> 1. Termijnen uitgedrukt in **uren** (24 en 72 uur) lopen in kalendertijd door weekenden en feestdagen heen en worden nooit verlengd. Grondslag: Verordening (EEG, Euratom) nr. 1182/71, art. 3 lid 1 en 2.
> 2. Termijnen uitgedrukt in **dagen, weken, maanden of jaren** die voortvloeien uit Unierecht en eindigen op een zaterdag, zondag of feestdag, lopen af aan het einde van de eerstvolgende werkdag. Grondslag: Verordening 1182/71, art. 3 lid 4.
> 3. Termijnen uit **nationale wetgeving en algemeen verbindende voorschriften** volgen de Algemene termijnenwet, tenzij de betreffende regeling anders bepaalt.
> 4. De **feestdagenkalender** is contentpackinhoud met een jurisdictie- en jaaraanduiding, geen code.
> 5. Bij elke berekende deadline toont de interface welke van deze regels is toegepast en op welke bepaling zij berust. Bij een omstreden interpretatie — zoals opschorting bij identiteitsverificatie — biedt de tool beide lezingen aan, met bronvermelding, en legt de gekozen lezing met motivering vast in het dossier.

### Invoegtekst — vervangt de regel over sectorale regelingen in §8.4

> | **Sectorale ministeriële regelingen** (Cyberbeveiligingsregeling overheid, Regeling cyberbeveiliging EZ, Cyberbeveiligingsregeling zorg) | Kunnen drie dingen tegelijk doen en worden daarom als drie afzonderlijke koppelingen gemodelleerd: (1) een **voorgeschreven normenkader** — variant C, bijvoorbeeld BIO2 v1.3 voor overheidsinstanties per 15-08-2026; (2) een **drempelset** op het `NIS2SPOOR`; (3) **aanvullende verplichtingen**. Elke regeling krijgt een eigen `NORMBEPALING`-set met versie en consolidatiedatum, met de open norm van Cbw art. 25 lid 2 altijd als vangnet erboven |

### Invoegtekst — nieuwe entiteit bij §7.6

> **AUTORITEIT**
> `id, soort (bevoegde_autoriteit|csirt|gegevensbeschermingsautoriteit), naam, sector[], entiteitstypen[], meldkanaal, kanaal_ref, geldig_van, geldig_tot, bron`
> Nergens in de code staat een naam van een toezichthouder, CSIRT of meldportaal. De resolver bepaalt op grond van sector, entiteitstype en peildatum welke autoriteit en welk CSIRT van toepassing zijn, en toont dat expliciet in de meldcockpit: *"Uw CSIRT is X; uw bevoegde autoriteit is Y; beide moeten worden geïnformeerd."* Wisselingen in aanwijzing zijn contentpackwijzigingen met een ingangsdatum, geen releases.

---

# 3. Beveiligingsgaten in de architectuur

## 3.1 De vier zwaarste

**S1 — De hashketen bewijst interne consistentie, geen antedatering.**
De dreiging die ertoe doet is niet de dief met het bestand, maar de insider met volledige toegang: de FG die achteraf wil laten zien dat hij op vrijdag 16:40 kennis nam in plaats van op woensdag. Die persoon bezit de wachtwoordzin, de token en het bestand, en kan de keten afkappen en opnieuw opbouwen. "Merkle-ankers" lossen dit alleen op als er buiten de kluis wordt verankerd, en het plan zegt niet waarin. Zonder extern anker is `dpofg-verify` een cirkelredenering. Bovendien loopt elke klok op de systeemklok van diezelfde gebruiker.

**S2 — Het TOTP-noodpad is een achterdeur of schijnveiligheid.**
Een TOTP-code van zes cijfers kan geen betekenisvolle entropie aan een sleutelafleiding bijdragen, en verificatie vereist opslag van het gedeelde geheim. Als dit pad een kluis kan openen, is de kluis te openen met een geheim van lage entropie plus dat wat naast de TOTP nodig is. Als het dat niet kan, is het decoratie. Dit staat bovendien haaks op R3 en op de belofte "er is geen achterdeur en er komt er geen".

**S3 — Compartimenten zijn niet cryptografisch afgedwongen zolang er één databasesleutel is.**
`COMPARTIMENT.sleutelwrap` suggereert eigen sleutels, maar SQLCipher versleutelt op paginaniveau met één sleutel. Wie de kluis kan openen, kan met de sqlite3-shell alles lezen; I7 is dan een applicatieregel, geen garantie. De zoekindex is de scherpste rand: een FTS-index over gecompartimenteerde inhoud lekt tokens, en WAL-, journal- en tijdelijke bestanden lekken mee.

**S4 — De WebView is de zachtste plek van een hard product.**
Alle ontsleutelde bijzondere persoonsgegevens worden gerenderd in de browser-engine van het besturingssysteem. WebView2 schrijft caches en crashdumps naar schijf en kan die naar buiten sturen; spellingcontrole en SmartScreen zijn eveneens netwerkgedrag. WebKitGTK op Ubuntu LTS loopt structureel achter op beveiligingsupdates. Zonder expliciete uitschakeling is de claim "geen telemetrie, geen crashrapportage naar buiten" feitelijk onjuist.

## 3.2 De overige bevindingen

| # | Gat | Concreet gevolg |
|---|---|---|
| S5 | Geen sleutelrotatie. Een vertrekkende FG heeft de dataversleutelingssleutel gezien; de kluis blijft met die sleutel werken | Offboarding is onmogelijk zonder volledige herversleuteling |
| S6 | Shamir zonder integriteitsbescherming. Klassieke Shamir over GF(256) is kneedbaar: onjuiste of gemanipuleerde shares leveren stilzwijgend een verkeerd geheim | Herstel faalt onopgemerkt; gebruik SLIP-0039 of voeg per-share- en totaal-authenticatie toe |
| S7 | Argon2id-parameters in de header, ongeauthenticeerd vóór gebruik | Downgrade-aanval: parameters op minimum zetten en offline brute-forcen. Nodig: harde ondergrens in code, parameters als AAD gebonden aan de afleiding |
| S8 | Geen tweede geregistreerde CTAP2-token verplicht; credential-id staat in de leesbare header | Verlies van één token vergt herstel; metadatalek over de gebruikte authenticator |
| S9 | PIV-terugval zonder verplichte pin-always- en touch-policy | Malware op de host kan stil unwrappen |
| S10 | Geen sandboxspecificatie per platform. "Gescheiden proces" is geen sandbox | Nodig: seccomp plus namespaces op Linux, App Sandbox op macOS, AppContainer met job object op Windows, in alle gevallen zonder netwerkcapability |
| S11 | Geheugenveiligheid stopt bij de FFI-grens. SQLCipher is C, OCR en beeldcodecs zijn C++ | De Rust-claim dekt juist de componenten niet waar de bugs zitten |
| S12 | Swap, hibernatie, crashdumps en Volume Shadow Copies niet geadresseerd | mlock is op Windows quotabeperkt; hiberfil.sys bevat de sleutel |
| S13 | Back-upoppervlak breder dan netwerkpaden: OneDrive Known Folder Move, Windows Search-index, Time Machine, automatische sample-inzending van antivirus | De kluis kan als "verdacht bestand" naar een leverancier worden geüpload |
| S14 | Klembordstroom als kernontwerp, zonder wissen en zonder waarschuwing voor cloudklembord | Meldingsteksten met bijzondere gegevens synchroniseren naar een cloudaccount |
| S15 | Exportbundels standaard onversleuteld | De rijkste dataset van de organisatie, inclusief ip-bereiken en kwetsbaarhedenbeeld, belandt in Verzonden items |
| S16 | XSS-oppervlak via geïmporteerde inhoud: e-mailbodies, bestandsnamen, leveranciersnamen | Eén `{@html}` of één te ruime CSP maakt van XSS volledige lokale bestandstoegang via de Tauri-brug |
| S17 | Redactie zonder verificatiestap | Tekstlaag onder een zwart vlak, XMP-metadata, annotaties. Dit is R11 die zich realiseert |
| S18 | Het uitcheckmodel vertakt de hashketen | Twee gebruikers, twee ketens, geen samenvoeging. Nodig: keten per apparaat plus kruisondertekening bij inname, en een monotone epoch-teller |
| S19 | Geen updatekanaal én geen telemetrie | Bij een kritiek lek in een parser is er geen manier om klanten te bereiken of te weten wie kwetsbaar is |
| S20 | Herstelshares zonder levenscyclus | Geen rotatie bij personeelswissel, geen periodieke bevestiging dat de shares nog bestaan |
| S21 | Term "WORM" op een lokaal bestand | Feitelijk onjuist woord in een document dat aan een toezichthouder wordt getoond |

### Invoegtekst — nieuw §6.x "Bewijskracht en de grenzen daarvan"

> **Wat de kluis wel en niet bewijst.** De hashketen bewijst dat de inhoud van de kluis na vastlegging niet ongemerkt is gewijzigd door iemand zónder de sleutels. Zij bewijst uit zichzelf niet dat een record is vastgelegd op het moment dat het claimt, want de houder van de sleutels beheert ook de systeemklok en het bestand. Voor bewijskracht tegenover een toezichthouder of een rechter is daarom externe verankering nodig:
>
> 1. **Verplicht dagelijks anker.** De kluis produceert per dag een `KETENANKER (dagdigest, volgnummerbereik, tijdstip)`. Het anker verlaat de kluis als los, klein bestand zonder persoonsgegevens.
> 2. **Drie ankerkanalen, ten minste één verplicht ingericht vóór productiegebruik:** (a) een tijdstempeltoken van een gekwalificeerde tijdstempeldienst conform RFC 3161, offline geïmporteerd via een handmatige verzoek-en-antwoordstroom; (b) verzending van de dagdigest naar een postbus buiten beheer van de FG, bijvoorbeeld die van de bestuurssecretaris of de externe accountant; (c) periodieke afdruk met paraaf. Optie (a) is de enige die bewijskracht levert tegenover een derde en is de standaard.
> 3. **Klokdiscipline.** Elk klokgevoelig record legt naast de wandkloktijd een monotone teller, de tijdzone-offset en de laatst bekende afwijking ten opzichte van het laatste externe anker vast. Een terugsprong van de systeemklok is een auditgebeurtenis die niet kan worden onderdrukt.
> 4. **Terminologie.** De term WORM wordt in alle uitvoer vervangen door *append-only met ketenverificatie en externe verankering*. In het dossier staat één alinea die uitlegt wat dit wel en niet bewijst.
>
> **Gepubliceerde specificatie.** Het bestands- en bundelformaat, de hashketen, de manifestopbouw en de handtekeningen worden gespecificeerd in een openbaar, versienummerd document met testvectoren, zodat een derde een onafhankelijke verifier kan bouwen. Zonder dat blijft `dpofg-verify` een verifier van de leverancier die het formaat van de leverancier controleert, en dat is geen onafhankelijke verificatie.

### Invoegtekst — nieuw §6.y "Onveilige standaarden die expliciet zijn omgekeerd"

> | Standaardinstelling | Waarde |
> |---|---|
> | TOTP-noodpad | **Vervalt volledig.** Er zijn precies twee ontgrendelpaden: wachtwoordzin plus geregistreerde authenticator, en Shamir-herstel. Ten minste twee authenticators moeten zijn geregistreerd vóór de kluis productiegegevens accepteert |
> | Vergrendeling | Automatisch bij schermvergrendeling, slaapstand, gebruikerswissel en na tien minuten inactiviteit; sleutelmateriaal verlaat het geheugen. Geen optie om de wachtwoordzin te onthouden |
> | Klembord | Wissen na dertig seconden; cloudklembord wordt gedetecteerd en geblokkeerd met blokkerende waarschuwing |
> | Export | Standaard versleuteld naar een ontvangersleutel; onversleutelde export vereist een gelogde overrule met motivering en verschijnt in het auditdossier |
> | Compartimenten | Envelopeversleuteling per compartiment bovenop de kluissleutel. Geïndexeerde tekst van een compartiment staat in een aparte, met de compartimentsleutel versleutelde index. De databasesleutel alleen geeft geen toegang tot compartimentsinhoud |
> | Parser- en OCR-processen | Eigen sandbox per platform, zonder netwerkcapability en zonder schrijfrechten buiten een tijdelijke map, met een harde geheugen- en tijdlimiet |
> | WebView | Devtools en remote debugging uit in release; crashrapportage, spellingcontrole en reputatiediensten uitgeschakeld via het installatiebeleid; startcontrole weigert te starten op een engineversie onder de gepubliceerde ondergrens en meldt dat expliciet |
> | Redactie | Uitvoer wordt na redactie opnieuw opgebouwd en vervolgens automatisch teruggelezen: tekstextractie, metadatascan en beeldvergelijking. Vindt de controle een geredigeerde term terug, dan is publicatie geblokkeerd |
> | Sleutelrotatie | Verplichte rotatie van de dataversleutelingssleutel bij elke wisseling van een sleutelhouder en ten minste jaarlijks, met herversleuteling op de achtergrond en een rotatiebewijsstuk |
> | Herstelshares | `SHARE (houder, uitgiftedatum, laatste_bevestiging, vervaldatum)` met een halfjaarlijkse bevestigingsverplichting in de werkbak; SLIP-0039-compatibele shares met integriteitscontrole |
> | Auditlogboek | Kan door geen enkele rol worden uitgeschakeld, ook niet door de beheerder; het uitschakelbaar maken is geen configuratieoptie |

---

# 4. Te schrappen of drastisch te versoberen

| # | Onderdeel | Oordeel | Actie |
|---|---|---|---|
| K1 | **Redactiewerkbank met OCR en patroonherkenning, 3,5 week** | Dit is een zelfstandig product. Realistisch twaalf tot twintig weken om het veilig te doen, en R11 noemt terecht het existentiële risico. Eén ontwikkelaar moet dit niet bezitten | Vervangen door een redactie-*regie*module: markeren, uitsluiten, exporteren naar een bewezen extern redactiehulpmiddel, en verplichte terugleescontrole op het eindbestand. Zie invoegtekst |
| K2 | **On-premise servervariant** | Een tweede product: andere authenticatie, andere back-up, andere hardening, meerdere schrijvers. Vier weken is een orde van grootte mis | Schrappen tot na v2, of vervangen door een gedeelde-map-uitcheckvariant |
| K3 | **TOTP- en PIV-terugvalpaden** | Vier sleutelpaden betekent vier keer testen en het zwakste pad bepaalt het niveau | Schrappen. Twee paden |
| K4 | **Crosswalk naar tien raamwerken** | CIS v8, DORA en CRA als mappingdoel leveren geen wettelijke waarde, wel een jaarlijkse hercontrole en aansprakelijkheid | v1 beperken tot Cbw, Cbb, UV 2024/2690, AVG en één van ISO 27001 of NEN 7510 of BIO2, afhankelijk van de klantsector. De rest als contentpack ná v1 |
| K5 | **Byte-identieke reproduceerbare builds op drie platforms** | Weken werk aan een claim die vrijwel geen klant zal narekenen | Vervangen door gepubliceerde hashes, ondertekende SBOM en een reproduceerbare bouwomgeving zonder byte-gelijkheidsgarantie |
| K6 | **Bitemporaliteit op alles** | Verdubbelt de complexiteit van elke query en elke migratie | Beperken tot `VERWERKING`, `CONTROL`, `BEWIJSSTUK`, `BELEIDSDOCUMENT` en `MAPPING`. Elders volstaat een gewoon wijzigingslogboek |
| K7 | **Module 17 campagnes en deelnameregistratie** | Elk leerplatform doet dit beter en het trekt personeelsgegevens de kluis in, wat R9 verergert | Terugbrengen tot het registreren van bewijs van deelname op groepsniveau plus het bestuurdersopleidingsspoor. Campagnebeheer schrappen |
| K8 | **Driftdetectie-importers voor CMDB, applicatieportfolio en contractbeheer** | Drie koppelvlakken zijn drie onderhoudsstaarten | Eén generieke veldmappingtool met bewaarde profielen en een verschilrapport |
| K9 | **AP-uitvraagsimulatie** | Aardig in een demo, weinig waarde in gebruik | Schrappen; de capaciteit gaat naar de tegenspraakbibliotheek (zie F5 hieronder) |
| K10 | **Benchmarkkengetallen in v2** | Vereist gegevens over klanten heen en botst frontaal met het zero-egressuitgangspunt | Schrappen, of leveren als statisch contentpack op basis van openbare bronnen |
| K11 | **Volledige WCAG 2.2 AA-audit in één week** | Onhaalbaar en voor interne desktopsoftware juridisch minder dwingend dan voor overheidswebsites | Toetsenbordbediening, contrast en schermlezerlabels als bouwvoorschrift vanaf dag één; formele audit pas wanneer een aanbesteding erom vraagt |
| K12 | **Module 18a intakepoortje als aparte binary** | Terecht al achter een criterium gezet, maar het staat wel in de begroting | Uit de basisplanning halen; pas begroten wanneer het criterium is gehaald |

### Invoegtekst — vervangt de regel "Redactiewerkbank" in fase 2

> **Redactieregie (2,0 weken).** De tool bepaalt *wat* moet worden geredigeerd en bewijst *dat* het is gebeurd; zij voert de redactie op de beeld- en tekstlaag niet zelf uit. Concreet: identificatie van te redigeren passages op basis van het register en van patroonlijsten, een uitvoerpakket naar een aangewezen extern redactiehulpmiddel, en daarna een **verplichte terugleescontrole** op het teruggeleverde bestand: tekstextractie, metadata- en annotatiescan, en een pixelvergelijking op de geredigeerde gebieden. Slaagt de controle niet, dan blokkeert de tool verstrekking. Deze keuze is bewust: het zelf bouwen van een redactiepijplijn zou de meest waarschijnlijke oorzaak van een datalek dóór de tool in eigen beheer nemen, en dat past niet bij een product dat door één persoon wordt onderhouden. Het besluit wordt herzien wanneer er een tweede ontwikkelaar is.

---

# 5. Vijf ontbrekende, echt onderscheidende functies

### F1 — Onafhankelijk verifieerbare tijdsverankering met een open bewijsformaat
Zie de invoegtekst bij §3. Dit is de enige functie die van "wij hebben het netjes vastgelegd" een juridisch bruikbare bewering maakt, en geen enkele Nederlandse GRC-suite biedt het. Het is bovendien het antwoord op de vraag die elke toezichthouder als eerste stelt: hoe weet ik dat dit er gisteren ook al zo stond.

### F2 — Bestuursrechtelijke procesmodule
Op het moment dat toezicht overgaat in handhaving heeft de FG geen enkel hulpmiddel en de hoogste tijdsdruk.

> **Invoegtekst — nieuwe module 16b, Bestuursrechtelijk spoor.**
> Het `TOEZICHTDOSSIER` krijgt een procesfase: *informeel contact*, *toezicht*, *voornemen*, *handhaving*, *rechtsbescherming*. Bij elke overgang toont de tool wat er verandert:
> - Bij overgang naar *voornemen*: zienswijzetermijn als klok (Awb art. 4:8), met een dossierbundel die precies het voornemen dekt.
> - Bij een besluit: bezwaartermijn van zes weken vanaf de dag na bekendmaking (Awb art. 6:7 en 6:8) als onverlengbare, rood gemarkeerde klok, met beroepstermijn erachter.
> - Bij overgang naar een punitief traject: een niet-uitschakelbare melding over de verhouding tussen de medewerkingsplicht (Awb art. 5:20) en het zwijgrecht bij boeteoplegging (Awb art. 5:10a), met de aanbeveling juridische bijstand in te schakelen vóórdat verder wordt geantwoord.
> - Een **verstrekkingslogboek**: wat is wanneer, aan wie, in welke versie en met welk redactieprofiel verstrekt, met de hash van elke verstrekte bundel. Bij een latere uitvraag toont de tool wat de toezichthouder al heeft en waar de eerdere en de huidige verklaring van elkaar afwijken.

### F3 — Aantoonbaarheidsradar: prognose van bewijsverval
De tool weet van elk bewijsstuk de geldigheid, van elke control de frequentie en van elke mapping de reviewdatum. Die kennis wordt nu alleen achteraf gebruikt.

> **Invoegtekst — uitbreiding module 11.**
> **Vervalprognose.** Voor elk gekozen horizonpunt — dertig, negentig en driehonderdvijfenzestig dagen — toont de tool welke wettelijke eisen op dat moment niet langer aantoonbaar zijn en waarom: verlopen bewijsstuk, verstreken frequentie, verlopen certificaat, verlopen mandaat of contract, verlopen mappingreview, vervallen doorgifte-instrument. De uitvoer is geen lijst met taken maar een lijst met **eisen die onbewijsbaar worden**, met de eigenaar en de benodigde doorlooptijd erbij. Deze prognose is een eigen exporteerbaar rapport voor de bestuursvergadering, want dit is de enige vorm waarin een bestuur een informatiebeveiligingsrisico begrijpt: niet als kleur, maar als datum.

### F4 — Ketenbewijs: ondertekende bewijspakketten tussen organisaties
Cbb-leverancierstoetsing en AVG art. 28 lid 3 onder h leveren nu overal pdf's met beweringen op.

> **Invoegtekst — nieuwe module 15b, Ketenbewijs.**
> Een leverancier of verwerker kan met een gratis, apart ondertekende hulpbinary een **bewijspakket** samenstellen: een ondertekende, extern verankerde bundel met beleid, certificaten, testrapporten, subverwerkerslijst en de art. 28 lid 3-vindplaatsen, met een geldigheidsvenster per onderdeel. De ontvangende entiteit importeert dat pakket rechtstreeks in het leveranciersregister; de handtekening en het anker worden geverifieerd, de geldigheidsvensters worden bewaakt en het verlopen van een certificaat bij de leverancier verschijnt automatisch als verplichting bij de klant. Hetzelfde formaat werkt in de andere richting voor een verwerker die zijn verwerkingsverantwoordelijken moet bedienen. Dit is de enige functie in dit plan met een netwerkeffect: elke klant die het gebruikt, oefent druk uit op tien leveranciers om dezelfde bundel te leveren.

### F5 — Persoonlijk onafhankelijkheidsdossier van de FG, buiten de organisatiekluis
`ONAFHANKELIJKHEIDSINCIDENT` staat nu in een kluis die van de organisatie is, terwijl de tegenpartij bij een art. 38-conflict de organisatie is.

> **Invoegtekst — uitbreiding §7.3.**
> **Persoonlijk FG-dossier.** De functionaris voor gegevensbescherming kan een tweede, kleine kluis aanmaken die uitsluitend met zijn eigen sleutel te openen is en waarvan de organisatie de inhoud niet kan lezen, exporteren of verwijderen. Daarin staan uitsluitend: `ONAFHANKELIJKHEIDSINCIDENT`, uitgebrachte adviezen met hun bestuursreactie, escalatiestappen en de daarbij horende bewijsstukken. Records worden bij vastlegging in de organisatiekluis onzichtbaar gespiegeld als hash, zodat later kan worden aangetoond dát een advies op een bepaald moment bestond zonder de inhoud prijs te geven. Bij beëindiging van de aanstelling neemt de FG dit dossier mee; de organisatie behoudt de hashes en het gewone adviesregister. Grondslag en bestaansreden: AVG art. 38 lid 3, dat verbiedt de FG te ontslaan of te straffen voor de uitvoering van zijn taken — een bescherming die waardeloos is als het bewijs ervan uitsluitend berust bij degene tegen wie zij is gericht.
>
> *Eervolle vermelding, te overwegen na v1:* een **tegenspraakbibliotheek** die per control en per verwerkingssoort de concrete vervolgvragen en de aangetroffen tekortkomingen uit gepubliceerde boete- en handhavingsbesluiten toont, met vindplaats. Zuivere contentpackinhoud, geen code, en precies waarvoor een ervaren FG betaalt.

---

# 6. Techstack en fasering

| # | Bevinding |
|---|---|
| P1 | **De schatting is met een factor 2 à 3 mis.** 60 tot 72 weken bij dertig uur is ongeveer 2000 uur voor achttien modules, drie platforms, eigen crypto, eigen documentgeneratie, juridische inhoud en een verificatiegereedschap. In de begroting ontbreken support, foutherstel, implementatie bij de eerste klanten, documentatie, afhankelijkheidsonderhoud en releasewerk per platform. Na de eerste betalende klant gaat doorgaans minder dan de helft van de tijd naar nieuwe functionaliteit |
| P2 | **Het beslispunt over de stack ligt elf weken te laat.** Aan het eind van fase 0 is de kluiskern juist het enige wat af is; overstappen betekent dan precies het moeilijkste deel weggooien. Het besluit hoort na een spike van twee tot drie weken |
| P3 | **SQLCipher is grotendeels overbodig naast compartimentversleuteling.** Zodra compartimenten eigen sleutels krijgen, is er al versleuteling op applicatieniveau. Twee overlappende lagen betekent twee sleutelmodellen, twee migratiepaden en een C-afhankelijkheid in het hart. De licentievraag van R14 is bovendien minder scherp dan het plan suggereert: de community-editie kent een permissieve licentie. De echte reden om te kiezen is architectonisch, niet juridisch, en de keuze hoort in fase 0 |
| P4 | **Tauri betekent drie verschillende engines waarvan er één, WebKitGTK, structureel achterloopt.** R13 behandelt dit als een compatibiliteitsvraag; het is een kwetsbaarheidsvraag voor een toepassing die bijzondere persoonsgegevens rendert |
| P5 | **Typst is het verkeerde probleem.** Het echte probleem is niet welke bibliotheek de pdf maakt, maar dat de handtekening onder de uitvoer met standaardgereedschap verifieerbaar moet zijn. Een losse Ed25519-handtekening kan een toezichthouder niet controleren zonder de leverancierstools |
| P6 | **Faseringsconflict.** Fase 2 levert betrokkenenverzoeken en redactie tegelijk op, maar in fase 1 kan een auditdossier al worden geëxporteerd terwijl er nog geen redactiecontrole is. Bovendien staan het bestuurdersregister en de registratieplicht — verplichtingen die per 15-08-2026 lopen en waarvan er één op 15-08-2028 afloopt — pas in fase 3, terwijl de bestuurdersopleiding de eerste verplichting is waarop gehandhaafd zal worden |
| P7 | **De inhoud komt na de machine.** Het onderscheidend vermogen zit in de contentpacks, en die worden pas gemaakt als de machine er staat. Dat is de omgekeerde volgorde voor risico én voor omzet |

### Invoegtekst — vervangt de kop van §9

> **Herziene uitgangspunten van de schatting.** Eén ontwikkelaar, dertig productieve uren per week. Op de ontwikkeltijd per werkpakket wordt een opslag van veertig procent gelegd voor integratie, foutherstel en releasewerk; vanaf de eerste betalende klant wordt bovendien de helft van de beschikbare tijd gereserveerd voor support, implementatie en contentonderhoud. De onderstaande weken zijn ontwikkeltijd vóór die opslag. **Realistische doorlooptijd tot een volwaardige v1: 30 tot 42 maanden.** Wie dat niet accepteert, moet de scope halveren en niet de schatting.

### Invoegtekst — nieuwe fase vóór fase 0

> ### Fase −1 — Inhoud en marktbewijs
> **Duur: 6-8 weken** · *Resultaat: het juridische fundament bestaat en drie organisaties hebben ervoor getekend, vóórdat er één regel productiecode is geschreven.*
>
> | Werkpakket | Weken |
> |---|---|
> | Contentpackformaat als platte, versienummerde tekstbestanden in versiebeheer; normbepalingen Cbw, Cbb, UV 2024/2690 en AVG met vindplaats en consolidatiedatum | 2,0 |
> | Termijnencatalogus: elke verplichting met anker, duur, eenheid, rekenregel, grondslag en tegenvoorbeeld — eerst op papier, dan pas in code | 1,5 |
> | Crosswalk Cbw naar één sectornorm, met relatietype en motivering per rand | 1,5 |
> | Handmatige pilot bij drie organisaties met uitsluitend deze inhoud en bestaande hulpmiddelen | 2,0 |
>
> *Mijlpaal:* drie organisaties bevestigen schriftelijk dat zij voor het onderhoud van deze inhoud willen betalen. Wordt die mijlpaal niet gehaald, dan is het contentpack-abonnement geen prijsmodel en vervalt de zakelijke basis onder het hele plan.

### Invoegtekst — vervangt de betreffende regels in bijlage B

> | Beslispunt | Moment | Criterium |
> |---|---|---|
> | Rust of tweede-keuzestack | **Einde week 3 van fase 0**, na een spike op de moeilijkste drie onderdelen: CTAP2 `hmac-secret`, versleutelde opslag en de hashketen | Werken alle drie de onderdelen aantoonbaar op alle drie de platforms? |
> | Opslaglaag | **Einde fase 0** | Envelopeversleuteling per compartiment op gewone SQLite, tenzij aantoonbaar is dat versleuteling op databaseniveau daar bovenop iets toevoegt dat de tweede sleutelhiërarchie rechtvaardigt |
> | Handtekening onder uitvoer | **Einde fase 1** | Is het exportbestand verifieerbaar met standaardgereedschap dat een toezichthouder al heeft, zonder installatie van software van de leverancier? Zo nee, dan een eIDAS-conforme handtekening op de pdf, met de losse Ed25519-handtekening als aanvulling en niet als vervanging |
> | Documentgenerator | Einde fase 1 | Gepinde versie, eigen ondersteuningslaag, en een schriftelijke inschatting van het onderhoud per jaar. Byte-gelijkheid vervalt als criterium |
> | On-premise servervariant | **Vervalt uit de planning** | Wordt pas opnieuw begroot wanneer drie klanten er schriftelijk om vragen én er een tweede ontwikkelaar is |

### Invoegtekst — twee nieuwe risico's bij §11

| # | Risico | Ernst | Mitigatie |
|---|---|---|---|
| R22 | **De leverancier is zelf fabrikant onder de CRA.** Verordening (EU) 2024/2847 legt aan fabrikanten van producten met digitale elementen verplichtingen op inzake kwetsbaarhedenbeheer, ondersteuningsperiode en melding; de meldplicht geldt vanaf 11 september 2026. Een product dat klanten helpt aan de Cbw te voldoen terwijl de maker zijn eigen verordening niet naleeft, is bij de eerste inspectie onverkoopbaar | Hoog | Eigen coordinated-vulnerability-disclosurebeleid en `security.txt` vóór de eerste release; kwetsbaarhedenafhandelingsproces met termijnen; SBOM per release; vastgelegde ondersteuningsperiode in de licentievoorwaarden; meldprocedure richting CSIRT en ENISA ingericht vóór 11-09-2026 |
| R23 | **Geen updatekanaal betekent geen herstelvermogen.** Zonder automatische update én zonder telemetrie is onbekend wie een versie met een bekend lek draait, en is er geen weg om die te bereiken | Hoog | Elk contentpack draagt een veld `minimaal_aanbevolen_applicatieversie` en een ondertekend beveiligingsbulletin; de tool waarschuwt blokkerend bij een te oude versie zonder ooit zelf contact te zoeken. Daarnaast een gepubliceerde adviesmailinglijst waarop elke klant bij ingebruikname wordt ingeschreven, contractueel vastgelegd |

### Invoegtekst — aanvulling op bijlage A

> **Bij elke export en elk auditdossier, direct onder de ondertekening:**
> > De integriteit van dit dossier is te controleren met de gepubliceerde formaatspecificatie en de bijgevoegde ankerbestanden. De ketenverificatie toont aan dat de inhoud na vastlegging niet ongemerkt is gewijzigd. Zij toont uit zichzelf niet aan op welk moment een record is vastgelegd; die vaststelling berust op de bijgevoegde externe tijdstempels. Ontbreken die, dan berust het tijdstip uitsluitend op de opgave van de organisatie zelf, en is dat in dit dossier als zodanig aangemerkt.

---

## Bronnen bij de feitelijke toetsing

- [Cyberbeveiligingswet en Wet weerbaarheid kritieke entiteiten vanaf 15 augustus 2026 van kracht — Rijksoverheid](https://www.rijksoverheid.nl/actueel/nieuws/2026/07/07/cyberbeveiligingswet-en-wet-weerbaarheid-kritieke-entiteiten-vanaf-15-augustus-2026-van-kracht)
- [Overheidsinstanties onder de Cyberbeveiligingswet — RDI](https://www.rdi.nl/onderwerpen/digitale-weerbaarheid/cyberbeveiligingswet/sectoren-onder-toezicht/overheid)
- [Registratieplicht Cyberbeveiligingswet — RDI](https://www.rdi.nl/onderwerpen/digitale-weerbaarheid/cyberbeveiligingswet/registratieplicht)
- [De Cyberbeveiligingswet is van kracht — Z-CERT](https://z-cert.nl/actueel/nieuws/de-cyberbeveiligingswet-is-van-kracht)
- [Het NCSC levert voorlopig CSIRT-dienstverlening — Digitale Overheid](https://www.digitaleoverheid.nl/nieuws/het-ncsc-levert-voorlopig-csirt-dienstverlening/)
- [Cyberbeveiligingsregeling overheid in Staatscourant — Digitale Overheid](https://www.digitaleoverheid.nl/nieuws/cyberbeveiligingsregeling-overheid-in-staatscourant/)
- [BIO2 v1.3 gepubliceerd in de Staatscourant — bio-overheid.nl](https://www.bio-overheid.nl/nieuws/bio2-v13-gepubliceerd-in-de-staatscourant/)
- [Uitvoeringsverordening (EU) 2024/2690, art. 3 en 4 — EUR-Lex](https://eur-lex.europa.eu/legal-content/NL/TXT/HTML/?uri=OJ:L_202402690)
- [Verordening (EEG, Euratom) nr. 1182/71, art. 3](https://www.legislation.gov.uk/eur/1971/1182/article/3/1971-06-08)
- [EDPB Guidelines 01/2022 on data subject rights — Right of access](https://www.edpb.europa.eu/system/files/2023-04/edpb_guidelines_202201_data_subject_rights_access_v2_en.pdf)
- [European Commission renews UK data adequacy decisions (19 december 2025)](https://www.hunton.com/privacy-and-information-security-law/european-commission-renews-uk-data-adequacy-decisions)
- [EU-US Data Privacy Framework, status 2026](https://europeanmartech.eu/blog/eu-us-data-privacy-framework-2026-status)
- [Digital Omnibus AI, Verordening (EU) 2026/1744 — nieuwe deadlines hoogrisico-AI](https://www.secureaudit.nl/kennisbank/eu-ai-act-digital-omnibus-deadlines)