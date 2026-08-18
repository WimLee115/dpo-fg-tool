# Foutbestendig ontwerp

*Ontwerphoofdstuk voor `dpo-fg-tool` — vertaling van de eis "gebruikers moeten vrijwel geen fouten kunnen maken" naar datamodel, schermgedrag, controleregels en meetbare normen.*

---

## 0. Uitgangspunt, reikwijdte en toetsnorm

### 0.1 De eis

De opdrachtgever stelt één eis centraal: **de tool laat de fout niet toe, of maakt hem onschadelijk.** Waarschuwen is daarmee expliciet niet het ontwerpdoel maar het vangnet. Dat is een harde eis, want de onderzoeksbasis laat zien waar het misgaat: niet bij kwaadwilligheid, maar bij normale mensen die onder tijdsdruk routinewerk doen in systemen die de fout toestaan. Het menselijk element zit in circa 62% van alle inbreuken (Verizon DBIR 2026); bij de AP is 64% van de 39.407 meldingen over 2025 één enkele categorie: een brief naar de verkeerde geadresseerde.

De consequentie voor dit project is dat **de gebruiker geen ontwerpvariabele is.** Het formulier, het datamodel, de statusovergang en de termijnmodule zijn dat wel.

### 0.2 Twee soorten fouten: wat deze tool kan uitsluiten en wat zij kan beheersen

Het is eerlijker en scherper om dit vooraf te scheiden.

| Klasse | Voorbeeld uit de onderzoeksbasis | Wat `dpo-fg-tool` doet |
|---|---|---|
| **A. Fouten binnen de applicatie** — de FG of compliance-medewerker maakt de fout in dit systeem | Grondslag "toestemming" waar die niet vrij is; bewaartermijn ontbreekt; 72-uursklok te laat gestart; besluit "niet melden" op de verkeerde grond; verlengingsbericht te laat; VWO die alleen de wettekst overschrijft | **Structureel uitsluiten.** Dit is het volledige toepassingsgebied van trede 1 tot en met 3 van de ontwerpladder. Hier geldt de eis onverkort. |
| **B. Fouten in uitgaande communicatie van de applicatie zelf** | Bcc-fout in een bericht aan betrokkenen; verkeerde bijlage bij een antwoord op een inzageverzoek; onvolledige lakking bij publicatie van het register of een Woo-stuk | **Structureel uitsluiten.** De tool mag de klassieke fouten uit de onderzoeksbasis niet zelf reproduceren. Zie §3.7. |
| **C. Fouten elders in de organisatie** | Brief uit de printstraat naar de verkeerde geadresseerde; autocomplete in de mailclient; verborgen werkblad in een gedeeld Excel-bestand; onterechte inzage door een geautoriseerde medewerker; persoonsgegevens in een publieke chatdienst | **Niet uit te sluiten door deze tool.** Wel: foutbestendig registreren, structureel op oorzaakcategorie ontleden (art. 33 lid 5), de maatregel eraan koppelen en de opvolging van die maatregel bewaken tot bewijs van uitvoering. De tool maakt het herhaalpatroon zichtbaar en de belofte afdwingbaar. |

Deze scheiding hoort in het projectplan te staan, omdat zij voorkomt dat het ontwerp beloftes doet die het niet waar kan maken, en omdat zij het werk aan klasse C een concrete vorm geeft: patroondetectie plus maatregelbewaking, niet "bewustwording".

### 0.3 De toetsnorm voor elk ontwerpbesluit

Voor **elke** beheersmaatregel in de applicatie legt het ontwerp vast:

1. Welke fout zij vangt.
2. Op welke trede van de ontwerpladder zij zit (§1).
3. Waarom een hogere trede niet haalbaar was.
4. In welke laag van het lagenmodel zij zit, en waardoor zij faalt.
5. Welke volgende laag die faalmodus opvangt.

Zonder punt 3 en 5 is een maatregel niet af. Dit is de reden dat dit hoofdstuk in het projectplan hoort en niet in code-commentaar: het is een ontwerpverantwoording, geen implementatiedetail.

Standaardvorm van zo'n verantwoording:

```
MAATREGEL M-142  Bewaartermijn verplicht bij vaststellen
Fout            Registerregel zonder bewaartermijn (art. 30(1)(f))
Trede           T3 — blokkerend bij de overgang Concept → Vastgesteld
Waarom niet     T1 onmogelijk: de juiste termijn is een organisatorisch besluit
hoger?          en volgt niet uit andere gegevens. T2 gedeeltelijk toegepast:
                de bibliotheek vult een richtwaarde voor, bevestiging blijft nodig.
Laag            Laag 2 — structurele validatie bij statusovergang
Faalt bij       Een plausibele maar inhoudelijk onjuiste termijn
Volgende laag   BEW-04 (afwijking van bibliotheeknorm, signaal) en de
                verplichte herziening na 12 maanden (REG-08)
Ontsnapping     "Nog te bepalen, uiterlijk [datum], eigenaar [rol]" — wordt
                automatisch een taak met termijn; voorkomt een verzonnen waarde
```

### 0.4 Het lagenmodel en de onafhankelijkheidstoets

Lagen zijn alleen iets waard als hun gaten niet gecorreleerd zijn. Drie validatieregels achter elkaar zijn één laag.

| Laag | Vangt | Faalt bij |
|---|---|---|
| 1. Datamodel- en invoerbeperking | slips, typefouten, onmogelijke toestanden | oordeelsfouten |
| 2. Structurele validatie bij statusovergang | lapses, onvolledigheid | plausibel maar onjuist ingevulde velden |
| 3. Beslisondersteuning met criteria in beeld | rule-based mistakes | tijdsdruk, routine, habituatie |
| 4. Vier ogen op de onomkeerbare stap | mistakes en violations | eenpersoonsorganisatie, bevestigingsdruk |
| 5. Time-out vlak vóór verzending | slips op het laatste moment | hoge frequentie (habituatie) |
| 6. Continue controles, auditspoor, termijnbewaking | achteraf-detectie, drift | niets — dit is de laatste technische laag |
| 7. Periodieke hercontrole en directierapportage | latente condities, genormaliseerde deviatie | traagheid |

**Onafhankelijkheidstoets bij elk ontwerpbesluit:** als deze laag faalt door tijdsdruk, faalt de volgende dan óók door tijdsdruk? Zo ja, is er geen tweede laag en moet er een gezocht worden met een andere faalmodus — bijvoorbeeld een andere persoon (laag 4), een ander moment (afkoelperiode) of een ander tijdvenster (laag 6/7).

---

## 1. De ontwerpladder

### 1.1 De zes treden

Elke maatregel in `dpo-fg-tool` krijgt een positie op deze ladder. Hoger is beter; lager vereist verantwoording.

| Trede | Naam | Mechanisme | Poka-yoke | Vangt | Toepasbaar wanneer |
|---|---|---|---|---|---|
| **T1** | Fout is onmogelijk | Forcing function, lockout, datamodelbeperking, ontbrekende keuze | control | slips, lapses, mistakes, violations | de foute toestand hoeft niet te kunnen bestaan |
| **T2** | Fout wordt automatisch gecorrigeerd of afgeleid | Afleiding uit bekende gegevens, normalisatie, standaardisatie, taakaanmaak | control | slips, lapses | de juiste waarde volgt uit gegevens die het systeem al heeft |
| **T3** | Fout wordt geblokkeerd bij de statusovergang | Interlock, volledigheidscontrole, verplichte motivering | control | lapses, deel van de mistakes | juistheid objectief bepaalbaar op het moment van de overgang |
| **T4** | Fout wordt gesignaleerd | Tegenspraak, criteria in beeld, prominent passief signaal | warning | rule-based mistakes | oordeel nodig; juistheid niet objectief bepaalbaar |
| **T5** | Fout wordt achteraf gedetecteerd | Continue controles, auditspoor, kwaliteits- en directierapportage | warning | knowledge-based mistakes, violations, latente condities | de fout is pas in samenhang of over tijd zichtbaar |
| **T6** | Instructie en opleiding | Microtoelichting in context, procedurebeschrijving | n.v.t. | niets betrouwbaar | uitsluitend als aanvulling, nooit als enige maatregel |

**Harde regel:** een waarschuwing (T4) op een objectief bepaalbaar feit is een ontwerpfout. Als het systeem kan weten dat iets fout is, mag het de fout niet toestaan.

**Tweede harde regel:** een maatregel die uitsluitend op T6 zit, is geen maatregel. Zij wordt geregistreerd als openstaande ontwerpschuld met de vraag of T1 tot en met T3 alsnog haalbaar is.

### 1.2 Per trede: voorbeelden uit deze applicatie

#### T1 — De fout is onmogelijk

| Fout die verdwijnt | Ontwerp |
|---|---|
| Verkeerd getypte grondslag, eigen bedachte grondslag | Er is geen tekstveld voor de grondslag. Zes radio's (art. 6(1)(a) t/m (f)), plus een gescheiden art. 9(2)-blok. |
| Backdating van de ontdekkingsdatum om de 72 uur te halen | Registratiemoment is een systeemgegeven en niet invoerbaar. Kennisnamemoment is een apart, invoerbaar veld. Het verschil is altijd zichtbaar. |
| Termijn met de hand berekenen en misrekenen | Er is geen invoerveld voor een deadline. Alle deadlines zijn afgeleid door één termijnmodule (§3.5). |
| Onherstelbaar verwijderen | Hard verwijderen bestaat niet in het datamodel. Alles is append-only; "verwijderen" is intrekken met einddatum en reden. |
| Bcc-fout in een bericht aan betrokkenen | Bij meer dan één ontvanger biedt de tool uitsluitend individuele verzending of een gescheiden verzendlijst aan. Eén bericht met meerdere zichtbare geadresseerden is geen bestaande handeling. |
| Tekstlaag onder de lakking bij publicatie | De publicatie-export produceert uitsluitend documenten waarbij het gelakte deel is verwijderd, niet bedekt, en waarbij documenteigenschappen zijn gestript. Er is geen "publiceren zoals het is". |
| Persoonsgegevens buiten de organisatie via een netwerkfunctie | Netwerkverkeer staat standaard uit en wordt per functie bewust aangezet (lockout, conform README). |
| BSN in een vrij tekstveld van een datalekomschrijving | Invoerfilter weigert het patroon met uitleg; het veld hoeft geen BSN te bevatten. |
| Bulkmelding gebruikt als verzamelbak voor ongelijksoortige incidenten | De groepssamensteller biedt uitsluitend incidenten aan met identiek type, oorzaak en gegevenscategorie binnen het toegestane venster (EDPB Guidelines 9/2022 par. 63-65). Andere incidenten zijn niet selecteerbaar. |
| "Geen bewijs van exfiltratie, dus geen lek" | De keuzelijst kent alleen "uitgesloten op grond van [technisch bewijs]" en "niet uit te sluiten". De onjuiste conclusie is geen selecteerbare uitkomst. |
| "We willen geen onnodige onrust" als reden om betrokkenen niet te informeren | De redenlijst bij art. 34 kent uitsluitend de drie uitzonderingen van art. 34(3), elk met verplichte onderbouwing. Het reputatieargument bestaat niet als keuze. |
| Capture error: doorwerken in het verkeerde dossier | Elke bewerking hangt aan een dossier-id; contextwissel vereist een expliciete handeling die de focusmodus zichtbaar verbreekt. |
| Register per afdeling of per systeem inrichten | Het aggregaat in het datamodel is de verwerkingsactiviteit. Afdeling en systeem zijn attributen, geen containers. Een "afdelingsregel" is niet aan te maken. |
| Doelen als losse opsomming naast losse grondslagen | Doel en grondslag zijn één paar in het datamodel. Een doel zonder eigen grondslag kan niet bestaan (AP-aanbeveling 5). |

#### T2 — De fout wordt automatisch gecorrigeerd of afgeleid

| Fout die verdwijnt | Ontwerp |
|---|---|
| Doorgifte buiten de EER over het hoofd gezien | Elk land buiten de EER in "opslaglocatie", "toegang vanuit" of "subverwerker" opent automatisch het hoofdstuk V-blok en maakt de taak "waarborg vaststellen" aan. |
| Vergeten DPIA-toets bij bijzondere gegevens | Aanvinken van bijzondere gegevens, grootschalige monitoring, profilering of nieuwe technologie maakt de DPIA-toets automatisch tot een verplicht onderdeel van de registerregel. |
| Vergeten art. 34-beoordeling na de AP-melding (post-completion error) | Verzending van de AP-melding maakt automatisch de taak "besluit informeren betrokkenen" aan, met eigen klok en eigenaar. |
| Registers die uit elkaar lopen ("Leverancier BV" / "Leverancier B.V.") | Normalisatie bij invoer plus fuzzy-duplicaatdetectie die de bestaande waarde vóórstelt in plaats van een nieuwe aan te maken. |
| Overgetypte dossiernummers die naar het verkeerde dossier verwijzen | Dossiernummers worden toegekend, nooit getypt, en bevatten een controleteken (mod-37). |
| Bewaartermijn als onbruikbare tekst | Termijn is (getal, eenheid, startgebeurtenis, grondslag); de concrete verwijderdatum per record wordt daaruit berekend en getoond. |
| Achterstallige actualiteit van het register | Een wijziging in een gekoppeld object (leverancier, systeem, hostingland, subverwerkerslijst) zet de afhankelijke registerregels automatisch op "herziening nodig" met vermelding van de reden. |
| Herhaald invullen van dezelfde toepasselijkheidsvraag | NIS2-toepasselijkheid, leidende toezichthouder en organisatietype worden één keer per entiteit vastgelegd en daarna afgeleid, nooit per incident opnieuw gevraagd. |
| Werk kwijt bij navigeren of onderbreking | Automatisch concept opslaan bij het verlaten van elk veld plus op interval; hervattingsanker bij terugkeer. |
| Verkeerde velden zichtbaar bij rol verwerker | Rol "verwerker" toont art. 30(2)-velden en verbergt art. 30(1)-velden. |

#### T3 — De fout wordt geblokkeerd bij de statusovergang

Uitgangspunt: **verplicht is een eigenschap van de overgang, niet van het veld.** Een concept mag altijd onvolledig zijn. Dat is de belangrijkste maatregel tegen situationele violations: niemand hoeft ooit iets te verzinnen om verder te komen.

| Overgang | Geblokkeerd tot |
|---|---|
| Registerregel → Vastgesteld | doel(en) met gekoppelde grondslag, categorieën betrokkenen en gegevens, ontvangers of expliciet "geen", bewaartermijn of gemotiveerde uitstelafspraak, systeemkoppeling, en bij doorgifte een waarborg |
| Datalek → Melding indienen | risicobeoordeling afgerond, de vier elementen van art. 33(3), contactgegevens FG, ontvangende toezichthouder afgeleid, checklist afgevinkt, tweede persoon akkoord |
| Datalek → Besluit "niet melden" | motivering, de gestructureerde risicoweging, en tweede persoon of afkoelperiode |
| Datalek → Afgesloten | art. 34-besluit, maatregelen met eigenaar en datum, evaluatie, oorzaakcategorie |
| Betrokkenenverzoek → Verlengd | verlengingsgrond gekozen én verzending van het verlengingsbericht geregistreerd binnen de eerste maand |
| Betrokkenenverzoek → Afgehandeld | elke vindplaats uit de afgeleide lijst expliciet afgehandeld |
| DPIA → Afgerond | de vier onderdelen van art. 35(7), geregistreerd FG-advies, en bij hoog restrisico een voorafgaande raadpleging |
| VWO → Actief | alle onderdelen van art. 28(3) gemapt op een vindplaats in het contract, met concrete invulling |
| Sjabloonversie → Gepubliceerd | review door een tweede persoon en uitgevoerde impactanalyse |

#### T4 — De fout wordt gesignaleerd

Uitsluitend waar oordeel nodig is. Altijd prominent passief (een strook in het dossier), nooit een modal, altijd met een oplossingsknop.

- Grondslag "toestemming" in een gezagsverhouding: tegenspraakblok met de alternatieven art. 6(1)(b)/(c)/(e) in beeld.
- Bewaartermijn langer dan de richtwaarde uit de bibliotheek, zonder motivering.
- Datalek beoordeeld als "laag risico" bij meer dan 250 betrokkenen: verplichte tegenspraakvraag over schaal en misbruikscenario (de AddComm-les: 16 organisaties informeerden 82.893 mensen alsnog).
- Meldtermijn voor de verwerker boven 24 uur, met de rekensom in beeld: "verwerker meldt binnen 48 uur, u houdt 24 uur over".
- Registerregel niet herzien in twaalf maanden.
- VWO die het documenttype "bewerkersovereenkomst" draagt of naar ingetrokken wetgeving verwijst.

#### T5 — De fout wordt achteraf gedetecteerd

De volledige set continue controles uit §5, het auditspoor, de kwartaalrapportage aan de directie, en de patroonanalyse over het interne datalekregister (art. 33 lid 5): dezelfde printstraat, hetzelfde formulier, dezelfde mailinglijst.

#### T6 — Instructie

Uitsluitend als microtoelichting náást het veld, op het moment van de beslissing, met de wettelijke grond erbij. Nooit als losse cursus, nooit als vervanging van een hogere trede. De onderzoeksliteratuur is hierover eenduidig: training als primaire beheersmaatregel is voorspelbaar teleurstellend.

### 1.3 Verdeling als ontwerpnorm

Voor de eerste release geldt als richtnorm dat ten minste **de helft van alle geïdentificeerde faalwijzen op T1 of T2 wordt afgevangen** en dat geen enkele faalwijze uitsluitend op T4 of lager rust. Deze verdeling wordt bij elke release gemeten en gerapporteerd (§6).

---

## 2. Foutbestendig ontwerp per werkproces

Elk werkproces volgt hetzelfde stramien: wat er misgaat, welk ontwerpbesluit dat onmogelijk maakt, hoe het statusmodel eruitziet, en welk restrisico er overblijft met het bijbehorende vangnet.

---

### 2.1 Verwerkingsregister aanleggen en actueel houden

#### Wat er misgaat

- Register zonder bewaartermijnen en zonder opslaglocaties; art. 30(1)(f) "indien mogelijk" gelezen als vrijblijvend.
- Register ingedeeld per afdeling of per systeem, met een losse opsomming van doelen — niet herleidbaar welk doel bij welke verwerking hoort (AP-aanbeveling 5).
- Register niet gebruikt als vertrekpunt om verwerkers en verwerkersovereenkomsten te inventariseren; register bij de FG, contracten bij inkoop, systemen bij IT (AP-aanbeveling 2).
- Register als eenmalige klus uit 2018; nooit bijgewerkt bij nieuwe systemen, verwerkers of organisatiewijzigingen.
- Register leeft in een persoonlijke Excel van de FG en verdwijnt bij vertrek.
- Nieuwe FG neemt de bestaande documentatie als uitgangspunt en toetst haar niet tegen de werkelijkheid.

#### Ontwerpbesluiten

| Fout | Trede | Maatregel |
|---|---|---|
| Indeling per afdeling of systeem | T1 | Het aggregaat is de verwerkingsactiviteit. Afdeling, proces en systeem zijn koppelingen. Een registerregel zonder doel bestaat niet. |
| Doelen los van grondslagen | T1 | Doel en grondslag vormen één paar. Bij elk extra doel verschijnt een eigen grondslagkeuze. De registerexport toont de paren, nooit twee kolommen. |
| Bewaartermijn ontbreekt | T3 + T2 | Blokkerend bij vaststellen; bibliotheek stelt een richtwaarde voor; ontsnapping "nog te bepalen, uiterlijk [datum], eigenaar [rol]" wordt automatisch een taak. |
| Opslaglocatie ontbreekt | T1 | Opslaglocatie is geen tekstveld maar een koppeling naar een systeemobject, dat op zijn beurt hostingland, toegangsland, leverancier en contract kent. |
| Verwerkers onzichtbaar | T1 | De koppelsleutel bestaat structureel: verwerking → systeem → leverancier → contract. Het register is daarmee per definitie de inventarisatiebron voor verwerkersovereenkomsten. |
| Register veroudert | T2 + T5 | Wijziging in een gekoppeld object zet afhankelijke regels op "herziening nodig". Herzieningsdatum per regel; REG-08 signaleert 12 maanden, rapporteert vanaf 15. |
| Register in een persoonlijke Excel | T1 | Er is geen ondersteund werkproces waarin een registerregel buiten de tool bestaat. Bijlagen landen in de versleutelde opslag, niet als verwijzing naar een netwerkpad. |
| Geërfd register wordt niet getoetst | T1 + T5 | Elke geïmporteerde of overgenomen regel draagt het kenmerk **"geërfd, niet geverifieerd"**. Dat kenmerk is niet handmatig te wissen; het verdwijnt alleen door de nulmeting (§2.10) en is tot die tijd zichtbaar in elke rapportage en elke export. |
| Bijwerken duurt te lang, dus gebeurt het niet | T2 | Doorlooptijdnorm: een bestaande registerregel bijwerken in minder dan twee minuten. Overschrijding is een defect, geen gedragsprobleem (§6). |
| Wees-systemen en wees-leveranciers | T5 | REG-10 en REG-11: een systeem of leverancier zonder enige registerkoppeling is een blinde vlek en verschijnt in de rapportage. |

#### Statusmodel

```
Concept ──▶ Ter beoordeling proceseigenaar ──▶ Vastgesteld
   ▲                                              │
   └────────── Wijziging voorgesteld ◀────────────┤
                                                  ▼
                                    Herziening nodig ──▶ Vastgesteld
                                                  │
                                                  ▼
                                            Ingetrokken (einddatum + reden)
```

- "Vastgesteld" is onbereikbaar zonder goedkeuring van de proceseigenaar (interlock). De FG stelt niet in zijn eentje vast wat een ander uitvoert.
- Een vastgestelde regel is zichtbaar vergrendeld met een route "Wijziging voorstellen", niet grijs zonder uitleg.
- Ingetrokken regels blijven reconstrueerbaar; de verantwoordingsplicht vereist dat het register op een historische datum te tonen is.

#### Restrisico en vangnet

Een plausibele maar onjuiste inhoudelijke invulling (verkeerde categorie betrokkenen, te ruim doel) wordt niet door laag 1 of 2 gevangen. Vangnet: de goedkeuring door de proceseigenaar (andere persoon, andere kennis), de jaarlijkse herziening (ander moment) en de confrontatiecontroles REG-10, REG-11 en REG-15.

---

### 2.2 Beoordelen of een DPIA verplicht is

#### Wat er misgaat

- DPIA niet uitgevoerd terwijl het moest; een pre-scan concludeert onder tijdsdruk te snel dat een volledige DPIA niet nodig is, en die pre-scan komt pas ná aanvang van de verwerking (AP-boete politie, camera-auto's).
- FG te laat betrokken: het advies komt als er niets meer te veranderen valt. Bij de Belastingdienst/FSV werd de FG pas ruim een jaar na de DPIA om advies gevraagd — EUR 450.000 van de totale boete van EUR 3,7 miljoen. EDPB CEF 2023: mediaan 22,5% van de organisaties betrekt de FG "altijd".

#### Ontwerpbesluiten

| Fout | Trede | Maatregel |
|---|---|---|
| DPIA-toets wordt overgeslagen | T2 + T3 | De toets is geen apart formulier maar een afgeleid onderdeel van de registerregel. Geen registerregel bereikt "Vastgesteld" zonder afgeronde toets. |
| Ten onrechte "niet nodig" bij een verplichte verwerking | T1 | Staat de verwerking op de AP-lijst van verplichte DPIA's, dan is de uitkomst "niet nodig" niet selecteerbaar. De beslisboom eindigt daar. |
| Ten onrechte "niet nodig" bij twee of meer EDPB-criteria | T3 | Bij twee of meer aangevinkte criteria uit de EDPB-richtsnoeren is "niet nodig" geblokkeerd. Bij precies één criterium is een gemotiveerd oordeel mogelijk, met verplichte motivering én tweede persoon. |
| Criteria niet gekend | T4 | De negen EDPB-criteria en de AP-lijst staan náást het veld, met een voorbeeld per criterium — niet in een helppagina. |
| DPIA achteraf | T3 + T5 | "Beoogde startdatum verwerking" en "datum toets" zijn twee verplichte, aparte velden. Ligt de toets ná de start, dan is dat een permanent dossierkenmerk (DPIA-03) dat niet weg te klikken is en in elke rapportage terugkomt. |
| FG te laat betrokken | T2 + T3 | Bij het openen van een DPIA wordt automatisch het adviesverzoek aan de FG aangemaakt, met een deadline die is teruggerekend vanaf de beoogde startdatum. Het DPIA-dossier bereikt "Afgerond" niet zonder geregistreerd FG-advies met datum. |
| FG-advies niet opgevolgd, onzichtbaar | T3 + T5 | Afwijken van het FG-advies vereist drie verplichte velden: de afwijking, de motivering en de besluitnemer. Die combinatie verschijnt automatisch in de eerstvolgende directierapportage. |
| Hoog restrisico zonder raadpleging AP | T3 | Restrisico "hoog" blokkeert de afronding tot een voorafgaande raadpleging (art. 36) is geregistreerd. |
| Standpunt betrokkenen vergeten | T3 | Art. 35(9): gevraagd, of gemotiveerd achterwege gelaten. Leeg laten kan niet. |

#### De beslisboom (regelgebaseerd, versiebeheerd)

```
1. Staat de verwerking op de AP-lijst van verplichte DPIA's?
   ja  → DPIA VERPLICHT. Einde. (uitkomst "niet nodig" bestaat niet)
   nee → 2

2. Valt de verwerking onder art. 35(3)(a), (b) of (c)?
   (systematische en uitgebreide beoordeling met rechtsgevolg;
    grootschalige bijzondere of strafrechtelijke gegevens;
    stelselmatige grootschalige observatie van openbaar toegankelijke ruimte)
   ja  → DPIA VERPLICHT. Einde.
   nee → 3

3. Hoeveel van de negen EDPB-criteria zijn van toepassing?
   (evaluatie/scoretoekenning; geautomatiseerde besluitvorming met rechtsgevolg;
    stelselmatige monitoring; gevoelige gegevens of gegevens van zeer persoonlijke
    aard; grootschalige verwerking; matching of samenvoeging van datasets;
    kwetsbare betrokkenen; innovatief gebruik van technologie; belemmering van
    een recht, dienst of overeenkomst)
   ≥ 2 → DPIA VERPLICHT. Einde.
   = 1 → 4
   = 0 → 5

4. Gemotiveerd oordeel vereist. Verplichte motivering + tweede persoon.
   Uitkomst "niet nodig" wordt vastgelegd met de gekozen criteria en de
   motivering, en herleeft automatisch bij elke wijziging van de verwerking.

5. Geen DPIA verplicht. Uitkomst wordt vastgelegd met datum en versie van de
   beslisboom. Herbeoordeling bij wijziging, en in elk geval na drie jaar.
```

De versie van de beslisboom wordt bij de uitkomst opgeslagen. Verandert de boom, dan toont de impactanalyse welke eerdere uitkomsten mogelijk anders uitvallen (§4.6).

#### Restrisico en vangnet

Een verwerking die niemand aanmeldt, wordt ook niet getoetst. Vangnet: REG-10 (systeem zonder registerregel), de leveranciers- en contractconfrontatie in de nulmeting, en de vaste ingang "nieuwe verwerking aanmelden" die als taakgerichte startactie voor proceseigenaren beschikbaar is.

---

### 2.3 Een datalek melden binnen 72 uur

Dit is het proces met de hoogste inzet en de meeste gedocumenteerde faalwijzen. Het krijgt daarom de dikste lagen.

#### Wat er misgaat

| Faalwijze | Bewijs uit de praktijk |
|---|---|
| 72-uursklok te laat gestart: geteld vanaf afronding onderzoek of vanaf het moment dat de FG het hoorde | AP-boete Booking.com EUR 475.000 — medewerkers wisten het op 13 januari, melding op 7 februari, 22 dagen te laat |
| "Geen bewijs van exfiltratie, dus geen lek" | AP Datalekkenrapportage 2023 (Nebu); risico slechts in 30-34% van dit soort aanvallen als hoog ingeschat |
| "Alleen contactgegevens, dus laag risico" | AddComm: 16 organisaties informeerden na een gesprek met de AP alsnog 82.893 mensen |
| "We willen geen onnodige onrust" | AP, standaardargument in het Nebu-dossier |
| Helemaal niet gemeld omdat het klein leek | AP-boete PVV Overijssel EUR 7.500 (basisbedrag EUR 525.000) voor 101 e-mailadressen |
| Alles melden "voor de zekerheid" | ICO: circa een derde van de telefonische meldingen haalde de drempel niet; DPC 2025: 55% laag of geen risico |
| Bulkmelding als verzamelbak | AP: bulkmeldingen van 5.480 (2023) naar 14.067 (2024) |
| Melden bij de verkeerde toezichthouder | EDPB Guidelines 9/2022 par. 66-69 |
| Beschikbaarheidsincident niet als datalek herkend | AP-categorie "tijdelijk niet beschikbaar door storing"; EDPB: verloren USB-stick is meldplichtig |
| Art. 34-beoordeling blijft liggen na de melding | klassieke post-completion error |
| Intern register niet gebruikt om te leren | AP Datalekkenrapportage 2024, art. 33 lid 5 |

#### Ontwerpbesluiten

| Fout | Trede | Maatregel |
|---|---|---|
| Klok te laat gestart | T1 | Twee gescheiden tijdstempels: **kennisnamemoment** (invoerbaar, met "door wie" en "via welk kanaal") en **registratiemoment** (systeemgegeven, niet invoerbaar). De klok loopt op kennisname. |
| Dagenlang intern uitzoeken vóór registratie | T1 + T3 | Intakeformulier van 60 seconden dat elke medewerker kan invullen. Wie het als eerste weet, registreert. Een gat groter dan vier uur tussen kennisname en registratie vereist een toelichting (LEK-03) en verschijnt in de rapportage. |
| Beschikbaarheidsincident niet herkend | T1 | De eerste beslisboomvraag heeft drie gelijkwaardige takken: vertrouwelijkheid, integriteit, **beschikbaarheid** — met voorbeelden (storing, versleuteling zonder exfiltratie, verloren back-up, kwijtgeraakte gegevensdrager). De beschikbaarheidstak is niet over te slaan. |
| "Geen bewijs = geen lek" | T1 | Zie §1.2 T1: de conclusie is geen selecteerbare uitkomst. Bij "niet uit te sluiten" rekent de weging met het ernstigste scenario. |
| Schaal genegeerd | T1 + T3 | Het aantal betrokkenen is een **aparte as** in de risicoweging, niet een van de factoren. De combinatie "gewone contactgegevens × grote schaal" kan niet op "laag" uitkomen zonder tegenspraak en tweede persoon (LEK-06). |
| Reputatieargument bij art. 34 | T1 | Alleen de drie uitzonderingen van art. 34(3), elk met verplichte onderbouwing. |
| Art. 34 vergeten | T2 + T3 | Verzending van de AP-melding maakt automatisch de art. 34-taak aan, met eigen klok ("onverwijld"). Het dossier bereikt "Afgesloten" niet zonder besluit. |
| Besluit "niet melden" zonder weerwoord | T3 | Dit is de gevaarlijkste beslissing in het proces en krijgt de zwaarste barrière: tweede persoon met complementaire controle (§3.10), of bij een eenpersoons-FG een afgedwongen afkoelperiode. |
| Onnodig melden | T3 + T5 | De drempelbeoordeling gaat verplicht vooraf aan de meldknop. "Uit voorzichtigheid melden ondanks uitkomst laag" bestaat als expliciete, gelabelde keuze — het is rechtmatig, maar het wordt zichtbaar en telbaar in plaats van het register te vervuilen. |
| Bulkmelding verkeerd toegepast | T1 | Zie §1.2 T1. Bovendien: elk incident in een bulkgroep houdt zijn **eigen** ontdekkingsmoment en eigen klok. De groep verschuift geen individuele termijn (LEK-10, LEK-11). |
| Verkeerde toezichthouder | T2 | De leidende toezichthouder is per entiteit vooraf vastgelegd met motivering (hoofdvestiging). De tool leidt de ontvanger af en toont die met de vastleggingsdatum. Afwijken vereist motivering (LEK-14). |
| Melding met fouten verstuurd | Laag 5 | Time-out van 60 seconden met itemsgewijze bevestiging vóór verzending (§3.7). |
| Herhaalpatroon onzichtbaar | T5 | Afsluiten vereist een oorzaakcategorie. LEK-13 rapporteert dezelfde oorzaak meer dan drie keer per kwartaal in hetzelfde proces. |

#### De beslisboom melden ja/nee, en aan wie

**Fase A — Is het een inbreuk in verband met persoonsgegevens?**

```
A1. Is er een inbreuk op de beveiliging?
    ▸ vertrouwelijkheid (onbevoegde kennisname of verstrekking)
    ▸ integriteit (ongeoorloofde wijziging)
    ▸ beschikbaarheid (verlies, vernietiging, tijdelijke ontoegankelijkheid)
    Elk van de drie is een aparte vraag met ja/nee. Alle drie moeten beantwoord.
    geen enkele ja → geen datalek; registreren als beveiligingsincident,
                     dossier blijft in het interne register (art. 33 lid 5)
    één of meer ja → A2

A2. Betreft het persoonsgegevens?
    nee → beveiligingsincident; als NIS2 van toepassing is, gaat het
          NIS2-spoor toch aan (§2.9)
    ja  → Fase B. De 72-uursklok loopt vanaf het kennisnamemoment.
```

**Fase B — Risicoweging (gestructureerd, elke factor apart gescoord)**

| As | Wat wordt gewogen | Bijzonderheid |
|---|---|---|
| B1 Aard van de inbreuk | vertrouwelijkheid / integriteit / beschikbaarheid, en of de gegevens versleuteld en de sleutel veilig waren | "versleuteld" vereist registratie van algoritme, sleutelbeheer en of de sleutel meelekte |
| B2 Aard en gevoeligheid | bijzondere gegevens, strafrechtelijke gegevens, BSN, financiële gegevens, inloggegevens, locatiegegevens, gewone gegevens | combinatie telt zwaarder dan de som |
| B3 Identificeerbaarheid | direct identificerend / pseudoniem / indirect via combinatie | |
| B4 Ernst van de gevolgen | identiteitsfraude, financiële schade, discriminatie, reputatieschade, fysieke onveiligheid, verlies van vertrouwelijkheid van beroepsgeheim | |
| B5 Bijzondere kenmerken betrokkenen | kinderen, patiënten, cliënten in kwetsbare posities, werknemers, bezwaarmakers, personen met een geheim adres | verhoogt automatisch |
| B6 Kenmerken verwerkingsverantwoordelijke | zorg, jeugd, veiligheid, schuldhulp, politieke organisatie | context maakt "gewone" gegevens gevoelig (vgl. PVV Overijssel: e-mailadres + politieke voorkeur) |
| B7 Aantal betrokkenen | **aparte as**, niet verrekend in B1-B6 | voorkomt de AddComm-fout |
| B8 Permanentie | terug te halen / niet terug te halen / al verder verspreid | een e-mail naar de verkeerde ontvanger is zelden terug te halen |

Uitkomst: **geen risico / risico / hoog risico**, met de motivering verplicht. De uitkomst is niet vrij te kiezen: de weging bepaalt een minimumuitkomst, waar de gebruiker alleen naar bóven van kan afwijken. Naar beneden afwijken vereist tegenspraak plus tweede persoon.

**Fase C — Melden aan de toezichthouder (art. 33)**

```
Uitkomst "geen risico"   → niet melden aan de AP.
                           Verplicht: motivering + tweede persoon of afkoelperiode.
                           Vastleggen in het interne register (art. 33 lid 5).
Uitkomst "risico"        → melden aan de AP binnen 72 uur na kennisname.
Uitkomst "hoog risico"   → melden aan de AP + Fase D.
```

**Fase D — Informeren van betrokkenen (art. 34)**

Deze fase ontstaat als een **aparte taak met eigen klok** zodra Fase C is afgerond, ook wanneer de AP-melding nog niet verzonden is. Zij is niet afleidbaar uit Fase C en wordt dus nooit stilzwijgend meegenomen.

```
Hoog risico voor de rechten en vrijheden?
  nee → vastleggen met motivering; klaar
  ja  → informeren, tenzij precies één van de drie uitzonderingen van
        art. 34(3) van toepassing is:
        (a) passende technische en organisatorische maatregelen die de
            gegevens onbegrijpelijk maken, en die zijn toegepast op de
            getroffen gegevens                → onderbouwing verplicht
        (b) maatregelen achteraf genomen die het hoge risico
            wegnemen                          → onderbouwing verplicht
        (c) onevenredige inspanning; dan een openbare mededeling
            of vergelijkbare maatregel        → alternatief verplicht vastleggen
        Andere redenen bestaan niet in de interface.
```

**Fase E — Aan wie**

| Spoor | Ontvanger | Afgeleid uit |
|---|---|---|
| AVG | leidende toezichthouder, vastgelegd per entiteit | hoofdvestiging, grensoverschrijdendheid |
| AVG, verwerker | de verwerkingsverantwoordelijke, onverwijld | rolbepaling in het register |
| NIS2 | CSIRT en/of bevoegde autoriteit | entiteitprofiel (§2.9) |
| Sectoraal | DNB, AFM, NZa, ACM, IGJ | sectorveld van de entiteit |
| Strafrechtelijk | aangifte politie | keuze, met vastlegging |

De tool toont de sporen naast elkaar met hun eigen klok. Eén spoor afhandelen sluit het dossier niet.

#### Statusmodel en escalatie

```
Gemeld (intake) ──▶ Geregistreerd ──▶ Beoordeeld ──▶ Besluit AP ──▶ [Gemeld bij AP]
                                                          │
                                                          ▼
                                                   Besluit art. 34 ──▶ [Betrokkenen geïnformeerd]
                                                          │
                                                          ▼
                                             Maatregelen ──▶ Evaluatie ──▶ Afgesloten
```

Escalatie loopt **naar een andere persoon, niet als herhaling naar dezelfde**:

| Moment | Actie |
|---|---|
| T+0 | registratie; eigenaar toegewezen |
| T+12u | geen risicobeoordeling: signaal aan de eigenaar (LEK-01) |
| T+24u | herinnering aan de behandelaar met de concrete deadline |
| T+48u | signaal aan de plaatsvervanger |
| T+60u | escalatie naar de directie, met de SBAR-samenvatting uit het dossier |
| T+72u | termijn verstreken: het dossier krijgt een permanent kenmerk "melding buiten de termijn" met verplichte toelichting; niet weg te klikken |

#### Restrisico en vangnet

Een verkeerde inschatting die door de gestructureerde weging heen komt en die de tweede persoon deelt (bevestigingsdruk), blijft mogelijk. Vangnet: de complementaire opzet van de vierogencontrole (§3.10), de meting "percentage besluiten 'niet melden' dat bij tegenspraak omdraaide" (§6) en de kwartaalanalyse van de risicoverdeling ten opzichte van de sectorcijfers.

---

### 2.4 Betrokkenenverzoek afhandelen binnen de termijn

#### Wat er misgaat

- Verzoek niet herkend omdat het via een ander kanaal binnenkomt (klachtenformulier, telefoon, een medewerker in de zorg, sociale media). De maandtermijn loopt vanaf ontvangst door de organisatie, niet vanaf ontvangst door de FG.
- Te laat of niet beantwoord; verlengingsbericht pas gestuurd als de termijn al verstreken is. AP 2025: ruim 13.500 klachten en tips (+75%), grootste categorie is het uitoefenen van rechten. ICO-berispingen over achterstanden van 287 en 120 verzoeken die jaren aanhielden.
- Scope te eng: metadata, e-mails en interne communicatie standaard uitgesloten; het verzoek wordt beperkt tot "wat het CRM kan exporteren" (EDPB CEF 2024, 1.185 controllers).
- Onnodige drempels: verplicht webformulier, standaard kopie identiteitsbewijs, of geld vragen.
- Geen gedocumenteerde procedure; kennis bij één of twee mensen (zeventien toezichthouders in CEF 2025).
- Account sluiten verward met wissen; back-ups standaard uitgesloten zonder motivering; pseudonimiseren gepresenteerd als verwijderd (CEF 2025, issues 5 tot en met 7).

#### Ontwerpbesluiten

| Fout | Trede | Maatregel |
|---|---|---|
| Verzoek niet herkend | T2 | Kanaalonafhankelijk intakeformulier van 60 seconden voor élke medewerker. Een korte regelgebaseerde vragenlijst ("vraagt iemand om zijn eigen gegevens, om wissen, om correctie, om bezwaar, om overdracht?") leidt het verzoektype af en opent het juiste dossier. De frontlijn hoeft geen artikelnummers te kennen. |
| Klok start te laat | T1 | Er is geen veld "datum ontvangst door de FG" dat de klok stuurt. Het enige klokveld is "datum ontvangst door de organisatie". |
| Termijn gemist | T2 | De maand wordt berekend als kalendermaand vanaf ontvangst (art. 12(3)), getoond als datum, tijd en tijdzone. Op dag 21 ontstaat automatisch de taak "verlengen of afronden". |
| Verlengingsbericht te laat | T3 | De status "Verlengd" is onbereikbaar zonder geregistreerde verzending van het verlengingsbericht binnen de eerste maand, én zonder een van de twee wettelijke gronden (complexiteit, aantal verzoeken). |
| Scope te eng | T2 + T3 | De **vindplaatsenlijst** wordt afgeleid uit het verwerkingsregister: alle systemen waarin die categorie betrokkenen voorkomt, plus de vaste categorieën e-mail, archief, logbestanden, back-up en papieren dossier. Elke vindplaats moet expliciet worden afgehandeld: gezocht en niets gevonden / gevonden en opgenomen / uitgesloten met wettelijke grond. Afronden met een openstaande vindplaats is geblokkeerd (BTR-04). |
| Onnodige identificatie | T3 | Aanvullende identificatie staat niet standaard aan. De tool vraagt eerst of de identiteit al via het kanaal vaststaat, en biedt aanvullende identificatie alleen met motivering per geval (BTR-05). |
| Kosten gevraagd | T1 | Er is geen kostenveld bij een eerste verzoek. Het veld verschijnt uitsluitend bij een aantoonbaar herhaald of kennelijk ongegrond verzoek, met verplichte onderbouwing. |
| Account sluiten = wissen | T1 | "Account gesloten" is geen geldige waarde in het veld "resultaat per vindplaats" bij een wisverzoek (BTR-07). |
| Back-up stilzwijgend uitgesloten | T3 | Uitsluiten van een back-up vereist een motivering én een geregistreerde hersteltermijn én een afspraak over wat er gebeurt als de back-up wordt teruggezet (BTR-08). |
| Pseudonimiseren als "verwijderd" | T3 | De keuze "geanonimiseerd" opent een verplichte toets op singling out, koppelbaarheid en afleidbaarheid, met tweede persoon. Zonder afgeronde toets valt het resultaat terug op "gepseudonimiseerd — nog persoonsgegevens" en blijft het verzoek open (BTR-09). |
| Geen procedure | T1 | De gedocumenteerde procedure is de applicatie. Elk dossier doorloopt dezelfde stappen; consistentie is een systeemeigenschap, geen persoonlijke eigenschap. |
| Afwijzing zonder rechtsmiddelen | T3 | Een afwijzing kan niet worden verzonden zonder verwijzing naar het klachtrecht bij de AP en de beroepsmogelijkheid (BTR-10). |

#### Termijn- en escalatieschema

| Dag | Gebeurtenis |
|---|---|
| 0 | ontvangst door de organisatie; klok start; eigenaar toegewezen binnen 1 werkdag |
| 3 | ontvangstbevestiging verzonden (taak) |
| 7 | vindplaatsenlijst compleet en uitgezet bij de systeemeigenaren |
| 14 | tussenstand; ontbrekende input wordt herinnerd bij de systeemeigenaar, niet bij de FG |
| 21 | verplichte beslissing: verlengen (met grond en bericht) of afronden |
| 25 | signaal aan de plaatsvervanger |
| 28 | escalatie naar de directie |
| 30 / einde kalendermaand | termijn; overschrijding is een permanent dossierkenmerk |

#### Restrisico en vangnet

Een verzoek dat nooit wordt geregistreerd, kan de tool niet bewaken. Vangnet buiten de tool: het intakeformulier moet zó kort zijn dat registreren goedkoper is dan doorverwijzen. Binnen de tool: BTR-11 meet het percentage binnen de maand per kwartaal, en de klachtenregistratie confronteert klachten met geregistreerde verzoeken — een klacht zonder voorafgaand geregistreerd verzoek is een sterk signaal van niet-herkenning.

---

### 2.5 Grondslag kiezen en onderbouwen

#### Wat er misgaat

- Grondslag ontbreekt of wordt achteraf "gerechtvaardigd belang" genoemd; het doel is niet vooraf specifiek omschreven. FSV: EUR 1 miljoen voor het ontbreken van een wettelijke grondslag, EUR 750.000 voor het niet vooraf specifiek omschrijven van het doel.
- Toestemming gekozen waar die niet vrij kan zijn: gezagsverhouding, overheidstaak.
- Systemen groeien organisch — een lijst, een signaalregister, een risicomodel — zonder dat iemand vooraf doel en grondslag vastlegt.

#### Ontwerpbesluiten

| Fout | Trede | Maatregel |
|---|---|---|
| Grondslag zonder doel | T1 | De grondslagkeuze verschijnt pas ná het doel en toont het doel erboven (semantische constraint). Doel en grondslag zijn één paar. |
| Doel niet specifiek | T2 + T3 | Het doel wordt opgebouwd met een sjabloonzin met invulplaatsen: "*[activiteit]* ten behoeve van *[proces]* voor *[categorie betrokkenen]*, zodat *[resultaat]*", náást een vrij motiveringsveld. Vier toetsvragen (welbepaald, uitdrukkelijk omschreven, gerechtvaardigd, verenigbaar) staan ernaast. |
| Gerechtvaardigd belang bij een overheidstaak | T1/T3 | Organisatietype "overheidsinstantie" gecombineerd met "verwerking in het kader van de publieke taak" maakt art. 6(1)(f) niet selecteerbaar (art. 6(1) slotzin). Voor niet-publieke verwerkingen van dezelfde organisatie blijft de keuze open, met motivering. |
| Gerechtvaardigd belang zonder toets | T3 | Keuze van art. 6(1)(f) opent een verplichte drietrapstoets: (1) het belang, concreet benoemd; (2) noodzaak, met subsidiariteit en proportionaliteit; (3) de afweging tegen de belangen, rechten en vrijheden van de betrokkene. Elk van de drie apart gemotiveerd. Zonder alle drie geen "Vastgesteld" (GRO-01). |
| Toestemming in een gezagsverhouding | T4 | Rol werkgever plus betrokkenen werknemers plus grondslag toestemming levert een tegenspraakblok met de alternatieven in beeld en een verplichte motivering (GRO-05). |
| Toestemming zonder intrekkingsroute of bewijs | T3 | Twee verplichte velden: hoe wordt toestemming verkregen en bewaard, en hoe kan zij even eenvoudig worden ingetrokken (GRO-03, GRO-04). |
| Wettelijke plicht zonder vindplaats | T3 | Keuze van art. 6(1)(c) of (e) vereist een gestructureerde verwijzing (wet + artikel) uit een beheerde lijst met de gangbare Nederlandse grondslagen, aanvulbaar via de voorstelroute (GRO-06). |
| Bijzondere gegevens zonder uitzondering | T2 + T3 | Aanvinken van een bijzondere categorie opent verplicht het art. 9(2)-blok en waar van toepassing de UAVG-uitzondering. Strafrechtelijke gegevens openen art. 10 en de UAVG-bepalingen (GRO-07, GRO-08). |
| BSN zonder grondslag voor het BSN zelf | T3 | Het BSN is een aparte categorie met een eigen verplichte wettelijke grondslag; de algemene grondslag volstaat niet (GRO-09). |
| Nieuw doel op een bestaande verwerking | T3 | Een doel toevoegen aan een vastgestelde verwerking opent verplicht de verenigbaarheidstoets van art. 6(4) (GRO-11). |
| Automatische besluitvorming ongemerkt | T2 + T3 | Aanvinken van geautomatiseerde besluitvorming met rechtsgevolg opent art. 22-grond, de betekenisvolle informatie over de logica, en de menselijke tussenkomstroute (GRO-12). Dit activeert tevens de DPIA-plicht. |
| Grondslagwijziging zonder gevolgenanalyse | T3 | Wijziging op een vastgestelde verwerking vereist een nieuwe versie, tweede persoon en de vraag welke verstrekkingen en afgeleide verwerkingen op de oude grondslag steunden. |

#### Restrisico en vangnet

De keuze zelf blijft een oordeel. Vangnet: de motivering is verplicht en wordt later het verantwoordingsstuk; de goedkeuring van de proceseigenaar; en de bibliotheek (§4.5) die per verwerkingstype de gangbare grondslag als vertrekpunt toont, waardoor afwijking een bewuste, gemotiveerde handeling wordt in plaats van een toevallige.

---

### 2.6 Bewaartermijn vastleggen en uitvoeren

#### Wat er misgaat

- Termijn staat in beleid of register, maar er is geen verwijderlabel en geen opschoningsopdracht; niets wordt ooit echt verwijderd. FSV: EUR 250.000 alleen voor te lang bewaren.
- Wettelijke bewaarplichten worden standaard op alles toegepast ("alles 7 jaar").
- EDPB CEF 2025 noemt als kernoorzaken het ontbreken van systematische dataclassificatie en van geautomatiseerde verwijderlabels in IT-systemen.

#### Ontwerpbesluiten

| Fout | Trede | Maatregel |
|---|---|---|
| Termijn als onbruikbare tekst | T1 | Vier velden: getal, eenheid, startgebeurtenis (beheerde lijst), grondslag van de termijn (wettelijk / organisatorisch / contractueel, met vindplaats). Geen vrij tekstveld. |
| "Zolang als nodig" / "onbepaald" | T1 | Niet selecteerbaar. Wel selecteerbaar: "nog te bepalen, uiterlijk [datum], eigenaar [rol]" — dat is een taak, geen eindwaarde (BEW-07). |
| Termijn vastgelegd maar nooit uitgevoerd | T2 + T5 | Elke termijn met een systeemkoppeling levert een **schoningsopdracht** op: periodiek, met eigenaar, met de berekende omvang ("naar schatting 12.400 records ouder dan de termijn"). Een termijn zonder schoningsopdracht is een eigen, zichtbare toestand (BEW-02). |
| Uitvoering niet aantoonbaar | T3 | De schoningsopdracht sluit alleen met aantal verwijderde records, uitvoerder en bewijs (BEW-08). |
| Wettelijke plicht te breed toegepast | T4 | De bibliotheek toont de reikwijdte van elke wettelijke termijn expliciet ("geldt voor de fiscaal relevante basisgegevens, niet voor het volledige dossier"). BEW-05 signaleert toepassing op een bredere set dan de plicht dekt. |
| Termijn langer dan gebruikelijk | T4 | BEW-04 signaleert afwijking naar boven van de bibliotheeknorm zonder motivering. |
| Back-up buiten beeld | T3 | Elk systeem heeft een verplicht veld "back-uphersteltermijn". Zolang dat leeg is, blokkeert het elk lopend wisverzoek op dat systeem (BEW-06). |
| Vernietiging per ongeluk | Laag 4/5 | Definitieve vernietiging is onomkeerbaar: tweede persoon, omvangsopgave vooraf, uitgestelde uitvoering met herstelvenster (§3.7). |

#### Van termijn naar uitvoering

```
Registerregel
  └─ bewaartermijn (getal, eenheid, startgebeurtenis, grondslag)
       └─ per gekoppeld systeem: verwijderregel
            └─ periodieke schoningsopdracht (eigenaar, frequentie, deadline)
                 └─ uitvoering (aantal, uitvoerder, bewijs, datum)
                      └─ bevestiging → volgende opdracht wordt ingepland
```

De tool voert zelf niets uit in andere systemen; zij maakt de opdracht, de eigenaar, de termijn en het bewijs afdwingbaar. Dat is precies het gat dat CEF 2025 signaleert.

#### Restrisico en vangnet

De eigenaar kan de schoningsopdracht afvinken zonder haar uit te voeren. Vangnet: het bewijsveld is verplicht en het aantal moet plausibel zijn ten opzichte van de vooraf berekende omvang; afwijkingen groter dan een ingestelde marge worden een rapportregel; en het onderwerp keert terug in de directierapportage.

---

### 2.7 Doorgifte buiten de EER

#### Wat er misgaat

- Alleen naar de hostinglocatie gekeken; inzage vanuit een derde land door supportmedewerkers niet als doorgifte herkend; subverwerkerslijst niet gecontroleerd. AP: raadplegen is ook verwerken, en "nee, tenzij" geldt.
- Uber: EUR 290 miljoen wegens overtreding van art. 44 nadat vanaf augustus 2021 werd gestopt met standaardcontractbepalingen.

#### Ontwerpbesluiten

| Fout | Trede | Maatregel |
|---|---|---|
| "Onze data staat in Amsterdam" als enige antwoord | T1 | Drie aparte, verplichte vragen per systeem: (a) waar staan de gegevens; (b) vanuit welke landen is toegang mogelijk — support, beheer, ontwikkeling, incidentafhandeling; (c) welke subverwerkers zijn er en waar zitten die. Vraag (b) heeft geen standaardwaarde en is niet over te slaan (EER-08). |
| "Onbekend" bij toegangsland | T3 | "Onbekend" is een geldige tussenwaarde in concept, maar blokkeert vaststelling en maakt automatisch een uitvraagtaak bij de leverancier aan (EER-02). |
| Doorgifte zonder waarborg | T2 + T3 | Elk land buiten de EER in een van de drie velden opent het hoofdstuk V-blok en blokkeert vaststelling zonder waarborg (EER-01). |
| Waarborg als losse aanvinking | T3 | Per waarborgtype verplichte velden: adequaatheidsbesluit (land, besluitdatum, reikwijdte, en bij een certificeringsstelsel of de ontvanger is aangesloten en voor welke gegevenscategorieën); standaardbepalingen (module 1 t/m 4, versie, ondertekendatum, ingevulde bijlagen); bindende bedrijfsvoorschriften (goedkeuringsdatum, toezichthouder); art. 49 (welke uitzondering, waarom incidenteel). |
| Transfer impact assessment vergeten | T2 + T3 | Keuze voor standaardbepalingen maakt de TIA automatisch tot een verplicht onderdeel; zonder afronding geen vaststelling (EER-03). |
| Art. 49 structureel gebruikt | T4 | EER-06: meer dan twee keer per jaar op dezelfde ontvanger is per definitie niet incidenteel; signaal met verplichte heroverweging. |
| Subverwerkerslijst veroudert | T2 + T5 | Per verwerker de datum van de laatst gecontroleerde lijst; jaarlijkse controletaak (VWO-09); wijziging in de lijst zet afhankelijke registerregels op "herziening nodig". |
| Waarborg vervalt door een juridische ontwikkeling | T2 | Eén beheerhandeling "juridische wijziging registreren" (adequaatheidsbesluit ingetrokken of geschorst, nieuwe versie van de standaardbepalingen) laat de tool automatisch alle afhankelijke regels blokkeren en de bijbehorende taken aanmaken (EER-07). Eén invoer, honderd correcties — precies het omgekeerde van de Uber-situatie. |

#### Restrisico en vangnet

Een leverancier die de toegangslanden onjuist opgeeft, kan de tool niet corrigeren. Vangnet: de opgave wordt met datum en bron vastgelegd, de VWO bevat de verplichting tot melding van wijzigingen, en de jaarlijkse controle is een taak met eigenaar. De onjuiste opgave wordt daarmee een aantoonbare tekortkoming van de leverancier in plaats van een blinde vlek van de organisatie.

---

### 2.8 Verwerkersovereenkomst sluiten en bewaken

#### Wat er misgaat

- VWO ontbreekt volledig terwijl er wel wordt verwerkt; de AP ontving overeenkomsten met een dagtekening ná haar eigen vragenbrief.
- VWO die alleen de AVG-tekst overschrijft — de AP noemt dat uitdrukkelijk onvoldoende.
- Verouderde bewerkersovereenkomsten die nog naar de Wbp verwijzen.
- Geen mapping van clausule naar wettelijke eis; de AP merkt op dat het vaak "niet eenvoudig waar te nemen" is waar een eis geregeld is.
- Onwerkbare meldtermijn: 48 of zelfs 72 uur aan de verwerker gegund, waardoor de verantwoordelijke per definitie te laat is.
- Geen of riskante exitafspraak: "verwerker wist alles" zonder teruggaveoptie.
- Register niet als vertrekpunt gebruikt; register, contracten en systemen zijn drie losse werelden zonder koppelsleutel.

#### Ontwerpbesluiten

| Fout | Trede | Maatregel |
|---|---|---|
| Drie losse werelden | T1 | De koppelsleutel zit in het datamodel: verwerking → systeem → leverancier → contract. Er is geen contractenlijst náást het register; er is één keten. |
| Verwerker zonder VWO onzichtbaar | T3 + T5 | Een leverancier aanmerken als verwerker zonder gekoppeld actief contract levert de toestand "verwerker zonder overeenkomst", die niet weg te klikken is, en blokkeert vaststelling van de afhankelijke registerregels (VWO-01). |
| VWO met een partij die zelf verantwoordelijke is | T2 | Een korte rolbeslisboom (wie bepaalt doel en middelen; verwerkt de partij voor eigen doeleinden; wordt gezamenlijk bepaald) leidt het contracttype af: verwerkersovereenkomst, art. 26-regeling, of geen van beide met een verstrekkingsgrondslag (VWO-11). |
| Geen mapping clausule → eis | T1 + T3 | De eisenlijst (art. 28(2), 28(3)(a) t/m (h), 28(4), 29, 32, 33(2)) is een eerste-klas gegeven. Per eis registreert de gebruiker de **vindplaats** in het contract (artikel- of paragraafnummer) én de **concrete invulling**. Een eis afvinken zonder vindplaats is onmogelijk (VWO-02). |
| Alleen de wettekst overgeschreven | T3 + T4 | Naast elke eis staat de wettekst, met de vraag "wat is hier concreet afgesproken?". Het invullingsveld is verplicht en mag niet gelijk zijn aan de wettekst; sterke gelijkenis met de standaardtekst uit de bibliotheek levert een signaal (VWO-03). |
| Onwerkbare meldtermijn | T3 + T2 | Eenheid uur; de tool rekent live mee: "verwerker binnen X uur → u houdt 72 − X = Y uur". Boven 24 uur een signaal met motivering; vanaf 48 uur geblokkeerd zonder tweede persoon (VWO-04). De sjabloonbibliotheek hanteert als standaard: onverwijld en uiterlijk binnen 24 uur. |
| Riskante exitafspraak | T3 | Keuze uit teruggave / verwijdering / teruggave gevolgd door verwijderbevestiging. "Alleen verwijderen" vereist motivering én een veld "hoe wordt de continuïteit voor betrokkenen geborgd". Verplicht: exportformaat, termijn, kosten, bewijs van vernietiging (VWO-05). |
| Verouderd contract | T3/T4 | Documenttype "bewerkersovereenkomst" of een verwijzing naar ingetrokken wetgeving levert een signaal met een herzieningsdatum; na die datum blokkeert het (VWO-06). |
| Contract leeft in de mailbox van de FG | T1 + T3 | Elke VWO heeft een contracteigenaar buiten de FG, een einddatum, een opzegtermijn en een herzieningsdatum. Zonder contracteigenaar geen actieve status (VWO-08). Dit is AP-aanbeveling 3 als datamodelbeperking. |
| Subverwerker zonder afspraak | T3 | Toevoeging van een subverwerker zonder geregistreerde toestemmings- of bezwaarprocedure blokkeert (VWO-10). |
| Contract ondertekend na aanvang | T5 | VWO-13 rapporteert een ondertekendatum die na de startdatum van de verwerking ligt — bewust een rapportregel, want de fout is dan al gemaakt; het doel is het patroon zichtbaar maken. |

#### Restrisico en vangnet

Een contract kan formeel compleet zijn en materieel zwak. Vangnet: het invullingsveld per eis maakt de zwakte leesbaar in plaats van verborgen; de jaarlijkse contractreview is een taak met eigenaar; en LEK-16 rapporteert leveranciers die bij een echt incident de afgesproken meldtermijn overschreden — de enige harde test van het contract.

---

### 2.9 NIS2-meldketen (24 uur / 72 uur / 1 maand)

#### Wat er misgaat

- Eén gebeurtenis activeert twee of drie sporen met verschillende klokken, verschillende ontvangers en verschillende inhoudseisen; in de praktijk verdringt het ene spoor het andere.
- De 24-uursklok wordt behandeld als een verkorte 72-uursmelding, waardoor men "wacht tot we meer weten" en de vroegtijdige waarschuwing te laat komt.
- Toepasselijkheid (sector, omvang, essentieel of belangrijk) wordt per incident opnieuw ter discussie gesteld.
- Het eindrapport blijft liggen omdat het incident nog loopt.

#### Ontwerpbesluiten

| Fout | Trede | Maatregel |
|---|---|---|
| Toepasselijkheid per incident opnieuw beoordeeld | T1 + T2 | Sector, omvang en classificatie worden één keer per entiteit vastgelegd met motivering en datum. De tool leidt daarna af; de vraag komt niet terug (NIS-06). |
| Sporen verdringen elkaar | T2 | Eén incident, meerdere sporen, naast elkaar in één tijdlijn: AVG-spoor, NIS2-spoor, sectoraal spoor. Elk met eigen klok, ontvanger, inhoudseis en status. Het dossier sluit pas als alle actieve sporen gesloten zijn. |
| Significantie niet beoordeeld | T3 | Verplichte beslisboom over de wettelijke criteria (ernstige operationele verstoring of financiële verliezen; aanzienlijke materiële of immateriële schade aan anderen) plus de sectorale drempels, elk gescoord, met motivering (NIS-05). |
| Wachten met de 24-uursmelding | T1 + T2 | Het 24-uursbericht is bewust een **kort** formulier met vier vragen: vermoeden van onrechtmatig of kwaadwillig handelen, mogelijke grensoverschrijdende gevolgen, korte omschrijving, contactpersoon. Het scherm zegt expliciet dat volledigheid niet vereist is. De knop is vanaf T+0 actief; het 72-uursformulier wordt pas daarna aangeboden (volgordecontrole). |
| Klokverwarring | T2 | Eén tijdlijnbalk met alle markers, elk met datum, tijd en tijdzone: NIS2 T+24u, NIS2 T+72u, NIS2 eindrapport T+1 maand, AVG T+72u. De AVG- en NIS2-kennisname kunnen op verschillende momenten liggen; het verschil wordt getoond en bij afwijking zonder toelichting gesignaleerd (NIS-07). |
| Eindrapport blijft liggen | T2 + T3 | Op T+1 maand controleert de tool de incidentstatus. Loopt het incident nog, dan ontstaat automatisch een **voortgangsrapport**-taak en verschuift de eindrapportdeadline naar één maand na afronding. Is het afgerond, dan blokkeert de afsluiting zonder de vier verplichte onderdelen: gedetailleerde beschrijving, aard van de dreiging en grondoorzaak, toegepaste en lopende mitigerende maatregelen, en grensoverschrijdende gevolgen (NIS-03, NIS-04). |

#### Tijdlijn

```
T+0      kennisname (NIS2)          ─┬─ klok NIS2 start
T+1u     24-uursbericht beschikbaar  │
T+18u    escalatie plaatsvervanger   │
T+24u    VROEGTIJDIGE WAARSCHUWING  ◀┘   verplicht
T+48u    escalatie directie
T+72u    INCIDENTMELDING            ◀    verplicht, inhoudelijk
T+…      tussentijds rapport op verzoek van het CSIRT
T+1md    EINDRAPPORT   of, bij een lopend incident, VOORTGANGSRAPPORT
         (eindrapport dan binnen 1 maand na afronding)
```

Parallel loopt, waar persoonsgegevens betrokken zijn, de AVG-klok vanaf het eigen kennisnamemoment met haar eigen 72-uursmarker en de art. 34-taak.

#### Restrisico en vangnet

De significantiebeoordeling blijft een oordeel onder tijdsdruk. Vangnet: de beslisboom legt de gescoorde criteria vast (verantwoording achteraf), de 24-uursmelding is bewust laagdrempelig zodat de veilige route ook de goedkoopste is, en NIS-01 escaleert op T+18u naar een andere persoon.

---

### 2.10 Overdracht van dossiers bij personeelswissel

#### Wat er misgaat

- Geen plaatsvervanger of achtervang. EDPB CEF 2023: publieke sector 36,4% tegenover private sector 56,3%.
- Lopende termijnen worden niet overgedragen; verzoeken staan in een persoonlijke mailbox; achterstanden houden jaren aan.
- Compliance-administratie leeft in persoonlijke bestanden: register in een eigen Excel, DPIA's als concept in de mailbox, afspraken met leveranciers alleen mondeling.
- Nieuwe FG erft documentatie die niet bij de werkelijkheid past en toetst dat niet.
- Vertrekkende medewerker neemt persoonsgegevens mee; toegangsrechten van vertrokken medewerkers en beëindigde leveranciers worden niet ingetrokken.
- FG niet aangemeld bij de toezichthouder of contactgegevens niet gepubliceerd — een eenmalige handeling die bij wisseling vergeten wordt.
- Rolconflict en te dun gespreide externe FG.

#### Ontwerpbesluiten

| Fout | Trede | Maatregel |
|---|---|---|
| Administratie in persoonlijke bestanden | T1 | Er is geen ondersteund werkproces waarin een registerregel, DPIA, toezegging of leveranciersafspraak buiten de tool bestaat. Bijlagen landen in de versleutelde opslag, niet als verwijzing naar een netwerkpad. |
| Eigenaarschap aan een persoon | T1 | Alle eigenaarschap hangt aan een **rol** met een persoonsbezetting en een periode. Bij vervanging verhuizen alle taken automatisch mee. Een taak zonder rol kan niet bestaan (ORG-10). |
| Geen achtervang | T3 + T5 | Elke rol heeft een verplichte plaatsvervanger met ingangsdatum. Ontbreekt die, dan is dat een permanente rapportregel en blokkeert het bij het vastleggen van afwezigheid (ORG-03). |
| Termijnen lopen door in de vakantie | T3 | **Afwezigheidsmodus**: bij het vastleggen van afwezigheid toont de tool alle dossiers met een termijn die in die periode valt en dwingt per dossier een keuze: overdragen aan [rol/persoon], afronden vóór vertrek, of expliciet laten liggen met motivering. Geen enkele termijn gaat onbeheerd de afwezigheid in (ORG-07). |
| Overdracht die niet aankomt | T3 | **Closed-loop overdracht** (§3.10): de ontvanger reproduceert de kritieke waarden actief uit keuzelijsten — aard, aantal betrokkenen, eerstvolgende deadline, openstaande toezegging. Afwijkingen tussen zender en ontvanger worden aan beiden getoond. Het dossier komt pas op naam van de ontvanger na de bevestigde lus (ORG-08). |
| Geërfd register wordt niet getoetst | T1 + T5 | **Overnamedossier** met vier verplichte confrontaties: register tegen de systemenlijst; register tegen de leveranciers- en contractenlijst; register tegen de autorisatiebronnen; lopende termijnen tegen de dossiers. Tot verificatie draagt elke geërfde regel het kenmerk "geërfd, niet geverifieerd" (REG-09), dat niet handmatig te wissen is. |
| Toegangsrechten blijven staan | T3 + T5 | Exit-checklist per vertrekkende medewerker met de gekoppelde autorisaties, accounts, sleutels, toegang bij leveranciers en openstaande dossiers. Afsluiten met openstaande items vereist motivering; openstaande intrekkingen escaleren (ORG-09). |
| Gegevens meegenomen bij vertrek | T1 + T5 | Binnen de tool: export van een volledig dossier vereist een tweede persoon, dubbele invoer van de bestemming en een exportstempel; alle exports staan in het auditspoor. |
| FG niet aangemeld / gegevens niet gepubliceerd | T2 + T3 | Per entiteit twee kleine, verplichte objecten: "FG aangemeld bij de AP op [datum], referentie [nr]" en "contactgegevens gepubliceerd op [plaats], laatst gecontroleerd op [datum]". Bij een rolwissel worden beide automatisch ongeldig en ontstaan er twee taken met deadline (ORG-01, ORG-02). Dit is precies de handeling die bij wisseling vergeten wordt. |
| Rolconflict | T3 + T5 | Bij het vastleggen van de FG-rol worden de overige functies van dezelfde persoon geregistreerd. Combinaties uit een regelgebaseerde conflictlijst (IT-manager, security officer, hoofd HR, hoofd marketing, tekenbevoegde voor contracten, FG bij zowel de verwerker als de verantwoordelijke) blokkeren zonder vastgelegde motivering én melding aan de directie (ORG-04). |
| Onvoldoende middelen onzichtbaar | T5 | Vastlegging van uren per week, opleidingsuren per jaar, budget en beheer daarvan, plaatsvervanger en rapportagefrequentie aan de hoogste leiding; jaarlijks in de directierapportage, afgezet tegen de CEF-referentiecijfers (ORG-05, ORG-06). |
| Externe FG te dun gespreid | T5 | Aantal klanten en contracturen per klant worden vastgelegd; onderschrijding van de vastgelegde ondergrens is een rapportregel (ORG-11). |
| Toezegging aan de toezichthouder verdwijnt | T3 | Elke toezegging is een object met eigenaar, deadline en bewijs van uitvoering; zonder eigenaar en einddatum kan zij niet bestaan (ORG-12). |

#### Restrisico en vangnet

De nulmeting kost tijd en kan half worden gedaan. Vangnet: het kenmerk "geërfd, niet geverifieerd" is niet te wissen zonder de confrontatie, staat in elke export en elke rapportage, en veroudert zichtbaar (REG-09 na 90 dagen). Onvolledige overname wordt daarmee een zichtbare toestand in plaats van een stilzwijgende aanname.

---

## 3. Concrete interactiepatronen

### 3.1 Begeleide invoer in stappen in plaats van één groot formulier

**Gedrag.** Elk intakeproces is opgedeeld in vier tot zes stappen met één duidelijke vraagstelling per stap. Voorbeeld register-intake:

| Stap | Inhoud | Aantal zichtbare velden |
|---|---|---|
| 1. Identificatie | naam van de verwerking, proces, proceseigenaar, rol (verantwoordelijke/verwerker/gezamenlijk) | 5 |
| 2. Doel en grondslag | doel(en) met gekoppelde grondslag, motivering | 3 per doel |
| 3. Gegevens en betrokkenen | categorieën betrokkenen, categorieën gegevens, bijzondere gegevens, bron | 4-6 |
| 4. Systemen en ontvangers | systemen, ontvangers, verwerkers, doorgifte | 4-8 |
| 5. Bewaren en beveiligen | bewaartermijn, maatregelen | 4-5 |
| 6. Afronden | afgeleide verplichtingen, controle-uitkomsten, indienen ter beoordeling | overzicht |

**Regels voor het schermgedrag:**

- De voortgangsindicator toont stap én volledigheid ("Stap 3 van 6 — 11 van 14 verplichte onderdelen compleet"), niet alleen de stap.
- Vrij heen en weer springen is altijd toegestaan; de wizard is een indeling, geen gevangenis.
- Elk veld slaat op bij het verlaten; de stap wordt niet als eenheid opgeslagen.
- **Hervattingsanker** bij terugkeer, permanent bovenaan: *"Je was gebleven bij stap 3 — Gegevens en betrokkenen. Laatst gewijzigd vandaag 11:42 door jou. Er zijn sindsdien geen wijzigingen door anderen."*
- Permanente contextbalk met dossiernummer, naam, status en een onderscheidend kenmerk (proceseigenaar of systeem), visueel dominant — niet als breadcrumb. Dit is de maatregel tegen capture errors bij gelijkende dossiers.
- Progressive disclosure geldt voor optionele diepte en conditionele relevantie, **nooit** voor een wettelijk minimum. Een verplicht art. 30-element mag nooit achter een "geavanceerd"-uitklapper zitten; dat zou van een ontwerptechniek een latente conditie maken.

### 3.2 Keuzelijsten en sjablonen in plaats van vrije tekst

**Vuistregel: gestructureerd voor het feit, vrije tekst voor de motivering — en altijd allebei, nooit het één in plaats van het ander.**

| Aantal opties | Besturingselement | Toelichting |
|---|---|---|
| 2-6 | radio's of segmented control | alles zichtbaar, geen klik nodig om te zien wat er is |
| 7-25 | keuzelijst met zoekfilter | |
| > 25 | type-ahead op een **beheerde** lijst | |

**De kritieke regel bij type-ahead:** vrije invoer maakt nooit stilzwijgend een nieuwe waarde aan. Dat is mechanisch de belangrijkste oorzaak van registers die uit elkaar lopen. In plaats daarvan: *"Staat er niet bij? → Nieuwe waarde voorstellen"*, met een duplicaatcontrole op fuzzy-match vóór aanmaak en, voor beheerde taxonomieën, goedkeuring door een beheerder.

Voorbeeld van het schermgedrag bij een nieuwe leverancier:

```
Leverancier          [ Acme Hosting B.V|                    ]
                     ┌──────────────────────────────────────┐
                     │ ACME Hosting BV      3 verwerkingen  │
                     │ Acme Support B.V.    1 verwerking    │
                     ├──────────────────────────────────────┤
                     │ Lijkt sterk op "ACME Hosting BV".    │
                     │ [ Deze gebruiken ]  [ Toch nieuw ▸ ] │
                     └──────────────────────────────────────┘
```

"Toch nieuw" leidt naar een klein formulier met KvK-nummer (formaatcontrole), vestigingsland en rol — niet naar een leeg tekstveld.

**Gestructureerd in `dpo-fg-tool`:** doelen, grondslagen, art. 9-uitzonderingen, categorieën betrokkenen, categorieën gegevens, ontvangers, landen, waarborgen, bewaartermijnen (getal + eenheid + startgebeurtenis), maatregelen, incidenttypen, oorzaakcategorieën, risiconiveaus, statussen, verzoektypen, vindplaatsen.

**Vrije tekst uitsluitend voor:** motivering bij een oordeel, feitelijke omschrijving van een incident, contextuele toelichting bij een afwijking — altijd naast het gestructureerde veld.

**Koppelen, niet overtypen.** Elke keer dat een gebruiker iets overtypt dat het systeem al weet, is dat een ontwerpfout met een meetbare foutkans. Uitgangscijfers uit de literatuur: 1-4% veldfouten in de dagelijkse praktijk, circa 2,5% bij gestructureerde numerieke velden tegenover circa 4,8% bij beschrijvende vrije tekst.

**Dubbele invoer** wordt spaarzaam en gericht ingezet — alleen waar een typefout onherstelbaar of onzichtbaar is: herstelcodes en sleutelmateriaal, het adres van een externe exportbestemming, een extern referentienummer dat later niet te verifiëren is. Overal elders werkt **bevestiging door herkenning** beter en goedkoper: toon de ingevoerde waarde terugvertaald in betekenis.

```
Bewaartermijn   [ 7 ] [ jaar ▾ ] na [ einde dienstverband ▾ ]
                Grondslag: [ wettelijk — art. 52 AWR ▾ ]

                ▸ Voor het oudste huidige record betekent dit: verwijderen
                  op 3 maart 2031.
                ▸ Deze termijn geldt voor de fiscaal relevante basisgegevens,
                  niet voor het volledige personeelsdossier.
```

De gebruiker beoordeelt de **betekenis**, niet de vorm. Herkenning boven herinnering.

### 3.3 Afhankelijke velden

Logische constraints zijn in formulierwerk het krachtigst en het goedkoopst: ze halveren het zichtbare formulier en elimineren hele klassen van irrelevante fouten.

| Antwoord | Wat verschijnt | Wat verdwijnt |
|---|---|---|
| rol = verwerker | art. 30(2)-velden, verwerkingsverantwoordelijke(n) | art. 30(1)-specifieke velden |
| rol = gezamenlijk verantwoordelijk | art. 26-regeling, kern van de regeling, contactpunt | — |
| bijzondere gegevens aangevinkt | art. 9(2)-uitzondering, extra beveiligingsvragen, DPIA-criterium aangezet | — |
| strafrechtelijke gegevens | art. 10 en UAVG-grondslag | — |
| doorgifte = nee | — | volledig hoofdstuk V-blok |
| land buiten EER (opslag, toegang of subverwerker) | waarborgkeuze, TIA-blok | — |
| waarborg = standaardbepalingen | module, versie, bijlagen, TIA | adequaatheidsvelden |
| grondslag = toestemming | verkrijging, bewijs, intrekkingsroute | — |
| grondslag = gerechtvaardigd belang | drietrapstoets | — |
| grondslag = wettelijke plicht / publieke taak | wet + artikel uit beheerde lijst | — |
| geautomatiseerde besluitvorming = ja | art. 22-grond, logica-uitleg, menselijke tussenkomst; DPIA verplicht | — |
| verwerker betrokken = ja | contractkoppeling, meldtermijn, subverwerkers | — |
| incidenttype = beschikbaarheid | hersteltijd, gevolgen van onbeschikbaarheid, continuïteitsmaatregelen | exfiltratievragen worden secundair, niet verwijderd |

**Twee harde regels bij verdwijnende velden:**

1. Een veld dat verdwijnt, wordt **niet stilzwijgend meegestuurd**. Zijn waarde blijft bewaard voor het geval het antwoord terugverandert, maar telt niet mee in volledigheid, export of controle zolang het verborgen is.
2. Het scherm meldt wat er is weggevallen: *"3 velden zijn niet meer van toepassing omdat u 'geen doorgifte' koos. Tonen."* Zonder die melding is een verdwenen verplichting niet te onderscheiden van een vergeten verplichting.

### 3.4 Automatische afleiding van verplichtingen

Het patroon: **een antwoord is een gebeurtenis; een gebeurtenis maakt een verplichting; een verplichting is een taak met eigenaar en termijn.** Geen enkele wettelijke vervolgstap bestaat alleen in het hoofd van de gebruiker.

| Aanleiding | Afgeleide verplichting | Waar die landt |
|---|---|---|
| Bijzondere gegevens aangevinkt | art. 9(2)-uitzondering + DPIA-criterium + verzwaarde beveiligingsvragen | registerregel, blokkerend bij vaststellen |
| Twee of meer EDPB-criteria | volledige DPIA | eigen DPIA-dossier, gekoppeld |
| Verwerking op de AP-lijst | volledige DPIA, geen ontsnapping | idem |
| DPIA geopend | adviesverzoek aan de FG met teruggerekende deadline | werkvoorraad FG |
| DPIA met hoog restrisico | voorafgaande raadpleging AP (art. 36) | taak, blokkeert afronding |
| Land buiten de EER | waarborg + transfer impact assessment | registerregel + leveranciersdossier |
| Verwerker gekoppeld | verwerkersovereen| Verwerker gekoppeld | verwerkersovereenkomst met volledige art. 28(3)-mapping | leveranciersdossier, blokkeert vaststelling registerregel |
| Grondslag = toestemming | intrekkingsroute + bewijsbewaring + informatieplicht | registerregel |
| Grondslag = gerechtvaardigd belang | drietrapstoets | registerregel |
| Geautomatiseerde besluitvorming | art. 22-grond + logica-uitleg + menselijke tussenkomst + DPIA | registerregel + DPIA |
| Bewaartermijn vastgesteld | schoningsopdracht per gekoppeld systeem, periodiek, met eigenaar | takenlijst systeemeigenaar |
| Cameratoezicht aangevinkt | DPIA-criterium, kenbaarheidsmaatregel, OR-instemmingstoets | registerregel |
| Kinderen als categorie betrokkenen | art. 8-toets, verzwaarde informatieplicht, DPIA-criterium | registerregel |
| AP-melding verzonden | art. 34-besluit met eigen klok | datalekdossier |
| Besluit "niet melden" | vastlegging in het interne register (art. 33 lid 5) + tweede persoon | datalekdossier |
| Datalek afgesloten | oorzaakcategorie + maatregel met eigenaar en bewijs | maatregelenregister |
| NIS2-entiteit + significant incident | drie meldmomenten met eigen klokken | NIS2-spoor |
| Wisverzoek geregistreerd | vindplaatsenlijst uit het register + back-upvraag per systeem | verzoekdossier |
| Rolwissel FG vastgelegd | aanmelding bij de AP + publicatie contactgegevens, beide ongeldig verklaard | entiteitdossier, twee taken |
| Afwezigheid vastgelegd | overdrachtsbeslissing per dossier met termijn in die periode | overdrachtsscherm |
| Subverwerkerslijst gewijzigd | herziening van alle afhankelijke registerregels | registerregels op "herziening nodig" |

Het scherm maakt de afleiding **zichtbaar op het moment dat zij ontstaat**, niet pas in een takenlijst:

```
☑ Bijzondere persoonsgegevens: gezondheidsgegevens

  ▸ Hierdoor zijn twee dingen toegevoegd aan dit dossier:
    • Uitzonderingsgrond art. 9(2) — verplicht vóór vaststellen   [ Invullen ]
    • DPIA-criterium 4 "gevoelige gegevens" aangezet
      (1 van 9 criteria; bij 2 of meer is een DPIA verplicht)     [ Toets openen ]
```

### 3.5 Klokken en termijnen die zichzelf berekenen

**Eén geïsoleerde termijnmodule.** Alle wettelijke termijnen worden door één module berekend, met expliciete testgevallen per termijn, inclusief zomertijdovergang, schrikkeljaar, maandeinde (31 januari + 1 maand) en feestdagen waar een termijn in werkdagen loopt. Eén verkeerde rekenregel in de kern maakt élk dossier ongemerkt te laat; dit is de gevaarlijkste latente conditie in de hele applicatie.

| Termijn | Eenheid | Startpunt | Bijzonderheid |
|---|---|---|---|
| AVG art. 33 | 72 klokuren | kennisname door de organisatie | doorlopend, geen werkdagen |
| AVG art. 34 | "onverwijld" | vaststelling hoog risico | geen vaste eindtijd; wel een interne norm en escalatie |
| AVG art. 12(3) | 1 kalendermaand | ontvangst door de organisatie | verlenging met 2 maanden, bericht binnen de eerste maand |
| NIS2 vroegtijdige waarschuwing | 24 klokuren | kennisname significant incident | eigen kennisnamemoment |
| NIS2 incidentmelding | 72 klokuren | idem | |
| NIS2 eindrapport | 1 maand | melding, of afronding bij een lopend incident | voortgangsrapport als tussenstap |
| Herziening registerregel | 12 maanden | laatste vaststelling | signaal, geen wettelijke termijn |
| DPIA-herbeoordeling | 36 maanden of bij wijziging | afronding DPIA | |

**Schermregels:**

- **Altijd de afgeleide concrete deadline** met datum, tijd en tijdzone — nooit alleen "nog 2 dagen". Een abstract getal draagt geen betekenis en wordt geleerd genegeerd.
- De wettelijke basis van de klok is zichtbaar bij het aanwijzen van de teller: *"72 uur, art. 33 lid 1 AVG, vanaf kennisname 18 augustus 09:14 CEST"*.
- Deadlines worden opgeslagen als **absolute momenten**, niet als resterende tijd. Stroomuitval, herstart of een verzette systeemklok verschuift niets.
- De klokbron is niet de aanpasbare gebruikersklok. Afwijkingen tussen systeemklok en de monotone referentie worden vastgelegd en gerapporteerd.
- **Kleurcodering met een expliciete afspraak, en nooit permanent rood:**

| Toestand | Weergave | Betekenis |
|---|---|---|
| ruim binnen termijn | neutraal, geen accent | geen actie vandaag |
| binnen de laatste 50% | geaccentueerd, niet alarmerend | staat bovenaan de werkvoorraad |
| binnen de laatste 25% of escalatiedrempel | nadrukkelijk | escalatie naar de volgende persoon staat klaar |
| verstreken | apart, permanent kenmerk op het dossier | niet weg te klikken; verschijnt in de rapportage |

Een dashboard dat structureel alarmerend is, wordt behang. De norm is een dagelijks haalbare toestand "nul openstaande overschrijdingen"; als die toestand structureel onhaalbaar is, is dat een capaciteitsprobleem dat gerapporteerd hoort te worden, niet een kleur die harder moet.

- **Eén dagelijkse werkvoorraad**, gesorteerd op urgentie, in plaats van losse meldingen per dossier. Eén contactmoment.
- **Escalatie naar een andere persoon, niet herhaling naar dezelfde.** Herhaalde prikkels aan dezelfde persoon zijn precies het mechanisme waarmee habituatie ontstaat; overdracht naar een ander niveau doorbreekt het.

### 3.6 Onvolledigheid zichtbaar maken in plaats van verbergen

Onvolledigheid wordt getoond als **teller en voortgang**, niet als foutmelding. "6 van de 8 verplichte onderdelen compleet" is een voortgangssignaal; "verplicht veld" is een verwijt.

**Per dossier**, permanent zichtbaar in de kop:

```
Verwerking 0412-K · Verzuimregistratie · Concept
████████████░░░░  11 van 14 verplichte onderdelen

Ontbreekt nog:
  • Bewaartermijn                                  [ Invullen ]
  • Uitzonderingsgrond art. 9(2)                    [ Invullen ]
  • Verwerkersovereenkomst arbodienst (VWO-01)      [ Naar leverancier ]
```

**Per register**, als statuspaneel met klikbare tellers:

```
Verwerkingsregister — 43 verwerkingen

  38  vastgesteld                                   ▸
   3  zonder bewaartermijn                          ▸
   1  doorgifte zonder waarborg                     ▸
   2  niet herzien in 14 maanden                    ▸
   5  geërfd, niet geverifieerd                     ▸
   2  verwerker zonder overeenkomst                 ▸
```

Elke teller opent de betreffende regels. Dit overbrugt de evaluatiekloof: de gebruiker weet na het opslaan of het register voldoet, zonder dat te hoeven afleiden.

**Diff-weergave na elke wijziging**: wat is er precies veranderd, door wie, wanneer, met welke motivering. Dat overbrugt de evaluatiekloof én levert het auditspoor — het is geen aparte voorziening maar een direct gevolg van het datamodel.

**Geen verborgen onvolledigheid.** Exports, rapportages en de registerweergave voor een toezichthouder tonen dezelfde tellers. Het kenmerk "geërfd, niet geverifieerd" gaat mee in elke export. Er is geen weergave waarin het register er completer uitziet dan het is.

### 3.7 Onomkeerbare handelingen

Onomkeerbaarheid moet een eigen uiterlijk hebben. Een knop die iets buiten de organisatie brengt of iets definitief vernietigt, krijgt een visuele klasse die nergens anders in de applicatie voorkomt, staat nooit naast een goedaardige knop en nooit in dezelfde stijl.

| # | Handeling | Waarom onomkeerbaar | Veiligmaking |
|---|---|---|---|
| 1 | Melding indienen bij de AP | verlaat de organisatie | time-out van 6 items (§3.7.1) + tweede persoon + uitgestelde verzending van 5 minuten met annuleerknop + idempotentiesleutel tegen dubbele indiening |
| 2 | NIS2 vroegtijdige waarschuwing (24u) | idem | verkorte time-out van 3 items; **geen** uitstel — snelheid gaat hier voor, en het bericht is bewust beperkt van omvang |
| 3 | NIS2 incidentmelding (72u) en eindrapport | idem | volledige time-out + tweede persoon |
| 4 | Bericht aan betrokkenen (art. 34) | verlaat de organisatie, groot bereik | tweede persoon + ontvangerslijstcontrole met aantal + **verzendmethode-lockout**: bij meer dan één ontvanger uitsluitend individuele verzending of een gescheiden lijst; één bericht met meerdere zichtbare geadresseerden bestaat niet + uitgestelde verzending |
| 5 | Antwoord op een betrokkenenverzoek | verlaat de organisatie, bevat vaak het volledige dossier | time-out + **bijlagecontrole**: elke bijlage wordt afzonderlijk bevestigd met naam, aantal pagina's en herkomstdossier; bijlagen uit een ánder dossier zijn niet selecteerbaar + tweede persoon bij bijzondere gegevens |
| 6 | Besluit "niet melden" | de termijn verstrijkt; herstel onmogelijk | zwaarste barrière: tweede persoon met complementaire controle, of afkoelperiode; permanent zichtbaar in de rapportage |
| 7 | Definitieve vernietiging na bewaartermijn | gegevens weg | tweede persoon + omvangsopgave vooraf + **uitgestelde uitvoering van 7 dagen** met herstelvenster + verplichte bevestiging achteraf met aantal en bewijs |
| 8 | Export van een volledig dossier | gegevens verlaten de tool | tweede persoon + dubbele invoer van de bestemming + classificatie-lockout (bijzondere gegevens naar een onbeveiligde bestemming is geblokkeerd, niet gewaarschuwd) + exportstempel + auditspoor |
| 9 | Publicatie van het register of een Woo-stuk | openbaarmaking | verplichte lak- en metadatacontrole als aparte stap met itemsgewijze bevestiging; de export verwijdert de gelakte tekst en de documenteigenschappen — bedekken is geen bestaande handeling |
| 10 | Sjabloon- of taxonomiewijziging publiceren | raakt alle bestaande en toekomstige dossiers | review-gate door een tweede persoon + impactanalyse met de dossierlijst + versienummer + terugdraaibaar naar de vorige versie |
| 11 | Wijziging van de autorisatiematrix | toegang tot het gevoeligste materiaal | tweede persoon + diff-weergave + terugdraaibaar |
| 12 | Sleutelrotatie of wijziging van het kluiswachtwoord | verlies betekent alles kwijt | herstelcode met dubbele invoer + bevestigde back-up vóór uitvoering + tweefasige uitvoering met bevestiging |
| 13 | Intrekken van een registerregel | — | **niet** onomkeerbaar: versie blijft; alleen inline undo, geen bevestigingsdialoog |

#### 3.7.1 Het time-out-scherm

Ontleend aan de pre-incisie time-out uit de chirurgie: pauzeren, de beschikbare informatie beoordelen, actief bevestigen. **Elk item wordt afzonderlijk bevestigd, met de waarde ernaast getoond.** Eén vinkje voor alles is een bevestigingsdialoog met extra stappen en heeft dezelfde habituatie-eigenschappen.

```
TIME-OUT — Melding aan de Autoriteit Persoonsgegevens
Deze melding verlaat de organisatie en is niet in te trekken.

☐ Entiteit          Stichting Zorggroep Noord (KvK 41xxxxxx)
                    Niet: Zorggroep Noord Vastgoed B.V.
☐ Aard              Onbevoegde verstrekking — e-mail naar verkeerde ontvanger
☐ Betrokkenen       ca. 340 personen, categorie: cliënten
                    Gegevens: NAW, BSN, behandelgegevens
☐ Ontvanger         Autoriteit Persoonsgegevens (leidende autoriteit,
                    vastgelegd 12-01-2026 op grond van hoofdvestiging Groningen)
☐ Termijn           Uiterlijk wo 20 augustus 2026, 09:14 (CEST) — nog 41 uur
☐ Bijlagen          2 bestanden: risicobeoordeling.pdf (4 p.),
                    tijdlijn.pdf (2 p.) — beide uit dossier 2026-0117

Tweede beoordelaar: M. de Wit, akkoord 18-08 14:02
                    (beoordeelde de onderliggende feiten, niet dit formulier)

                              [ Terug ]   [ Verzenden over 5:00 ▸ ]
```

Na "Verzenden" loopt vijf minuten een annuleervenster met een prominente annuleerknop in de werkvoorraad. Pas ná bevestiging van ontvangst met referentienummer en tijdstempel krijgt het dossier de status "Gemeld" — niet eerder.

### 3.8 Volledige ongedaanmaakbaarheid en versiegeschiedenis in plaats van bevestigingsdialogen

**Waarom "weet je het zeker?" niet werkt.** Door habituatie wordt een herhaalde bevestiging een verlengstuk van de klik ervóór: de gebruiker klikt niet "ja" op de vraag, hij klikt de dialoog weg als onderdeel van één motorische handeling. Bovendien beschermt een bevestigingsdialoog vooral tegen slips en niet tegen mistakes: wie het verkeerde plan heeft, bevestigt dat verkeerde plan met volle overtuiging.

**Ontwerpuitgangspunt: niets wordt hard verwijderd; alles is een versie.**

| In plaats van | Doet de tool |
|---|---|
| "Weet u zeker dat u deze verwerking wilt verwijderen?" | Regel wordt ingetrokken met einddatum en reden. Inline: *"Verwerking 0412-K ingetrokken per 18-08-2026. [ Ongedaan maken ]"* — 30 seconden zichtbaar, daarna terugvindbaar in de versiehistorie. |
| "Weet u zeker dat u wilt overschrijven?" | Nieuwe versie; de oude blijft, met diff. |
| "Wijzigingen gaan verloren" | Wijzigingen gaan nooit verloren; concept is een geldige, opgeslagen toestand. |
| "Weet u zeker dat u wilt afsluiten?" | Afsluiten van het scherm sluit het dossier niet. Een open datalekdossier blijft in de werkvoorraad staan met lopende klok tot het is afgesloten of expliciet ingetrokken mét reden (lockin). |

**Wat het model oplevert:**

- Volledige versiehistorie per dossier met herstel per veld, wie, wanneer en met welke motivering.
- Een prullenbak met hersteltermijn voor ingetrokken objecten.
- Het manipulatiebestendige auditspoor uit de README is geen aparte voorziening maar een gevolg van het datamodel.
- Een historisch geldig register is reconstrueerbaar op elke datum — wat de verantwoordingsplicht sowieso vereist.
- Undo maakt experimenteren goedkoop, wat de leercurve verkort en het mentale model verbetert, en daarmee indirect ook mistakes vermindert.

**Waar een bevestiging tóch gerechtvaardigd is — precies drie gevallen:**

1. De handeling verlaat de organisatie of vernietigt gegevens definitief (§3.7). Dan is het geen dialoog maar een time-out met itemsgewijze bevestiging.
2. De handeling raakt veel objecten tegelijk. Dan toont de tool het aantal en de uitklapbare lijst *vóórdat* er iets gebeurt: *"Deze wijziging in het DPIA-sjabloon raakt 47 lopende dossiers. [ Tonen ]"*.
3. De handeling is technisch niet terug te draaien binnen de tool (sleutelrotatie). Dan is er een herstelcode met dubbele invoer en een bevestigde back-up, niet een vraag.

Nooit "Weet u het zeker?" — altijd wát dit is, in termen waarin de gebruiker zijn eigen vergissing kan herkennen.

### 3.9 Waarschuwingsmoeheid voorkomen

De cijfers uit de klinische besluitondersteuning zijn ontnuchterend: overrule-percentages tussen 0,49 en 0,96, met een uitschieter van 92,9%. Alertacceptatie daalt bij hogere werkcomplexiteit en bij meer alerts met lage informatiewaarde. **Elke waarschuwing die genegeerd kán worden, wórdt op termijn genegeerd — en zij sleept de wél belangrijke waarschuwingen mee in haar val.**

**Drie niveaus, streng gescheiden:**

| Niveau | Vorm | Wanneer | Frequentienorm |
|---|---|---|---|
| **Blokkerend** | verhindert de statusovergang; staat bij het veld, met de wettelijke grond en twee uitwegen | uitsluitend bij objectief bepaalbare, wettelijk harde condities | zeldzaam; elke nieuwe blokkade vereist expliciete goedkeuring in het ontwerp |
| **Prominent passief** | statusstrook in het dossier, geen modal; blijft staan tot opgelost of gemotiveerd geaccepteerd | waar oordeel nodig is | ongelimiteerd in aantal, want niet onderbrekend |
| **Rapportregel** | uitsluitend in de periodieke kwaliteitsrapportage | patronen, drift, latente condities | wekelijks of per kwartaal |

**Regels:**

1. **Waarschuwingsbudget.** Maximaal vijf *onderbrekende* meldingen per gebruiker per week — onderbrekend betekent: ongevraagd, buiten een door de gebruiker gestarte handeling. Overschrijding is een defect (SYS-06). Zonder budget groeit het aantal alerts monotoon, want elke individuele alert lijkt gerechtvaardigd.
2. **Nieuwe signalen beginnen op rapportniveau** en promoveren alleen op bewijs: het signaal moet aantoonbaar tot correctie leiden.
3. **Dedupliceren binnen het dossier.** Dezelfde waarschuwing één keer, met teller — niet vijf keer in één sessie.
4. **Rolgericht routeren.** Een bewaartermijnsignaal gaat naar de proceseigenaar, een technisch signaal naar de security officer. De FG krijgt de uitzonderingen en de escalaties, niet de ruis.
5. **Elke waarschuwing is met één klik handelbaar.** Een waarschuwing zonder oplossingsknop is een verwijt, geen hulpmiddel.
6. **Meet en snoei.** Per signaalregel wordt lokaal geregistreerd: hoe vaak getoond, hoe vaak opgevolgd, hoe vaak genegeerd. Regels die boven 80% genegeerd worden, gaan in de kwartaalreview op de schop of eruit (SYS-05). Dit past bij het telemetrievrije uitgangspunt: de gegevens verlaten de machine niet, maar de FG kan zijn eigen tool onderhouden.

**Kwaliteit van de blokkerende melding.** Drie eigenschappen maken het verschil: benoem het gevolg, geef de wettelijke grond, bied twee uitwegen waarvan één de situationele violation legaliseert.

```
Deze verwerking kan nog niet worden vastgesteld.

De bewaartermijn ontbreekt. Artikel 30(1)(f) AVG vraagt om de beoogde
termijnen waarbinnen de verschillende categorieën gegevens worden gewist.

  [ Bewaartermijn invullen ]   [ Motiveren waarom die nog niet bepaald kan worden ]
```

De tweede knop legt vast: reden, uiterlijke datum en eigenaar — en wordt automatisch een taak. Niemand hoeft iets te verzinnen om verder te komen.

**Verdere regels:** nooit ingevoerde gegevens wissen bij een fout; de melding staat bij het veld, niet bovenaan de pagina; valideer bij het verlaten van het veld, niet bij elke toetsaanslag en niet pas bij verzending; markeer de *optionele* velden wanneer de meeste velden verplicht zijn, want anders is elk sterretje betekenisloos.

**Sterile cockpit.** Tijdens het opstellen van een AP- of NIS2-melding, het formuleren van een DPIA-conclusie en het opstellen van een antwoord op een betrokkenenverzoek is de focusmodus automatisch actief: niet-kritieke meldingen onderdrukt en gebufferd, geen banners, geen achtergrondverversing die de weergave verspringt, één dossier tegelijk. Onderbrekingen die tóch binnenkomen worden geregistreerd, zodat zichtbaar wordt hoe vaak kritiek werk wordt onderbroken — een latente conditie die anders onzichtbaar blijft.

### 3.10 Vier-ogen-principe: waar wel, waar niet

**Ontwerpnuance die het vaakst gemist wordt:** de tweede persoon doet een **complementaire** controle, niet dezelfde. Toon de tweede beoordelaar niet het ingevulde formulier, maar de onderliggende feiten plus de vraag — anders ontstaat bevestigingsdruk in plaats van controle. Dit is precies de reden dat onafhankelijke dubbelcontrole in de effectiviteitshiërarchie middelhoog en niet hoog scoort.

| Wél vier ogen | Waarom |
|---|---|
| Besluit "niet melden" aan de toezichthouder | gevaarlijkste beslissing in het proces; termijn verstrijkt onherstelbaar |
| Melding aan de AP, aan het CSIRT, aan een sectorale toezichthouder | verlaat de organisatie, niet in te trekken |
| Bericht aan betrokkenen (art. 34) | groot bereik, onherstelbaar |
| Antwoord op een betrokkenenverzoek dat bijzondere gegevens bevat | de klassieke fout van de verkeerde bijlage |
| Risico-uitkomst naar beneden bijgesteld ten opzichte van de weging | tegen de systematiek in |
| DPIA-uitkomst "niet nodig" bij precies één EDPB-criterium | randgeval, hoge inzet |
| Afwijken van het FG-advies | FSV-les |
| Definitieve vernietiging na bewaartermijn | onomkeerbaar |
| Wijziging van een sjabloon, taxonomie of termijnrekenregel | latente conditie die zich over honderden dossiers vermenigvuldigt |
| Wijziging van de autorisatiematrix | toegang tot het gevoeligste materiaal |
| Export van een volledig dossier of van het register | gegevens verlaten de tool |
| Meldtermijn verwerker vanaf 48 uur accepteren | maakt de eigen termijn structureel onhaalbaar |

| Géén vier ogen | Waarom niet |
|---|---|
| Registerregel aanmaken of bijwerken in concept | routine; vier ogen op invoervelden levert alleen habituatie op |
| Vaststellen van een registerregel | er is al een andere persoon in de keten: de proceseigenaar keurt goed |
| Intake van een datalek of een verzoek | snelheid gaat voor; drempels bij intake zijn de reden dat er niet wordt gemeld |
| Taken afvinken, notities toevoegen, dossiers lezen | omkeerbaar, geen externe werking |
| Iedere afzonderlijke veldwijziging | zou het budget aan aandacht uitputten waardoor de wél zinvolle controles verwateren |

**Vier ogen zit op oordeelsvelden, niet op invoervelden.** Dat is de enige effectieve barrière tegen knowledge-based mistakes, en tegelijk de reden om hem spaarzaam in te zetten.

**Hoe de complementaire controle eruitziet:**

```
BEOORDELING GEVRAAGD — dossier 2026-0117
U beoordeelt niet het ingevulde formulier, maar de vraag zelf.

De feiten
  Op 16-08 om 22:10 is een e-mail met een bijlage verzonden aan
  een adres buiten de organisatie. De bijlage bevat 340 regels met
  NAW, BSN en behandelcode. De ontvanger heeft niet gereageerd.
  De e-mail is niet ingetrokken. Betrokkenen zijn cliënten van de
  ambulante ggz.

De vraag
  Levert dit een risico op voor de rechten en vrijheden van
  de betrokkenen?

  ( ) geen risico     ( ) risico     ( ) hoog risico
  Motivering: [                                              ]

  ▸ Uw oordeel wordt pas naast dat van de behandelaar gelegd nadat
    u het heeft vastgelegd. Bij verschil ziet u beiden beide
    motiveringen en wordt het besluit gezamenlijk vastgelegd.
```

**Eenpersoons-FG.** De regel is dan niet uitvoerbaar. Vervang hem niet door niets, maar door een **afkoelperiode**: het besluit en de motivering worden vastgelegd, de applicatie dwingt een tweede sessie na minimaal 30 minuten — of de volgende ochtend waar de termijn dat toelaat — waarin het besluit opnieuw wordt bevestigd op basis van de feiten, met de eerdere motivering pas ná de herbevestiging zichtbaar. Het ontwerpdocument legt expliciet vast dat dit een **zwakkere laag** is, en de directierapportage vermeldt hoe vaak deze route is gebruikt (ORG-03).

**Read-back / hear-back bij overdracht.** De lus is pas gesloten bij de derde stap. De ontvanger reproduceert de kritieke waarden actief uit keuzelijsten — aard van het incident, aantal betrokkenen, eerstvolgende deadline, openstaande toezegging — in plaats van een samenvatting te lezen en akkoord te klikken. Afwijkingen tussen wat de zender vastlegde en wat de ontvanger reproduceert worden aan beiden getoond. Bij telefonische intake vinkt de intakemedewerker "teruggelezen en bevestigd door [naam melder] om [tijd]" — dat veld is later verantwoordingsmateriaal.

### 3.11 Halverwege stoppen, of de computer valt uit

| Situatie | Gedrag |
|---|---|
| Gebruiker stopt halverwege | Concept is een geldige, opgeslagen toestand. Niets wordt geweigerd wegens onvolledigheid. Het dossier verschijnt in de werkvoorraad met de voortgangsteller. |
| Gebruiker navigeert weg | Elk veld is opgeslagen bij het verlaten; er is geen "opslaan?"-vraag. |
| Sessie verloopt / scherm op slot | Het scherm gaat op slot, het werk blijft. Na ontgrendeling staat de gebruiker op dezelfde plek met het hervattingsanker. |
| Stroomuitval of crash tijdens het typen | Schrijfacties gaan via een journaal in de versleutelde opslag (write-ahead, atomaire hernoeming, geforceerde schrijfbevestiging). Naast opslag-bij-veldverlating loopt een intervalopslag van 10 seconden voor lange motiveringsvelden. |
| Opstarten na een crash | Herstelrapport: *"3 dossiers hadden niet-opgeslagen wijzigingen. Hersteld. [ Verschillen tonen ]"* — met diff, nooit stilzwijgend. |
| Crash midden in een onomkeerbare handeling | Tweefasig: intentie vastgelegd → uitvoering → bevestiging. Bij herstart toont de tool onafgeronde handelingen (SYS-08) en biedt afronden of terugdraaien; de applicatie blokkeert tot de gebruiker kiest. Nooit stilzwijgend voltooien, nooit stilzwijgend vergeten. |
| Dubbele verzending na een crash | Elke uitgaande melding draagt een idempotentiesleutel; opnieuw verzenden van dezelfde sleutel levert de bestaande bevestiging, geen tweede melding. |
| Klok verschuift door herstart of tijdzonewijziging | Deadlines zijn absolute momenten; er verschuift niets. Een afwijking tussen systeemklok en referentie wordt vastgelegd. |
| Gebruiker sluit een datalekdossier af zonder het af te ronden | Lockin: het dossier blijft prominent in de werkvoorraad met lopende klok. Het kan niet stilzwijgend verdwijnen. |
| Twee sessies bewerken hetzelfde dossier | Veldniveau-samenvoeging waar mogelijk; bij een echt conflict beide versies tonen met de vraag welke geldt, nooit stilzwijgend overschrijven. |
| Beschadigde opslag | Integriteitscontrole bij openen; een ketenbreuk in het auditspoor blokkeert alle bewerkingen en meldt direct (SYS-04). |

---

## 4. Ingebouwde kennis

Uitgangspunt: **de tool kent de regels, zodat de gebruiker ze niet uit het hoofd hoeft te kennen.** Alle kennis is regelgebaseerd, versiebeheerd en herleidbaar naar een bron met datum. Er wordt niets afgeleid dat niet in een expliciete regel, beslisboom of sjabloon is vastgelegd.

### 4.1 Beslisbomen

Elke beslisboom heeft: een naam, een versie, een brondatum, een ingang, expliciete knopen, expliciete uitkomsten, en de vastlegging van het gekozen pad bij de uitkomst.

| Beslisboom | Ingang | Uitkomsten |
|---|---|---|
| DPIA-plicht | nieuwe of gewijzigde verwerking | verplicht / gemotiveerd niet verplicht / niet verplicht |
| Rolbepaling | nieuwe leverancier of samenwerking | verwerker / verwerkingsverantwoordelijke / gezamenlijk verantwoordelijken |
| Grondslagkeuze | doel vastgesteld | een van de zes van art. 6, met de bijbehorende vervolgvragen |
| Art. 9-uitzondering | bijzondere categorie aangevinkt | een van de tien uitzonderingen, met UAVG-aanvulling |
| Datalek: is het een inbreuk | incidentintake | geen datalek / datalek (met de drie beveiligingsaspecten apart) |
| Datalek: risicoweging | inbreuk vastgesteld | geen risico / risico / hoog risico |
| Datalek: melden aan wie | risico vastgesteld | AP / verwerkingsverantwoordelijke / CSIRT / sectoraal / combinatie |
| Art. 34: informeren betrokkenen | hoog risico | informeren / uitzondering (a), (b) of (c) met alternatief |
| Bulkmelding toelaatbaar | tweede gelijksoortig incident | toevoegen aan groep / apart melden |
| NIS2-toepasselijkheid | entiteit aanmaken | essentieel / belangrijk / niet van toepassing |
| NIS2-significantie | incident met NIS2-spoor | significant / niet significant |
| Doorgifte: waarborg kiezen | land buiten de EER | adequaatheidsbesluit / standaardbepalingen / BCR / art. 49-uitzondering / geen doorgifte mogelijk |
| Verzoektype herkennen | intake betrokkenenverzoek | inzage / rectificatie / wissing / beperking / bezwaar / overdraagbaarheid / art. 22 / geen verzoek |
| Wissing: uitzondering van toepassing | wisverzoek | wissen / gedeeltelijk / weigeren op grond van art. 17(3) |
| Bewaartermijn bepalen | registerregel met categorie gegevens | wettelijke termijn uit de bibliotheek / organisatorische termijn met motivering |
| Gerechtvaardigd belang | keuze art. 6(1)(f) | toelaatbaar met motivering / niet toelaatbaar |

### 4.2 Ingebouwde toetsingskaders

Deze staan **náást het veld op het beslismoment**, niet in een helppagina.

| Kader | Waar in beeld |
|---|---|
| De negen EDPB-criteria voor DPIA-plicht, met een voorbeeld per criterium | DPIA-toets |
| De AP-lijst van verplichte DPIA's | DPIA-toets, als eerste knoop |
| De vier verplichte onderdelen van art. 35(7) | DPIA-dossier, als voortgangsteller |
| De vier elementen van art. 33(3) | meldformulier, als voortgangsteller |
| De acht risicofactoren voor de datalekweging | risicobeoordeling, elk als eigen as |
| De drie uitzonderingen van art. 34(3) | art. 34-besluit, als enige keuzelijst |
| De uitzonderingen van art. 17(3) | wisverzoek per vindplaats |
| De eisenlijst van art. 28(2), (3)(a)-(h), (4), 29, 32 en 33(2) | VWO-mapping, als checklist met vindplaats |
| De waarborgen van hoofdstuk V en de uitzonderingen van art. 49 | doorgifteblok |
| De drietrapstoets voor gerechtvaardigd belang | grondslagblok |
| De vier eisen aan doelomschrijving | doelveld |
| De verenigbaarheidsfactoren van art. 6(4) | nieuw doel op bestaande verwerking |
| De significantiecriteria voor NIS2 | NIS2-spoor |
| De voorwaarden voor bulkmelden | groepssamensteller |
| De anonimiseringstoets (singling out, koppelbaarheid, afleidbaarheid) | wisverzoek bij keuze "geanonimiseerd" |

Elk kader draagt zijn bron en datum. Bij een gewijzigd kader toont de impactanalyse welke bestaande uitkomsten opnieuw beoordeeld moeten worden (§4.6).

### 4.3 Sjablonen

Sjablonen zijn versiebeheerde documenten met **invulplaatsen die uit het dossier worden gevuld**, plus verplichte velden die de gebruiker zelf invult. Een sjabloon dat volledig automatisch compleet is, is verdacht: dan is er geen oordeel toegevoegd.

| Sjabloon | Gevuld uit | Verplicht zelf in te vullen |
|---|---|---|
| Verwerkersovereenkomst (model) | leverancier, systemen, gegevenscategorieën, doel, bewaartermijn, subverwerkers, meldtermijn 24 uur, exitregeling | per eis de concrete invulling; beveiligingsbijlage |
| Melding aan de AP | art. 33(3)-elementen, aantallen, categorieën, tijdlijn, contactgegevens FG | omschrijving van de waarschijnlijke gevolgen; genomen maatregelen |
| Bericht aan betrokkenen (art. 34) | aard van het lek, categorieën gegevens, contactpunt | wat de betrokkene zelf kan doen; excuses en context |
| Antwoord inzageverzoek | vindplaatsen, categorieën, ontvangers, bewaartermijnen, herkomst, rechten | de gegevenskopie zelf; toelichting per uitzondering |
| Antwoord wisverzoek | resultaat per vindplaats, uitzonderingen | motivering per geweigerd onderdeel |
| Verlengingsbericht | ontvangstdatum, nieuwe termijn, grond | toelichting op de complexiteit |
| DPIA-format | verwerking, doel, grondslag, gegevens, betrokkenen, ontvangers, doorgiften, bewaartermijn, maatregelen | noodzaak en evenredigheid; risico's; aanvullende maatregelen; restrisico |
| Datalek-intakeformulier | — | de zeven intakevragen |
| SBAR-overdracht | situatie, achtergrond, beoordeling, aanbeveling — uit het dossier | bevestiging door de ontvanger (read-back) |
| Directierapportage | tellers, termijnprestatie, escalaties, afwijkingen van FG-advies, middelenregistratie | duiding en gevraagde besluiten |
| Register-export | volledige registerinhoud plus de tellers van onvolledigheid | — |
| Voorafgaande raadpleging (art. 36) | DPIA, restrisico, maatregelen | motivering van het resterende hoge risico |
| NIS2 vroegtijdige waarschuwing | entiteit, tijdstip, vermoeden, grensoverschrijding | korte omschrijving |
| NIS2 eindrapport | tijdlijn, maatregelen | grondoorzaak; grensoverschrijdende gevolgen |

### 4.4 Standaardteksten met invulplaatsen

Voor terugkerende formuleringen waar een verkeerde formulering juridisch gevolg heeft:

- Doelomschrijving: *"[activiteit] ten behoeve van [proces] voor [categorie betrokkenen], zodat [resultaat]"*.
- Bewaartermijnmotivering: *"[getal] [eenheid] na [startgebeurtenis], op grond van [wettelijke bepaling / organisatorische reden], omdat [motivering]"*.
- Meldtermijn verwerker: *"onverwijld en uiterlijk binnen 24 uur na kennisname"* — als vaste standaardwaarde in het model.
- Rechtsmiddelenpassage bij een afwijzing: klachtrecht bij de AP en beroepsmogelijkheid, met de juiste verwijzingen.
- Exitclausule: teruggave in [formaat] binnen [termijn], gevolgd door verwijdering met schriftelijke bevestiging binnen [termijn].

### 4.5 Bibliotheek van veelvoorkomende verwerkingen

Bij het aanmaken van een verwerking biedt de tool eerst een keuze uit de bibliotheek. De gekozen inhoud wordt **voorgevuld**, niet vastgesteld: elke overgenomen waarde draagt de status *"overgenomen uit de bibliotheek, niet getoetst"* tot de gebruiker haar bevestigt of aanpast. Een niet-bevestigde waarde blokkeert de vaststelling. Zonder die regel wordt de bibliotheek zelf de gevaarlijkste latente conditie in de applicatie.

De waarden hieronder zijn **richtwaarden en vertrekpunten**, geen normen. De organisatie bevestigt en motiveert per geval.

| # | Verwerking | Typisch doel | Typische grondslag | Bijzondere gegevens | Richtwaarde bewaartermijn | Vaste aandachtspunten |
|---|---|---|---|---|---|---|
| 1 | Personeelsadministratie | uitvoering arbeidsovereenkomst, personeelsbeheer | art. 6(1)(b) en (c) | nee | 2 jaar na einde dienstverband; fiscaal relevante delen 7 jaar; loonheffingsgegevens en identiteitsbewijs 5 jaar na einde dienstverband | dossierdelen apart; salarisverwerker |
| 2 | Salarisadministratie | loonbetaling en afdracht | art. 6(1)(b) en (c) | nee | 7 jaar (fiscaal) | verwerkersovereenkomst; bankgegevens |
| 3 | Werving en selectie | vervulling van een vacature | art. 6(1)(b) precontractueel of (f) | nee | 4 weken na afronding; 1 jaar met toestemming | toestemming apart vastleggen; assessmentgegevens; referenties |
| 4 | Verzuim en re-integratie | loondoorbetaling, Wet verbetering poortwachter | art. 6(1)(c), bij de arbodienst met art. 9(2)(h) | ja (gezondheid) | 2 jaar na einde verzuim bij de arbodienst; werkgever registreert strikt beperkt | AP-beleidsregels over wat de werkgever mág weten; rol arbodienst |
| 5 | Cameratoezicht | beveiliging van personen en eigendommen | art. 6(1)(f) | soms | richtwaarde 4 weken; langer alleen bij een concreet incident | DPIA bij systematische monitoring; kenbaarheid; instemming OR |
| 6 | Toegangscontrole en bezoekersregistratie | beveiliging van het gebouw | art. 6(1)(f) | nee | logging 3-6 maanden; bezoekerslijst korter | koppeling met camerabeelden vermijden of motiveren |
| 7 | Klant- of cliëntadministratie | uitvoering van de overeenkomst | art. 6(1)(b) | nee | administratie 7 jaar fiscaal; contactgegevens korter | onderscheid administratie en dossier |
| 8 | Debiteurenbeheer en incasso | inning van vorderingen | art. 6(1)(b) en (f) | nee | 7 jaar | rolbepaling incassobureau |
| 9 | Nieuwsbrief en direct marketing | informeren en verkoopbevordering | toestemming; bij bestaande klanten art. 6(1)(f) met de uitzondering uit de Telecommunicatiewet | nee | tot intrekking; bewijs van toestemming 2 jaar na intrekking | afmeldroute in elk bericht; verzenden via een verzendtool, niet via de mailclient |
| 10 | Websitestatistieken en cookies | meten en verbeteren | toestemming, tenzij strikt noodzakelijk | nee | 6-26 maanden | doorgifte buiten de EER; verwerkersrol; toestemmingsbewijs |
| 11 | Klachten en bezwaren | behandeling en afdoening | art. 6(1)(c), (e) of (f) | soms | 2 jaar na afhandeling; bij bestuursorganen volgens de selectielijst | een klacht kan een verkapt betrokkenenverzoek bevatten |
| 12 | Leveranciers- en contactpersonenadministratie | inkoop en contractbeheer | art. 6(1)(b) en (f) | nee | 7 jaar na einde contract | koppeling naar de VWO-bewaking |
| 13 | ICT-beheer en logging | beveiliging, beschikbaarheid, foutanalyse | art. 6(1)(c) en (f) | nee | 6 maanden tot 1 jaar | logging is zelf een verwerking; toegang tot logs beperken |
| 14 | Toegangs- en autorisatiebeheer | beheersing van toegang | art. 6(1)(c) en (f) | nee | duur dienstverband plus 1 jaar | periodieke autorisatiereview als maatregel opnemen |
| 15 | Incident- en datalekregistratie | naleving art. 33 lid 5 en leren | art. 6(1)(c) | soms | 5 jaar na afsluiting, motiveerbaar | dit register bevat zelf gevoelig materiaal |
| 16 | Integriteits- en klokkenluidersmeldingen | onderzoek naar misstanden | art. 6(1)(c) (Wet bescherming klokkenluiders) | vaak, ook art. 10 | 2 jaar na afsluiting; langer bij een procedure | zeer beperkte kring; DPIA; vertrouwelijkheid van de melder |
| 17 | Cliëntdossier zorg | goede zorgverlening | art. 6(1)(b) of (c) met art. 9(2)(h) | ja | 20 jaar na einde behandeling (WGBO), of zoveel langer als goed hulpverlenerschap vereist | vernietigingsverzoek WGBO; logging van inzage; tweefactorauthenticatie |
| 18 | Leerling- of studentenadministratie | onderwijs en begeleiding | art. 6(1)(c) en (e) | soms | 2 jaar na uitschrijving; verzuimgegevens 5 jaar | verwerkers van leermiddelen; doorgifte; ouderlijk gezag |
| 19 | Subsidie- en aanvraagbehandeling | uitvoering van een wettelijke taak | art. 6(1)(c) en (e) | soms | volgens de Archiefwet-selectielijst | publicatie op grond van de Woo: lakken en documenteigenschappen |
| 20 | Toezicht en handhaving | uitvoering van een publieke taak | art. 6(1)(e), waar van toepassing art. 10 | ja | volgens de selectielijst | risicomodellen: DPIA, art. 22, rectificatie- en schoningsproces |
| 21 | Wagenpark en rittenregistratie | wagenparkbeheer en fiscale verantwoording | art. 6(1)(c) en (f) | nee | rittenregistratie 7 jaar fiscaal; locatiegegevens korter | volgen van werknemers: DPIA, instemming OR |
| 22 | Telefoonopnames klantcontact | kwaliteit, opleiding, bewijs | art. 6(1)(f) of (b) | nee | 1-3 maanden; bewijsopnames langer | grondslag per doel apart; mededeling vooraf |
| 23 | Schuldhulpverlening en betalingsregelingen | uitvoering van een wettelijke taak | art. 6(1)(c) en (e) | vaak | volgens de selectielijst en de Wgs | onjuiste of verouderde gegevens hebben directe gevolgen; rectificatieproces verplicht inrichten |
| 24 | Back-up en archivering | continuïteit en wettelijke archivering | volgt de onderliggende verwerking | volgt | hersteltermijn expliciet vastleggen | wisverzoeken en herstelscenario's |

### 4.6 Onderhoud van de ingebouwde kennis

De kennisbank is zelf een latente conditie en wordt dienovereenkomstig behandeld.

| Maatregel | Uitwerking |
|---|---|
| Versiebeheer | elke beslisboom, elk toetsingskader, elk sjabloon en elke taxonomie heeft een versienummer en een brondatum |
| Review-gate | een wijziging is niet te publiceren zonder review door een tweede persoon (SYS-01) |
| Impactanalyse vóór publicatie | *"Deze wijziging in het DPIA-sjabloon raakt 47 lopende dossiers. [ Tonen ]"* — zonder die knop is de latente conditie onzichtbaar (SYS-02) |
| Vastlegging bij de uitkomst | elke uitkomst bewaart de versie van de beslisboom waarmee zij tot stand kwam |
| Herbeoordeling bij wijziging | wijzigt een boom of kader, dan worden de eerdere uitkomsten die daardoor kunnen omslaan als taak aangeboden |
| Levering als ondertekend pakket | de kennisbank wordt als afzonderlijk, ondertekend pakket geleverd en handmatig geïnstalleerd; geen automatische update, conform het lokale uitgangspunt van het project |
| Zichtbare veroudering | de leeftijd van de kennisbank staat in de directierapportage; achterstand is een rapportregel |
| Cascade bij juridische wijzigingen | één beheerhandeling "juridische wijziging registreren" laat alle afhankelijke dossiers herzien (§2.7) |

---

## 5. Controles die continu draaien

### 5.1 Werking

- **Wanneer:** bij elke schrijfactie op het geraakte object, plus een volledige doorloop over de hele gegevensverzameling bij het openen van de kluis en daarna dagelijks.
- **Drie niveaus:** **Blokkerend** (B) verhindert een statusovergang; **Signaal** (S) staat prominent passief in het dossier tot opgelost of gemotiveerd geaccepteerd; **Rapport** (R) verschijnt uitsluitend in de periodieke kwaliteitsrapportage.
- **Routering:** elke regel heeft een vaste ontvangerrol. De FG krijgt de uitzonderingen en escalaties, niet de ruis.
- **Introductie van een nieuwe regel:** start altijd op R; promotie naar S of B alleen op bewijs dat de regel tot correctie leidt.
- **Snoeien:** regels die in een kwartaal in meer dan 80% van de gevallen genegeerd worden, gaan in de review op de schop (SYS-05).
- **Gemotiveerde acceptatie:** elk S-signaal kan gemotiveerd worden geaccepteerd, met eigenaar en herbeoordelingsdatum. De acceptatie is zichtbaar en telbaar; zij verdwijnt niet.

### 5.2 De controleregels

#### Verwerkingsregister (REG)

| ID | Naam | Wat het controleert | Niveau | Naar |
|---|---|---|---|---|
| REG-01 | Doel ontbreekt | verwerking zonder ten minste één doel | B | FG |
| REG-02 | Doel zonder grondslag | een doel zonder gekoppelde grondslag | B | FG |
| REG-03 | Categorie betrokkenen ontbreekt | geen categorie betrokkenen vastgelegd | B | proceseigenaar |
| REG-04 | Categorie gegevens ontbreekt | geen categorie persoonsgegevens vastgelegd | B | proceseigenaar |
| REG-05 | Bewaartermijn ontbreekt | geen termijn en geen gemotiveerde uitstelafspraak | B | proceseigenaar |
| REG-06 | Opslaglocatie ontbreekt | geen gekoppeld systeem | B | proceseigenaar |
| REG-07 | Ontvangers onbepaald | ontvangers leeg en niet expliciet "geen" | B | proceseigenaar |
| REG-08 | Niet herzien in 12 maanden | herzieningsdatum verstreken | S (R vanaf 15 mnd) | proceseigenaar |
| REG-09 | Geërfd, niet geverifieerd | kenmerk ouder dan 90 dagen | S | FG |
| REG-10 | Systeem zonder registerregel | systeem in de systeemlijst zonder enige verwerking | S | FG |
| REG-11 | Leverancier zonder registerkoppeling | leverancier zonder enige verwerking | R | FG |
| REG-12 | Vervallen proces, regel nog actief | proces gemarkeerd als vervallen, regel niet ingetrokken | S | proceseigenaar |
| REG-13 | Uitgefaseerd systeem nog gekoppeld | gekoppeld systeem heeft status uitgefaseerd | S | FG |
| REG-14 | Verwerkersvelden ontbreken | rol verwerker zonder art. 30(2)-velden | B | FG |
| REG-15 | Mogelijke dubbele registerregel | identiek doel, systeem en categorie betrokkenen | R | FG |
| REG-16 | Contactgegevens entiteit ontbreken | verwerkingsverantwoordelijke of FG niet vastgelegd | B (op export) | FG |
| REG-17 | Bedrijfsproces zonder verwerkingen | proces in de processenlijst zonder enige registerregel | R | FG |

#### Grondslag en rechtmatigheid (GRO)

| ID | Naam | Wat het controleert | Niveau | Naar |
|---|---|---|---|---|
| GRO-01 | Gerechtvaardigd belang zonder toets | art. 6(1)(f) zonder afgeronde drietrapstoets | B | FG |
| GRO-02 | Gerechtvaardigd belang bij publieke taak | overheidsinstantie, publieke taak, grondslag (f) | B | FG |
| GRO-03 | Toestemming zonder intrekkingsroute | geen vastgelegde wijze van intrekken | B | proceseigenaar |
| GRO-04 | Toestemming zonder bewijsvoering | geen vastgelegde bewijsbewaring | B | proceseigenaar |
| GRO-05 | Toestemming in een gezagsverhouding | werkgever-werknemer of vergelijkbare afhankelijkheid | S | FG |
| GRO-06 | Wettelijke grondslag zonder vindplaats | art. 6(1)(c) of (e) zonder wet en artikel | B | FG |
| GRO-07 | Bijzondere gegevens zonder uitzondering | art. 9-categorie zonder art. 9(2)-grond | B | FG |
| GRO-08 | Strafrechtelijke gegevens zonder grondslag | art. 10 zonder UAVG-basis | B | FG |
| GRO-09 | BSN zonder eigen grondslag | BSN als categorie zonder wettelijke basis voor het gebruik ervan | B | FG |
| GRO-10 | Doelomschrijving onvolledig | sjabloonvelden van het doel niet ingevuld | S | proceseigenaar |
| GRO-11 | Nieuw doel zonder verenigbaarheidstoets | doel toegevoegd aan vastgestelde verwerking | B | FG |
| GRO-12 | Geautomatiseerde besluitvorming onvolledig | art. 22 aangevinkt zonder grond, logica-uitleg of menselijke tussenkomst | B | FG |

#### Bewaren en verwijderen (BEW)

| ID | Naam | Wat het controleert | Niveau | Naar |
|---|---|---|---|---|
| BEW-01 | Startgebeurtenis ontbreekt | termijn zonder startgebeurtenis | B | proceseigenaar |
| BEW-02 | Termijn zonder uitvoering | geen schoningsopdracht bij een vastgestelde termijn | S | systeemeigenaar |
| BEW-03 | Schoningsopdracht over de datum | uitvoerdatum meer dan 14 dagen verstreken | S (R vanaf 30 dagen) | systeemeigenaar |
| BEW-04 | Termijn boven de richtwaarde | langer dan de bibliotheeknorm zonder motivering | S | FG |
| BEW-05 | Wettelijke plicht te breed toegepast | plichttermijn op een bredere gegevensset dan de plicht dekt | S | FG |
| BEW-06 | Hersteltermijn back-up ontbreekt | systeem zonder vastgelegde back-uphersteltermijn | S (B bij een lopend wisverzoek) | systeemeigenaar |
| BEW-07 | Onbepaalde bewaartermijn | waarde "onbepaald" of "zolang nodig" | B | proceseigenaar |
| BEW-08 | Vernietiging zonder bewijs | afgeronde schoning zonder aantal en bewijs | S | systeemeigenaar |

#### DPIA (DPIA)

| ID | Naam | Wat het controleert | Niveau | Naar |
|---|---|---|---|---|
| DPIA-01 | Verplichte DPIA ontbreekt | verwerking op de AP-lijst zonder DPIA | B | FG |
| DPIA-02 | Twee of meer criteria zonder DPIA | ≥2 EDPB-criteria zonder DPIA | B | FG |
| DPIA-03 | DPIA na aanvang | toetsdatum ligt na de startdatum van de verwerking | S, permanent kenmerk | FG en directie |
| DPIA-04 | FG-advies ontbreekt | DPIA zonder geregistreerd advies | B | FG |
| DPIA-05 | Advies niet opgevolgd, niet gemotiveerd | afwijking zonder motivering of besluitnemer | B | directie |
| DPIA-06 | Hoog restrisico zonder raadpleging | restrisico hoog zonder art. 36-raadpleging | B | FG |
| DPIA-07 | DPIA verouderd | ouder dan 36 maanden of gekoppelde verwerking gewijzigd | S | FG |
| DPIA-08 | Onderdelen art. 35(7) ontbreken | een of meer van de vier onderdelen leeg | B | FG |
| DPIA-09 | Standpunt betrokkenen ontbreekt | art. 35(9) niet gevraagd en niet gemotiveerd nagelaten | S | FG |

#### Verwerkers en contracten (VWO)

| ID | Naam | Wat het controleert | Niveau | Naar |
|---|---|---|---|---|
| VWO-01 | Verwerker zonder overeenkomst | verwerking met verwerker zonder actief contract | B | FG en inkoop |
| VWO-02 | Art. 28(3) niet volledig gemapt | een of meer eisen zonder vindplaats | B | contracteigenaar |
| VWO-03 | Eis zonder concrete invulling | alleen wettekst overgenomen | S | contracteigenaar |
| VWO-04 | Meldtermijn verwerker te lang | boven 24 uur signaal; vanaf 48 uur blokkerend zonder tweede persoon | S / B | FG |
| VWO-05 | Exitafspraak ontbreekt of riskant | geen afspraak, of alleen verwijdering zonder continuïteitsborging | S | contracteigenaar |
| VWO-06 | Verouderd contract | verwijzing naar ingetrokken wetgeving of documenttype bewerkersovereenkomst | S (B na de herzieningsdatum) | contracteigenaar |
| VWO-07 | Geen eind- of herzieningsdatum | contract zonder looptijdbeheer | S | contracteigenaar |
| VWO-08 | Geen contracteigenaar buiten de FG | eigenaarschap uitsluitend bij de FG | S | directie |
| VWO-09 | Subverwerkerslijst niet gecontroleerd | laatste controle ouder dan 12 maanden | S | contracteigenaar |
| VWO-10 | Subverwerker zonder procedure | toegevoegd zonder geregistreerde toestemmings- of bezwaarprocedure | B | FG |
| VWO-11 | Rolbepaling ontbreekt | leverancier als verwerker aangemerkt zonder rolbeslisboom | B | FG |
| VWO-12 | Auditrecht niet geregeld | geen inspectie- of auditbepaling | S | contracteigenaar |
| VWO-13 | Contract getekend na aanvang | ondertekendatum na de startdatum van de verwerking | R | FG |

#### Doorgifte buiten de EER (EER)

| ID | Naam | Wat het controleert | Niveau | Naar |
|---|---|---|---|---|
| EER-01 | Doorgifte zonder waarborg | derde land zonder gekozen waarborg | B | FG |
| EER-02 | Toegangsland onbekend | veld "toegang vanuit" op onbekend | B | FG |
| EER-03 | Standaardbepalingen zonder TIA | geen afgeronde transfer impact assessment | B | FG |
| EER-04 | Standaardbepalingen onvolledig | module, versie of bijlagen ontbreken | B | contracteigenaar |
| EER-05 | Adequaatheidsbesluit zonder reikwijdte | geen vastlegging van reikwijdte of certificering van de ontvanger | S | FG |
| EER-06 | Art. 49 structureel gebruikt | meer dan twee keer per jaar op dezelfde ontvanger | S | FG |
| EER-07 | Waarborg vervallen | geregistreerde juridische wijziging raakt deze doorgifte | B op alle afhankelijke regels | FG |
| EER-08 | Alleen hostingland ingevuld | opslaglocatie ingevuld, toegangsland leeg | B | systeemeigenaar |

#### Datalekken (LEK)

| ID | Naam | Wat het controleert | Niveau | Naar |
|---|---|---|---|---|
| LEK-01 | Geen beoordeling binnen 12 uur | status geregistreerd zonder risicobeoordeling | S met escalatie | behandelaar |
| LEK-02 | Termijn nadert zonder besluit | minder dan 12 uur tot de 72-uursgrens zonder meldbesluit | S met escalatie naar plaatsvervanger | FG |
| LEK-03 | Gat tussen kennisname en registratie | meer dan 4 uur zonder toelichting | B | FG |
| LEK-04 | "Niet melden" zonder tweede laag | geen tweede persoon en geen afkoelperiode | B | FG |
| LEK-05 | Art. 34 blijft liggen | melding verzonden, geen art. 34-besluit binnen 24 uur | S met escalatie | FG |
| LEK-06 | Laag risico bij grote schaal | uitkomst geen risico bij meer dan 250 betrokkenen | S met verplichte tegenspraak | FG |
| LEK-07 | Laag risico bij gevoelige gegevens | uitkomst geen risico terwijl bijzondere gegevens, BSN of financiële gegevens betrokken zijn | B zonder tweede persoon | FG |
| LEK-08 | Inconsistente exfiltratieconclusie | exfiltratie "niet uit te sluiten" maar weging op geen risico | B | FG |
| LEK-09 | Beschikbaarheid niet beoordeeld | beschikbaarheidsaspect niet beantwoord | B | behandelaar |
| LEK-10 | Bulkgroep niet gelijksoortig | afwijkend type, oorzaak of gegevenscategorie in de groep | B | FG |
| LEK-11 | Bulkvenster overschreden | groep loopt langer dan het toegestane venster | B | FG |
| LEK-12 | Afsluiten zonder maatregel of oorzaak | geen maatregel met eigenaar, of geen oorzaakcategorie | B | FG |
| LEK-13 | Herhaalde oorzaak | dezelfde oorzaakcategorie meer dan 3× per kwartaal in hetzelfde proces | R | FG en directie |
| LEK-14 | Afwijkende toezichthouder | melding aan een andere dan de vastgelegde leidende autoriteit | B zonder motivering | FG |
| LEK-15 | Lek zonder registerkoppeling | datalek niet gekoppeld aan een verwerking | S | FG |
| LEK-16 | Verwerker meldde te laat | verwerker overschreed de contractuele meldtermijn | R | contracteigenaar |

#### Betrokkenenverzoeken (BTR)

| ID | Naam | Wat het controleert | Niveau | Naar |
|---|---|---|---|---|
| BTR-01 | Ontvangstdatum ontbreekt | geen datum ontvangst door de organisatie | B | behandelaar |
| BTR-02 | Termijn nadert zonder concept | minder dan 7 dagen te gaan zonder conceptantwoord | S met escalatie | behandelaar |
| BTR-03 | Verlenging zonder bericht | status verlengd zonder verzonden bericht binnen de eerste maand | B | FG |
| BTR-04 | Vindplaats niet afgehandeld | een of meer vindplaatsen zonder uitkomst | B | behandelaar |
| BTR-05 | Identificatie zonder motivering | aanvullende identificatie gevraagd zonder motivering per geval | B | FG |
| BTR-06 | Kosten bij een eerste verzoek | kostenveld gevuld zonder aangetoonde herhaling of ongegrondheid | B | FG |
| BTR-07 | Account sluiten als wissen | vindplaats afgehandeld met "account gesloten" bij een wisverzoek | B | behandelaar |
| BTR-08 | Back-up uitgesloten zonder grond | geen motivering of geen hersteltermijnafspraak | B | behandelaar |
| BTR-09 | Anonimisering niet getoetst | uitkomst geanonimiseerd zonder afgeronde toets | B | FG |
| BTR-10 | Afwijzing zonder rechtsmiddelen | geen verwijzing naar klachtrecht en beroep | B | behandelaar |
| BTR-11 | Termijnprestatie onder de norm | percentage binnen de maand onder de vastgestelde norm over een kwartaal | R | directie |
| BTR-12 | Verzoek zonder eigenaar | langer dan 14 dagen zonder toegewezen behandelaar | S | FG |

#### NIS2 (NIS)

| ID | Naam | Wat het controleert | Niveau | Naar |
|---|---|---|---|---|
| NIS-01 | Vroegtijdige waarschuwing te laat | geen 24-uursbericht binnen de termijn | B voor afsluiten; escalatie op T+18u | security officer |
| NIS-02 | Incidentmelding te laat | geen 72-uursmelding | escalatie naar directie | security officer |
| NIS-03 | Eindrapport niet ingepland | geen taak voor het eindrapport binnen een maand | S | security officer |
| NIS-04 | Lopend incident zonder voortgangsrapport | maand verstreken, incident loopt, geen voortgangsrapport | B | security officer |
| NIS-05 | Significantie niet beoordeeld | geen afgeronde significantiebeslisboom | B | security officer |
| NIS-06 | Toepasselijkheid niet vastgelegd | entiteit zonder NIS2-profiel bij het eerste incident | B | directie |
| NIS-07 | Afwijkende kennisnamemomenten | AVG- en NIS2-kennisname verschillen zonder toelichting | S | FG en security officer |

#### Organisatie, rol en overdracht (ORG)

| ID | Naam | Wat het controleert | Niveau | Naar |
|---|---|---|---|---|
| ORG-01 | FG niet aangemeld | geen aanmelding bij de AP vastgelegd voor de huidige rolbezetting | B op de directierapportage | directie |
| ORG-02 | Contactgegevens niet gepubliceerd of verouderd | publicatie ouder dan de rolwissel, of niet gecontroleerd in 12 maanden | B | FG |
| ORG-03 | Geen plaatsvervanger | rol zonder vastgelegde achtervang | S (B bij het vastleggen van afwezigheid) | directie |
| ORG-04 | Rolconflict | FG combineert een functie uit de conflictlijst | B zonder motivering en directiemelding | directie |
| ORG-05 | Geen rapportage aan de leiding | geen rapportage in de afgelopen 12 maanden | S | directie |
| ORG-06 | Opleidingsuren onder de norm | vastgelegde opleidingsuren onder de afgesproken ondergrens | R | directie |
| ORG-07 | Termijn valt in een afwezigheid | dossier met termijn in de afwezigheidsperiode zonder overdrachtsbeslissing | B | FG |
| ORG-08 | Overdracht zonder terugleeslus | dossier overgedragen zonder bevestigde read-back | B | ontvanger |
| ORG-09 | Autorisatie na vertrek | vertrokken medewerker met openstaande intrekking | S met escalatie | security officer |
| ORG-10 | Taak zonder geldige eigenaar | geen rol, of een rol zonder actuele bezetting | B | FG |
| ORG-11 | Externe FG overbelast | meer klanten of minder uren per klant dan de ondergrens | R | directie |
| ORG-12 | Toezegging zonder eigenaar of datum | toezegging aan de toezichthouder zonder eigenaar of einddatum | B | FG |

#### Integriteit van de applicatie zelf (SYS)

| ID | Naam | Wat het controleert | Niveau | Naar |
|---|---|---|---|---|
| SYS-01 | Sjabloon zonder review | wijziging gepubliceerd zonder review door een tweede persoon | B | beheerder |
| SYS-02 | Publicatie zonder impactanalyse | sjabloon- of taxonomiewijziging zonder uitgevoerde impactanalyse | B | beheerder |
| SYS-03 | Verouderde lijstwaarde | waarde op verouderd die nergens meer voorkomt | R | beheerder |
| SYS-04 | Ketenbreuk in het auditspoor | integriteitscontrole faalt | B op alle bewerkingen, direct alarm | beheerder |
| SYS-05 | Signaalregel wordt genegeerd | meer dan 80% genegeerd in een kwartaal | R | beheerder |
| SYS-06 | Waarschuwingsbudget overschreden | meer dan 5 onderbrekende meldingen per gebruiker per week | R (behandeld als defect) | beheerder |
| SYS-07 | Veld met veel dummywaarden | meer dan 30% "n.v.t." of een dummywaarde | R (behandeld als ontwerpdefect) | beheerder |
| SYS-08 | Onafgeronde onomkeerbare handeling | intentie vastgelegd, uitvoering niet bevestigd | B tot afronden of terugdraaien | gebruiker |
| SYS-09 | Termijnmodule niet geverifieerd | testgevallen van de termijnmodule niet groen na een update | B op alle termijnberekeningen | beheerder |
| SYS-10 | Klokafwijking | systeemklok wijkt af van de monotone referentie | S | beheerder |

**Totaal: 123 controleregels** — 17 REG, 12 GRO, 8 BEW, 9 DPIA, 13 VWO, 8 EER, 16 LEK, 12 BTR, 7 NIS, 12 ORG, 10 SYS.

---

## 6. Meten

Alle meting is lokaal. Er is geen telemetrie; de gegevens verlaten de machine niet. Wat gemeten wordt, dient twee doelen: aantonen dat de naleving verbetert, en aantonen dat het *ontwerp* werkt in plaats van de gebruiker harder loopt.

### 6.1 Foutmaten

| Maat | Definitie | Meetwijze | Startnorm |
|---|---|---|---|
| Veldfoutpercentage | onjuiste velden per 100 gecontroleerde velden | maandelijkse steekproef van 20 dossiers, hercontrole tegen de bron door een tweede persoon | < 1% na zes maanden gebruik |
| Correctiefrequentie binnen 24 uur | wijzigingen op een veld binnen een dag na vaststelling | uit het auditspoor | dalende trend; proxy voor slips |
| Geblokkeerde statusovergangen | aantal per 100 pogingen, per controleregel | uit het controlelogboek | dit is de teller van *gevangen* fouten en mag hoog zijn |
| Undo-gebruik | inline undo binnen 30 seconden na een actie | uit het auditspoor | stabiel; een piek wijst op een verwarrend scherm |
| Dossierverwisseling | bewerkingen in dossier A gevolgd door correctie in dossier B binnen 5 minuten | uit het auditspoor | naar nul; maat voor capture errors |

### 6.2 Volledigheid

| Maat | Definitie | Startnorm |
|---|---|---|
| Volledige dossiers | percentage dossiers met alle verplichte onderdelen ingevuld | register 100% van de vastgestelde regels; alle dossiers samen ≥ 90% |
| Dekking verplichte elementen | percentage aanwezige art. 30-, 33(3)- en 35(7)-elementen over alle dossiers | 100% bij vastgestelde dossiers |
| Vastgesteld en actueel | percentage registerregels vastgesteld én herzien binnen 12 maanden | ≥ 95% |
| Geërfd, niet geverifieerd | percentage regels met dat kenmerk | naar 0% binnen 6 maanden na de overname |
| Openstaande gemotiveerde uitstelafspraken | aantal "nog te bepalen"-waarden en hun gemiddelde leeftijd | dalend; geen enkele ouder dan de afgesproken datum |
| Verwerkers met volledige contractmapping | percentage VWO's met alle art. 28(3)-eisen op vindplaats én invulling | 100% |

### 6.3 Termijnen

| Maat | Definitie | Startnorm |
|---|---|---|
| Tijd tot melding | mediaan en p90 van kennisname tot verzending aan de AP | mediaan < 24 uur; p90 < 60 uur |
| Meldingen binnen 72 uur | percentage | 100% |
| Tijd tot art. 34-besluit | mediaan van AP-melding tot vastgelegd besluit | < 24 uur |
| Verzoeken binnen de maand | percentage betrokkenenverzoeken afgehandeld binnen de wettelijke termijn | ≥ 98%; dit is de kern-KPI die CEF 2025 aanbeveelt |
| Verlengingen | percentage verzoeken dat wordt verlengd, en percentage daarvan met tijdig bericht | verlengingsbericht 100% tijdig |
| Achterstandsleeftijd | leeftijd van het oudste openstaande verzoek en van het oudste openstaande datalekdossier | geen dossier ouder dan de wettelijke termijn |
| NIS2-ketentijdigheid | percentage 24-uurs-, 72-uurs- en eindrapportmomenten gehaald | 100% |
| Schoningsopdrachten op tijd | percentage uitgevoerd binnen 14 dagen na de uitvoerdatum | ≥ 90% |
| Openstaande overschrijdingen | aantal per dag, aan het begin van de dag | nul als haalbare dagelijkse toestand |

### 6.4 Kwaliteit van beslissingen

Dit is de belangrijkste categorie, omdat zij meet of de tweede en derde laag werken en niet slechts bestaan.

| Maat | Definitie | Wat het aantoont |
|---|---|---|
| Omkeerpercentage bij "niet melden" | percentage besluiten "niet melden" dat na tegenspraak of afkoelperiode alsnog omsloeg | > 0% bewijst dat de barrière werkt; 0% over lange tijd betekent bevestigingsdruk of een te lage drempel |
| Omkeerpercentage risicoweging | percentage risico-uitkomsten dat na tegenspraak omhoog ging | idem |
| Omkeerpercentage DPIA-toets | percentage uitkomsten "niet nodig" dat bij de tweede beoordeling omsloeg | idem |
| Afwijkingen van FG-advies | aantal per kwartaal, met besluitnemer | governance-indicator voor de directierapportage |
| Meldingen die achteraf gecorrigeerd moesten worden | aantal aanvullingen of correcties na verzending aan de AP | maat voor de kwaliteit van de time-out en de vierogencontrole |
| Verhouding gemeld / beoordeeld | percentage geregistreerde incidenten dat meldplichtig bleek | afwijking naar boven wijst op "voor de zekerheid melden"; naar beneden op onderregistratie |

### 6.5 Ontwerpgezondheid

| Maat | Definitie | Norm |
|---|---|---|
| Doorlooptijd per taak | mediane tijd voor: registerregel bijwerken, datalek registreren, verzoek intaken | < 2 min / < 3 min / < 1 min |
| Onderbrekende meldingen | aantal per gebruiker per week | ≤ 5 (SYS-06) |
| Negeerpercentage per signaalregel | getoond versus opgevolgd | regels boven 80% negeren gaan in de review (SYS-05) |
| Velden met dummywaarden | percentage invoer "n.v.t." of dummywaarde per veld | geen veld boven 30% (SYS-07) |
| Onderbrekingen tijdens focusmodus | aantal per kritieke sessie | dalend; hoge waarden zijn een organisatorische latente conditie |
| Ladderverdeling | percentage faalwijzen afgevangen op T1/T2 | ≥ 50%, en geen faalwijze uitsluitend op T4 of lager |
| Afgebroken sessies | percentage dossiers dat halverwege wordt verlaten en niet binnen 5 dagen wordt hervat | dalend; wijst op een te zware stap |

### 6.6 Latente condities

| Maat | Definitie |
|---|---|
| Sjabloonversies zonder impactanalyse | moet nul zijn (SYS-02 blokkeert, dus dit meet omzeiling) |
| Dossiers op een verouderde sjabloonversie | aantal en leeftijd |
| Leeftijd van de kennisbank | dagen sinds de laatste installatie van het kennispakket |
| Testdekking van de termijnmodule | aantal testgevallen per wettelijke termijn, inclusief zomertijd, schrikkeljaar en maandeinde; alle groen |
| Aantal beheerde lijstwaarden dat via de voorstelroute is aangemaakt | maat voor of de lijsten de werkelijkheid dekken |

### 6.7 Meetopzet

**Nulmeting vóór ingebruikname.** Vóór de eerste release wordt de bestaande situatie gemeten uit de huidige werkwijze: hoeveel registerregels hebben een bewaartermijn, hoeveel verwerkers hebben een overeenkomst, hoeveel verzoeken werden vorig jaar binnen de maand afgehandeld, wat was de mediane tijd tot melding. Zonder nulmeting is elke latere uitspraak over verbetering ongefundeerd.

**Kan het ontwerp de verbetering claimen?** Een gecontroleerd experiment is niet haalbaar. In plaats daarvan:

1. Elke maatregel wordt met een **datum** in de tijdreeks gemarkeerd. De vraag is of de betreffende maat op dat moment een knik vertoont, niet of hij over het geheel daalt.
2. **Gevangen fouten worden geteld**, niet alleen doorgelaten fouten. Een geblokkeerde statusovergang, een omgekeerd besluit na tegenspraak en een teruggedraaide actie binnen het herstelvenster zijn elk een fout die het dossier niet in kwam. Dit is de directe uitkomstmaat van het ontwerp.
3. **Contrastmeting bij ontsnappingen.** Waar een maatregel een gemotiveerde ontsnapping kent, wordt geteld hoe vaak die wordt gebruikt. Stijgt dat aandeel structureel, dan is de maatregel aan het verworden tot een formaliteit.

**Rapportagecyclus:**

| Frequentie | Product | Ontvanger |
|---|---|---|
| dagelijks | werkvoorraad met termijnen, gesorteerd op urgentie | behandelaars |
| wekelijks | openstaande blokkerende bevindingen en escalaties | FG |
| maandelijks | kwaliteitsrapportage: volledigheid, termijnprestatie, signaalregels met hun negeerpercentage, ontwerpgezondheid | FG en beheerder |
| per kwartaal | directierapportage: termijnprestatie, escalaties, afwijkingen van FG-advies, herhaalde oorzaakcategorieën, middelenregistratie, rolconflicten | directie |
| jaarlijks | verantwoordingsoverzicht: registerdekking, DPIA's, contracten, doorgiften, incidenten en de nulmeting-vergelijking | directie en bestuur |

---

## 7. Acceptatiecriteria voor nieuw werk

Deze criteria gelden als *definition of done* voor elk nieuw scherm, veld of proces in `dpo-fg-tool`. Ze zijn bewust afvinkbaar.

1. Elke geïdentificeerde faalwijze heeft een maatregel met een vastgelegde ladderpositie en een verantwoording waarom een hogere trede niet haalbaar was.
2. Geen faalwijze rust uitsluitend op T4, T5 of T6.
3. Elke maatregel is toegewezen aan een laag, met een genoteerde faalmodus en een volgende laag met een *andere* faalmodus.
4. Geen vrij tekstveld waar een beheerde lijst kan; waar vrije tekst blijft, staat zij náást het gestructureerde veld, nooit ervoor in de plaats.
5. Geen waarschuwing op een objectief bepaalbaar feit.
6. Elke blokkerende melding benoemt het gevolg, de wettelijke grond en biedt twee uitwegen waarvan één de situationele afwijking legaliseert.
7. Elk nieuw onderbrekend signaal past binnen het waarschuwingsbudget, of een bestaand signaal verdwijnt.
8. Elke wettelijke termijn wordt berekend door de termijnmodule, met testgevallen inclusief zomertijd, schrikkeljaar en maandeinde, en wordt getoond als datum, tijd en tijdzone.
9. Elke onomkeerbare handeling heeft een time-out met itemsgewijze bevestiging, plus ten minste één van: tweede persoon, uitgestelde uitvoering, herstelvenster.
10. Geen bevestigingsdialoog waar undo mogelijk is; niets wordt hard verwijderd.
11. Elke afgeronde stap die een wettelijke vervolgstap oproept, maakt die vervolgstap automatisch aan als taak met eigenaar en termijn.
12. Elk nieuw veld heeft een gemeten of geschatte doorlooptijdbijdrage; de taaknormen uit §6.5 blijven gehaald.
13. Elk nieuw sjabloon, elke nieuwe taxonomie en elke nieuwe beslisboom is versiebeheerd, heeft een review-gate en een impactanalyse.
14. Concept is een geldige toestand; de nieuwe functie verliest geen werk bij navigeren, sessieverloop of uitval.
15. Voor elke nieuwe controleregel is vastgelegd op welk niveau zij start (altijd R), naar welke rol zij routeert, en op welk bewijs zij mag promoveren.

---

## 8. Grenzen en restrisico

Eerlijkheid over wat dit ontwerp niet oplost, hoort in het projectplan.

| Grens | Toelichting | Wat er wél gebeurt |
|---|---|---|
| Fouten buiten de applicatie | De brief uit de printstraat, de autocomplete in de mailclient, het verborgen werkblad, de onterechte inzage door een geautoriseerde medewerker en het invoeren van gegevens in een externe dienst gebeuren niet in deze tool. | Foutbestendige registratie, verplichte oorzaakcategorie, patroondetectie (LEK-13) en afdwingbare maatregelopvolging met bewijs. |
| Eenpersoons-FG | Het vierogenprincipe is niet uitvoerbaar. | Afkoelperiode als expliciet zwakkere laag, met vastlegging van het gebruik en rapportage aan de directie. |
| Onjuiste opgaven van derden | Een leverancier die zijn toegangslanden of subverwerkers verkeerd opgeeft, kan de tool niet corrigeren. | Opgave met datum en bron vastgelegd, meldplicht contractueel geborgd, jaarlijkse controle als taak; de fout wordt aantoonbaar de tekortkoming van de leverancier. |
| Inhoudelijk oordeel | Een verkeerd gekozen grondslag is syntactisch volmaakt. Validatie staat hier machteloos. | Criteria in beeld op het beslismoment, tegenvoorbeelden, verplichte motivering, vierogen op oordeelsvelden, en meting van het omkeerpercentage. |
| Capaciteit | Geen ontwerp compenseert een structureel te dun bezette FG-functie. | Middelenregistratie, plaatsvervangerbewaking en zichtbare achterstanden in de directierapportage; het capaciteitsprobleem wordt een bestuurlijk besluit in plaats van een individueel probleem. |
| Genormaliseerde deviatie | Herhaald afwijken zonder gevolgen wordt de norm. | Elke ontsnapping wordt geteld; een stijgend aandeel gemotiveerde afwijkingen is zelf een rapportregel en agendapunt in de kwartaalreview. |

---

*Einde hoofdstuk. De controleregels uit §5 vormen samen met de ladderverantwoording uit §0.3 de brug naar het technisch ontwerp: elke regel wordt geïmplementeerd als een benoemde, testbare eenheid met een vast niveau en een vaste ontvangerrol.*