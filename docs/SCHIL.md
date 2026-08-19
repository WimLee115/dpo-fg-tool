# De grafische schil

*Ontwerpdocument voor de grafische schil van `dpo-fg-tool`. Het bouwt voort op het voorstel "de werkbak als enige beginpunt", verwerkt wat de beoordelaars uit de twee andere voorstellen wilden overnemen, en herstelt de zwakke plekken die zij in het winnende voorstel aanwezen. Alle getallen en verwijzingen in dit stuk zijn nagerekend tegen de broncode zoals die op 19 augustus 2026 in de werkruimte staat; waar dat niet kon, staat dat er.*

---

## Waarom deze opzet

De schil kent één beginpunt, en dat is de werkbak: één lijst met openstaande verplichtingen over AVG, UAVG, Cbw, Woo, Wpg en het bestuursrecht heen, in een vaste volgorde die de gebruiker niet kan omdraaien. De reden om dáár te beginnen en niet bij een kast met dossiersoorten of bij zes benoemde plaatsen, is dat dit het enige is wat de bedieningsschil principieel niet kan. `dpofg register vul` is in een terminal sneller dan in een formulier, en `dpofg termijn` geeft het anker, de duur, het verstrijkmoment in CEST, de grondslag en de rekenregel al netjes terug. Wat de opdrachtregel niet kan, is die uitkomsten van tweeëntwintig opdrachtgroepen samenvoegen tot één lijst die zichzelf bijhoudt terwijl vijf klokken op vier verschillende ankers doorlopen. Dat is het bestaansrecht van de schil, en al het overige in dit ontwerp is daaraan ondergeschikt.

Elke regel in die lijst draagt drie dingen die niet weg te configureren zijn: de citeerbare grondslag, de toegepaste rekenregel met het anker, en de eigenaar als rol met de bezetting erbij. Die drie staan inline in de regel en niet achter een tooltip, want een tooltip is een verborgen feit voor toetsenbordgebruik, voorlezen en schermafdruk — en `PLAN.md` §4.2 eist de grondslag toch al als citeerbare tekst per werkbakregel.

**De belangrijkste correctie ten opzichte van het winnende voorstel: de werkbak is geen schermwerk.** In de motor bestaat `AfgeleideVerplichting` uitsluitend voor incidenten; `verplichtingen_uit_incident` in `crates/dpofg-domain/src/klokken.rs` is de enige producent, en elke andere dossiersoort rekent haar termijn binnen haar eigen opdrachtbestand uit. Er is bovendien geen enkele handeling die een verplichting afdoet: `gemeld_op` wordt in de productiecode nergens gezet, `dpofg incident` kent geen `melden`, en `AfgeleideVerplichting` heeft geen voldaan-toestand. Een werkbak bouwen als scherm boven die laag levert een lijst op waaruit niets kan verdwijnen. Daarom staat in dit ontwerp `dpofg werkbak --json` in de kern, vóór er één scherm wordt getekend. Dat heeft drie voordelen: de schil wordt een weergave in plaats van een tweede motor, de aggregatie is te dekken met golden tests per dossiersoort zoals acceptatiecriterium 8 verlangt, en de opdrachtregel krijgt dezelfde lijst — waarmee cron, mail, een afdruk en een agenda-export gratis meekomen op precies het punt waar de schil niets kan, namelijk als hij dicht is.

De schil rendert en rekent niet. Klokken, termijnen, aggregaten en de vergrendeling draaien in Rust; in de webview zitten geen sleutels, geen crypto, geen bestands-I/O en geen timers, zodat een opgeschorte webview-timer nooit een gemiste vierentwintig uur kan worden. Bestandskiezers lopen uitsluitend via de dialoogplug-in, en de brug is een allowlist van getypeerde commando's zonder jokertekens, conform `docs/PLATFORMONDERSTEUNING.md`.

Eén dossier tegelijk bewerken, maar niet één dossier tegelijk lezen. Het winnende voorstel verbood elke route van dossier naar dossier en brak dat verbod in zijn eigen schets meteen met een knop naar de effectbeoordeling. Die tegenstrijdigheid is hier opgelost in het voordeel van de knop: `FOUTBESTENDIGHEID.md` §3.4 en §3.6 schrijven zulke routes juist voor, omdat een afgeleide verplichting nu eenmaal in een ánder dossier landt. Wat beschermd wordt is het *bewerken*, niet het kijken. Naast het dossier dat u bewerkt opent een leespaneel met de geraakte registerregel, de verwerkersovereenkomst of de contactgegevens; dat paneel is alleen-lezen en breekt de focusmodus niet. Wilt u het andere dossier werkelijk bewerken, dan is dat een expliciete handeling die de focusmodus zichtbaar verbreekt en in het logboek landt.

### Verantwoording per nieuw mechanisme

`FOUTBESTENDIGHEID.md` §0.3 verlangt voor elke maatregel vijf dingen: welke fout zij vangt, op welke trede zij zit, waarom een hogere trede niet haalbaar was, in welke laag zij zit en waardoor zij faalt, en welke volgende laag die faalmodus opvangt. Zonder punt drie en vijf is een maatregel niet af. Het winnende voorstel sloeg die toetsnorm over voor tien nieuwe mechanismen; hieronder staat zij alsnog, compact.

| Mechanisme | Vangt | Trede | Waarom niet hoger | Faalt bij | Volgende laag |
|---|---|---|---|---|---|
| Voetregel "niet in deze lijst" | lege lijst gelezen als "klaar" | T1 op de weergave: er bestaat geen render van de werkbak zonder die regel | de onderliggende onvolledigheid zelf is niet af te dwingen | de gebruiker leest de regel wel en handelt niet | volledigheidsmaten in de maandrapportage (§6.2) |
| Eén regel per lopende klok, `spoor n van m` | post-completion error: één spoor afgehandeld, dossier "klaar" gewaand | T2, afgeleid uit `verplichtingen_uit_incident` | het besluit per spoor is menselijk | een verplichting zonder anker | `wacht_op_anker` blijft zichtbaar; LEK-regels |
| Buiten-beeldstrook plus noemergeschiedenis | krimpende noemer gelezen als voortgang | T2: de weggevallen waarde, de oorzaak en het tijdstip worden bewaard | de krimp kan volstrekt legitiem zijn; blokkeren zou onjuist zijn | krimp die klopt maar het dossier leeg maakt | rapportregel bij meer dan dertig procent krimp, startend op rapportniveau |
| Veertien losse blokjes, geen doorlopende balk | teller vertaald in een percentage | T1: de brug levert geen deling; `Volledigheid::percentage()` wordt door de schil niet aangeroepen | de gebruiker kan zelf delen | iemand rekent het alsnog uit | statuspaneel per soort met benoemde tellers |
| Twee vensters met eigen capabilities per vensterlabel | capture error: werken in de verkeerde kluis | T1: het FG-venster kent de organisatiecommando's niet | — | verkeerde dossier binnen het juiste venster | contextbalk met onderscheidend kenmerk (T4) en het logboek |
| Slot noemt de kluis; nooit een kruisproef bij inloggen | verkeerde wachtwoordzin, en het orakel dat daaruit ontstaat | T1: het proberen op de andere kluis is geen bestaande handeling | — | twee kluizen met dezelfde zin | kruisproef bij aanmaken (T3, `crates/dpofg-cli/src/opdrachten/fg.rs`) |
| Weigering bij het veld met twee uitwegen | situationele violation: iets verzinnen om verder te komen | T3, blokkerend bij de statusovergang | de juiste waarde is een organisatorisch besluit | een plausibele maar onjuiste waarde | BEW-04 en de herzieningstermijn (T5) |
| Time-out, gedifferentieerd per handeling | slip op het laatste moment | T5 in de ladder, laag 5 in het lagenmodel | verzending is per definitie de laatste stap | habituatie bij hoge frequentie | annuleervenster en de status "Gemeld" pas na ontvangstbevestiging |
| Verstreken als eigen band met permanent kenmerk | een gemiste termijn die uit beeld zakt | T5, achteraf-detectie | de termijn is al verstreken; er valt niets meer te blokkeren | het kenmerk wordt genegeerd | termijnmaten in de maandrapportage (§6.3) |
| Rolfilter op de werkbak | ruis bij de FG, signalen die hun ontvanger niet bereiken | T4: het is een weergave, geen toegangsbeperking | er is geen rollen- en aanstellingsregister en geen authenticatie per rol | de gebruiker zet de filter uit of neemt andermans werk over | `Ontvangerrol` per controleregel in de motor; contrastmeting per regel |
| Herkomstmarkering "geërfd, niet geverifieerd" | overgenomen waarde die op een besluit lijkt | T3 blokkerend bij vaststellen, plus T1 doordat het kenmerk niet wisbaar is | of de waarde klopt, is een oordeel | bevestigen zonder te kijken | het kenmerk reist mee in elke export |
| Handelingsmanifest als bouwlint | een knop zonder opdracht eronder | T1 op bouwtijd: de bouw faalt | — | een handeling die wél bestaat maar verkeerd is aangeroepen | getypeerde brug plus golden tests |

---

## Wat de schil niet doet

| Verboden patroon | Wat er in plaats daarvan staat | Waarom |
|---|---|---|
| Nalevingspercentage, groene ring, stoplicht, "voldoet" | Per dossier een teller met noemer: `11 van 14 onderdelen vastgelegd`, met de drie ontbrekende onderdelen direct eronder en een knop per stuk. De balk bestaat uit veertien losse blokjes, één per onderdeel. | Een proportie is niet af te lezen uit losse blokjes, en de blokjes veranderen zichtbaar mee als een conditioneel antwoord het aantal verlaagt. `FOUTBESTENDIGHEID.md` §3.6 wil een teller en voortgang, geen verwijt en geen score. |
| Eén totaal over de dossiersoorten heen | Per soort een statuspaneel met klikbare tellers (`38 vastgesteld`, `3 zonder bewaartermijn`, `5 geërfd, niet geverifieerd`), elk met de grondslag als koptekst van de gefilterde lijst. Daaronder één regel: *"Er is met opzet geen totaal over deze soorten heen"*, met een uitklap die uitlegt waarom. | Het statuspaneel per register is voorgeschreven door §3.6 en overbrugt de evaluatiekloof; het winnende voorstel schafte het per ongeluk af door het scoreverbod te ver door te trekken. Wat verboden is, is de optelling over ongelijksoortige dossiers, niet het aggregaat binnen één soort. `prognose --factoren` levert zulke tellingen al en blijft dus bestaan. |
| "Nog 2 dagen" | `uiterlijk do 20-08-2026 09:14 CEST`, altijd datum, tijd en tijdzone, met de rekenregel en het anker eronder. Resterende tijd staat er hooguit bij, nooit in plaats van. | Deadlines zijn absolute momenten in Rust met `chrono-tz` uit de binary; een verzette systeemklok of een hervatting uit slaapstand verschuift niets. |
| Permanent rood dashboard | De vier toestanden uit §3.5, plus verstreken als eigen band met een permanent, niet weg te klikken kenmerk op het dossier. | Een dashboard dat structureel alarmeert wordt behang. Er is geen kleur die harder wordt; als de bak structureel onhaalbaar is, is dat een capaciteitsprobleem dat gerapporteerd hoort te worden. |
| Sterretjes en "verplicht veld" als foutmelding | Niets bij verplichte velden; onderaan elk blok staat wat optioneel is. | Waar bijna alles verplicht is, markeert het sterretje niets. |
| Modal, pop-up, "Weet u het zeker?" | Oordeelssignalen staan als strook in het dossier, permanent, met een oplossingsknop en een route "gemotiveerd accepteren met eigenaar en herbeoordelingsdatum". Bevestigen bestaat alleen in de drie gevallen van §3.8, en dan als time-out met itemsgewijze bevestiging. | Een herhaalde bevestiging wordt door habituatie een verlengstuk van de klik ervoor, en beschermt bovendien alleen tegen slips en niet tegen een verkeerd plan. |
| De weigering als schermvullend tussenscherm | De weigering is een gedeelde component die **bij het veld of bij de geblokkeerde knop** verschijnt, met het gevolg, de wettelijke grond en twee uitwegen waarvan één de afwijking legaliseert. | §3.9 zegt letterlijk dat de melding bij het veld staat en niet bovenaan de pagina. Een apart scherm staat daar nog verder vandaan, en juist deze melding is de meest geziene van de applicatie — precies waar habituatie ontstaat. |
| Breadcrumb als plaatsbepaling | Een visueel dominante contextbalk met dossiernummer, naam, status en een onderscheidend kenmerk (systeem of proceseigenaar). | Twee gelijkende incidenten moeten op de bovenste regel te onderscheiden zijn zonder te scrollen; dat is de maatregel tegen capture errors. |
| Type-ahead die stilzwijgend een waarde aanmaakt | Vrije invoer opent de duplicaatlijst met fuzzy match en de knoppen "Deze gebruiken" en "Toch nieuw". "Toch nieuw" leidt naar een klein formulier met KvK-nummer, vestigingsland en rol, en bij een beheerde taxonomie wordt de nieuwe waarde als voorstel vastgelegd tot een beheerder haar goedkeurt. | §3.2 eist duplicaatcontrole vóór aanmaak én goedkeuring voor beheerde lijsten. Zonder die tweede stap breidt iedere gebruiker de taxonomie eenzijdig uit en lopen registers alsnog uit elkaar. |
| Vrij tekstveld voor grondslag, bewaartermijn, deadline of registratiemoment | Zes radio's plus een gescheiden art. 9 lid 2-blok. De bewaartermijn is getal, eenheid, startgebeurtenis en grondslag — met een derde vorm voor "zolang een toestand duurt, en daarna een duur", omdat het domein die variant kent en de gebruiker anders naar een verzonnen vaste duur wordt geduwd. Deadlines zijn afgeleid; het registratiemoment is een systeemgegeven. | Waar de laag eronder het kan weten, mag de schil het niet vragen. |
| Grijs uitgeschakeld zonder uitleg | Zichtbaar vergrendeld met de reden en een route: `Vaststellen — vergrendeld: 2 onderdelen ontbreken [ Tonen ]`. Een knop die pas na een beoordeling betekenis heeft, bestaat vóór die beoordeling helemaal niet. | Grijs is een mededeling zonder inhoud; afwezig is eerlijker dan onbereikbaar. |
| Helppagina, cursus, tooltipmuseum | Naslag opent als zijpaneel náást het veld waar de beslissing valt, met de wettelijke tekst en de rekenregel. Dezelfde inhoud is vóór ontgrendeling bereikbaar. | Uitleg op afstand van de beslissing is uitleg die niemand leest. |
| Notificatiestroom per dossier | Eén werkbak, één contactmoment per dag, maximaal vijf onderbrekende meldingen per week. Nieuwe signalen beginnen op rapportniveau en promoveren alleen op bewijs. | Overschrijding is een defect (SYS-06), geen ruis die harder mag; `crates/dpofg-rules/src/budget.rs` levert de meetwaarde al. |
| Bureaubladmeldingen | Geen. In plaats daarvan bij elke ontgrendeling een strook "wat er is gebeurd terwijl de tool dicht was", en een dagafsluiting die de harde momenten van vanavond en morgenochtend naar buiten zet als agenda-item of afdruk. | Een melding suggereert bewaking. De applicatie bewaakt niets als zij dicht is, en dat mag niet verdoezeld worden. |
| Globale schakelaar `--uitgebreid` | Het woord komt in de schil niet voor. De prognosevariant heet voluit "Per horizon elke eis tonen"; de globale variant wordt "Toon herkomst en berekening" en is een sessieschakelaar in het naslagpaneel. | De vlag bestaat vandaag twee keer met twee betekenissen (`crates/dpofg-cli/src/main.rs` en `crates/dpofg-cli/src/opdrachten/prognose.rs`), waarbij de globale wordt overschaduwd. Eén woord voor twee dingen is precies hoe iemand de verkeerde uitvoer aan een bestuur laat zien. |
| Export die completer oogt dan de werkelijkheid | Dezelfde tellers, dezelfde ontbreeklijst en dezelfde kenmerken in de bundel als op het scherm, met de weglatingen in het manifest en een niet over te slaan inhoudslijst vóór het verstrekken. | §3.6: er is geen weergave waarin het dossier completer lijkt dan het is. |
| Prullenbak en "verwijderen" | Intrekken met einddatum en verplichte reden, gevolgd door dertig seconden inline ongedaan maken. De knop is eerlijk: terugzetten schrijft de vorige waarde als nieuwe wijziging, de intrekking blijft in het logboek. | Het datamodel verwijdert niets; een schil die suggereert dat er iets weggaat, liegt over de opslag. |

Drie van deze regels worden afgedwongen in de bouwstraat en niet in een ontwerpafspraak, omdat een afspraak de eerste haastige vrijdag niet overleeft. De visuele klasse voor onomkeerbare handelingen is toegewezen aan een gesloten lijst en de generator weigert haar elders. De uitklapcomponent accepteert geen kind dat als verplicht onderdeel geregistreerd staat, zodat een wettelijk minimum nooit achter progressieve onthulling kan verdwijnen. En elke knop verwijst naar een handeling in een manifest dat de servicecrate uitgeeft: een schermdefinitie met een onbekende handeling laat de bouw falen. Datzelfde manifest levert de regelcodes, niveaus, ontvangerrollen en regeltellingen, zodat een getal in een scherm nooit uit schermtekst komt. Dat had de fouten gevangen die in de drie voorstellen zelf zijn aangetroffen: een regelcode die "concept blijft liggen" betekent op de plek van een ontbrekende bewaartermijn, en een catalogus van 147 regels waar er 78 zijn.

---

## De schermenkaart

| Scherm | Bereikbaar vanaf | Wat het is |
|---|---|---|
| **Slot** (per kluis één) | start; na vergrendelen | Ontgrendelen van één met naam genoemde kluis. Geeft toegang tot de dertien schermen die geen kluis openen. |
| **Blokkadescherm** | in plaats van alles | SYS-04 (ketenbreuk), SYS-08 (onafgeronde onomkeerbare handeling) en SYS-09 (termijnmodule niet geverifieerd). Zolang zo'n toestand geldt, is er geen ander scherm. |
| **Herstelrapport** | eerste scherm na uitval | Wat er is hersteld, met diff, niet weg te klikken zonder gekeken te hebben. |
| **Werkbak** | direct na ontgrendelen | Eén lijst, vaste sortering, met rolfilter en zoekfilter die de sortering niet raken. |
| **Dossiervenster** | vanaf een werkbakregel of vanaf Vastleggen | Eén dossier in bewerking, met contextbalk, volledigheidsteller met noemergeschiedenis, klokkentabel, signaalstrook en het werkblad. |
| **Leespaneel** | naast het dossiervenster | Alleen-lezen weergave van een gekoppeld dossier. Breekt de focusmodus niet. |
| **Werkblad** (zestien varianten) | binnen het dossiervenster | Het enige dat per dossiersoort verschilt; één frame, zestien invullingen. |
| **Time-outscherm** | vanaf een handeling met externe werking | Itemsgewijze bevestiging met de waarde ernaast. Aantal items en uitstel verschillen per handeling (zie verderop). |
| **Complementaire beoordeling** | vanaf een handeling die vier ogen vergt | Toont de onderliggende feiten plus de vraag, nooit het ingevulde formulier. |
| **Naslagpaneel** | overal, ook op het slot | Twaalf bestaande weergaven plus de systeemcontrole. Opent naast het veld waar de beslissing valt. |
| **Statuspaneel per soort** | vanaf de werkbak of het naslagpaneel | Tellers per dossiersoort, elk klikbaar naar de gefilterde lijst met de grondslag als koptekst. |
| **Ketenvenster** | vanaf de kop van de werkbak | Logboek met filters, verificatierapport, anker plaatsen. |
| **Uitlevervenster** | vanaf een dossier of vanaf de werkbakkop | `dossier <map>` en `prognose --export`, met de vijf waarborgen uit §3.7 rij 8. |
| **Persoonlijk FG-venster** | eigen slot, eigen venster, eigen wachtwoordzin | Werkbak en dossiers van het persoonlijke dossier. Nooit in hetzelfde venster als de organisatiekluis. |
| **Wat deze versie nog niet doet** | vanaf de werkbakkop | Per dossiersoort: volledig, alleen-lezen of afwezig. Dezelfde aanduiding staat op de plek zelf. |

De dertien schermen zonder kluis zijn: `termijn`, `pakket toon`, `pakket voorbehoud`, `controle --dekking`, `kluis sleutel`, `verzoek lezingen`, `woo gronden`, `leverancier eisen`, `zorgplicht onderdelen`, `zorgplicht kaders`, `doorgifte instrumenten` en `fg gronden` — twaalf weergaven die vandaag al zonder wachtwoord draaien, aangevuld met de systeemcontrole, die nieuw is. Dat aantal staat hier één keer en wordt verder uit het manifest getrokken, want in de drie voorstellen liep dezelfde telling drie keer uiteen, en dat is geen detail in een product waarvan de kernstelling is dat een teller nooit decoratief mag zijn.

Nieuw werk begint niet met een menu van dossiersoorten. `Vastleggen` is altijd bereikbaar, ook tijdens de focusmodus, en vraagt precies twee dingen: wat er is gebeurd en wanneer het binnenkwam. Pas dáárna volgen de vier feitelijke vragen die de soort bepalen — is er iets misgegaan, is er iets binnengekomen, gaan we iets nieuws doen, moeten we iets aantonen. De volgorde is met opzet zo: §3.10 verbiedt drempels bij intake, en vier routeringsvragen vóór het eerste veld zijn een drempel. Naast `Vastleggen` staat een opdrachtvenster dat op kenmerk, naam en systeem springt; de blinde index in de opslaglaag maakt dat mogelijk en zonder zo'n route is de schil op navigatie langzamer dan een shell met geschiedenis.

### De werkbak

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ ORGANISATIEKLUIS  Gemeente Noorderveld · algemeen           [ Vergrendelen ] │
│ Kennispakket 0.3-start · consolidatiedatum 01-08-2026 (18 dagen oud)         │
│ Voorbehoud: 18 punten te verifiëren vóór gebruik            [ Tonen ]        │
├──────────────────────────────────────────────────────────────────────────────┤
│ WERKBAK — woensdag 19 augustus 2026, 09:14 CEST                              │
│ Berekend over alle compartimenten van deze kluis (algemeen, vertrouwelijk).  │
│ Toon: [ alles ] [ mijn rol: functionaris ]        zoeken: [            ]     │
├──────────────────────────────────────────────────────────────────────────────┤
│ ▶ IN UITVOERING — nog te annuleren                                           │
│   2026-0117  AP-melding klaargezet 09:12       annuleren kan nog 3:41        │
│                                                       [ Annuleren ]          │
├──────────────────────────────────────────────────────────────────────────────┤
│ VERSTREKEN                                                                   │
│  (geen)  Een verstreken termijn verdwijnt hier nooit vanzelf; hij verdwijnt  │
│          door afhandelen of intrekken met reden, en laat een permanent       │
│          kenmerk op het dossier achter.                                      │
│                                                                              │
│ ONHERSTELBAAR, VANDAAG                                                       │
│  2026-0117  Datalek verzuimportaal                        spoor 1 van 4      │
│             Vroegtijdige waarschuwing meldketen · uiterlijk wo 19-08 21:00   │
│             24 u vanaf vaststelling significant di 18-08 21:00 CEST          │
│             grondslag: meldketen zorgplicht, eerste bericht — het artikel-   │
│             nummer is nog niet vastgesteld (voorbehoud, punt 5)              │
│             eigenaar: security officer (M. de Wit)             [ Openen ]    │
│                                                                              │
│ ONHERSTELBAAR, DEZE WEEK                                                     │
│  2026-0117  Datalek verzuimportaal                        spoor 2 van 4      │
│             Melden aan de AP · uiterlijk do 20-08-2026 09:14 CEST            │
│             art. 33 lid 1 AVG · 72 u vanaf ontvangst verwerkersmelding       │
│             ma 17-08 09:14 CEST                                              │
│             eigenaar: functionaris (u)                         [ Openen ]    │
│                                                                              │
│  2026-0083  Inzageverzoek Jansen · uiterlijk vr 21-08-2026 23:59 CEST        │
│             art. 12 lid 3 AVG · 1 maand vanaf ontvangst do 21-07             │
│             lezing "kalendermaand", door u gekozen 22-07     [ Openen ]      │
│                                                                              │
│ ACHTERSTALLIG SINDS                                                          │
│  ZP-04      Bedrijfscontinuïteit · verplicht sinds 15-08-2026 (4 dagen)      │
│             achterstand gerekend vanaf de inwerkingtreding, niet vanaf het   │
│             onderwerp zelf · art. 21 lid 3 Cbw, onderdeel c volgens de       │
│             indeling in het kennispakket (te verifiëren, punt 14)            │
│             eigenaar: nog toe te wijzen — dat is zelf een taak  [ Openen ]   │
│                                                                              │
│ VERVALPROGNOSE 30 DAGEN                                                      │
│  ZP-07      Pentest vervalt 12-11-2026 · doorlooptijd 60 dagen, opgegeven    │
│             door M. de Wit op 03-06-2026 → start uiterlijk zo 13-09-2026     │
│                                                                [ Openen ]    │
│                                                                              │
│ OVERIG, OP TERMIJN                                          18 regels  [ ▾ ] │
├──────────────────────────────────────────────────────────────────────────────┤
│ Niet in deze lijst: 7 dossiers zonder lopende klok · 12 registerregels in    │
│ concept · 5 geërfd, niet geverifieerd · 14 van de 78 controleregels hebben   │
│ geen evaluatiefunctie en hebben dus nog nooit iets kunnen beoordelen         │
│                                                              [ Tonen ]       │
│                                                              [ Vastleggen ▸ ]│
└──────────────────────────────────────────────────────────────────────────────┘
```

Vijf dingen in deze schets vergen uitleg. De strook **In uitvoering** is de plaats die `FOUTBESTENDIGHEID.md` §3.7.1 eist voor de prominente annuleerknop na een melding; hij is vastgezet bóven de banden en is geen zesde categorie, zodat de vaste sortering intact blijft. De band **Verstreken** is een afwijking van `PLAN.md` §4.2, dat vijf categorieën kent: §3.5 legt "verstreken" vast als eigen toestand met een permanent kenmerk, en die toestand had in het winnende voorstel nergens een plek. De **vervalprognose** houdt de dertig dagen van `PLAN.md` aan; de doorlooptijd verschijnt alleen wanneer een mens hem heeft opgegeven, met naam en datum erbij, want `Vervalpunt` in `crates/dpofg-report/src/prognose.rs` kent geen doorlooptijdveld en een standaardwaarde zou verzonnen precisie zijn. De **compartimentsregel** zegt wat vandaag waar is: in `crates/dpofg-store/src/kluis.rs` ontsluit de kluissleutel alle compartimenten, dus wie de kluis opent, ziet alles. De regel "geen sleutel voor: vertrouwelijk" uit het winnende voorstel beschrijft een toestand die nog niet kan bestaan en staat er daarom niet. En de **voetregel** telt regels zonder evaluatiefunctie — dat is wat `controle --dekking` meet, en dus wat er staat.

### Het dossiervenster

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ 2026-0117  Datalek verzuimportaal · Beoordeeld · systeem HR-portaal          │
│ proceseigenaar A. Bakker       FOCUS ACTIEF — werkbak opzij     [ Verlaten ] │
├──────────────────────────────────────────────────────────────────────────────┤
│ ▌▌▌▌▌▌▌▌▌▌▌░░░  11 van 14 onderdelen vastgelegd                              │
│ Noemer 14 · ongewijzigd sinds het aanmaken                        [ Waarom ] │
│ Ontbreekt: aantal betrokkenen [ Invullen ] · oorzaakcategorie [ Invullen ]   │
│            besluit art. 34 [ Beoordeling openen ]                            │
├──────────────────────────────────────────────────────────────────────────────┤
│ SIGNAAL EN KENNISNAME                                                        │
│  Signaal binnengekomen  ma 17-08-2026 08:40 CEST (melding van de verwerker)  │
│  Kennisname vastgelegd  ma 17-08-2026 09:14 CEST                             │
│  Verificatieperiode 34 minuten, gemotiveerd            [ Motivering tonen ]  │
├──────────────────────────────────────────────────────────────────────────────┤
│ KLOKKEN                                        vier sporen, twee regimes     │
│  AP-melding        do 20-08-2026 09:14 CEST  art. 33 lid 1 AVG               │
│                    anker: ontvangst verwerkersmelding ma 17-08 09:14         │
│  Intern register   do 20-08-2026 09:14 CEST  art. 33 lid 5 AVG               │
│                    geldt ook wanneer er niet wordt gemeld                    │
│  Waarschuwing      wo 19-08-2026 21:00 CEST  meldketen zorgplicht            │
│                    anker: vaststelling significant di 18-08 21:00            │
│  Melding zorgplicht vr 21-08-2026 21:00 CEST  idem anker                     │
│  Eindrapport       wacht op anker: de verzending van de incidentmelding      │
│                                                                              │
│  Contractueel: de verwerker meldde 31 uur na het optreden bij hem; zijn      │
│  overeenkomst noemt 24 uur. LEK-16 signaleert die overschrijding. Uw eigen   │
│  termijn verschuift daardoor niet: het anker is de ontvangst van zijn        │
│  melding, niet het moment waarop het bij hem gebeurde.                       │
│  Afronden van één spoor sluit dit dossier niet.                              │
├──────────────────────────────────────────────────────────────────────────────┤
│ STAAT OPEN — blijft staan tot opgelost of gemotiveerd geaccepteerd           │
│  Gezondheidsgegevens vastgelegd; uitzonderingsgrond art. 9 lid 2 ontbreekt   │
│  op de gekoppelde registerregel 0412-K.                                      │
│           [ 0412-K ernaast lezen ]  [ Naar 0412-K ]  [ Gemotiveerd accepteren]│
├──────────────────────────────────────────────────────────────────────────────┤
│  1 Signaal  2 Aantasting  3 Feiten •  4 Weging  5 Afronden                   │
│  ─────────────────────────────────────────────────────────────────────────   │
│  U was gebleven bij 3 Feiten. Laatst gewijzigd vandaag 08:47 door u.         │
│  Sindsdien is de AP-klok van 34 naar 24 uur gelopen.                         │
│                                                                              │
│  [ werkblad ]                                                                │
├──────────────────────────────────────────────────────────────────────────────┤
│ Naslag naast dit veld: art. 33 AVG · meldketen zorgplicht · rekenregel 72 u  │
│ [ Kopieer de stand ]  één A4 platte tekst, met ankers en grondslagen     [ ▸]│
└──────────────────────────────────────────────────────────────────────────────┘
```

Het hervattingsanker noemt op een kluis met één houder niet dat er geen wijzigingen van anderen waren — dat is altijd waar en daarmee ruis — maar de enige verandering die er wel was: hoeveel de klok is gelopen. "Kopieer de stand" levert één A4 platte tekst voor het crisisoverleg en de mail; die handeling verlaat de organisatie feitelijk wel en krijgt daarom een regel in het auditspoor, ook al is het geen bundel.

### Het slot

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  ORGANISATIEKLUIS                                                            │
│  ~/.local/share/nl.dpofgtool.app/data/kluis.db   (lokaal, ext4)              │
│  installatiesleutel  9f3a 2c81 44de 0b76 · laatste anker 12-08-2026          │
│  In deze uitgave opent één wachtwoordzin alle compartimenten van deze kluis. │
│                                                                              │
│  Wachtwoordzin van de ORGANISATIEKLUIS                                       │
│  [                                                          ]                │
│                                              [ Ontgrendelen ]                │
│  Deze zin wordt niet op een andere kluis geprobeerd.                         │
│                                                                              │
│  Uw persoonlijke FG-dossier heeft een eigen zin en opent in een eigen        │
│  venster.  ~/dpo-fg/fg-dossier.db · sleutel 4a7c 91e2 …                      │
│                                              [ Persoonlijk dossier openen ]  │
│                                                                              │
│  Zonder ontgrendelen bruikbaar                                               │
│  Termijnen · Kennispakket · Voorbehoud · Regeldekking · Kluissleutel ·       │
│  Twee lezingen van de maandtermijn · Weigeringsgronden Woo · Art. 28 lid 3 · │
│  Zorgplichtonderdelen · Normenkaders · Doorgifte-instrumenten ·              │
│  Gronden FG-positie · Systeemcontrole                                        │
│                                                                              │
│  Systeemcontrole: 2 opmerkingen · onderbrekende meldingen deze week: 1 van 5 │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Het eerste scherm en de weg erheen

Het eerste scherm is het slot, en het slot draagt de identiteit van het *bestand*, niet van de rol: naam, volledig pad, bestandssysteem, de vingerafdruk van de installatiesleutel in blokken van vier, en de datum van het laatste anker. Dat kan vandaag al, want `Kluis::installatiesleutel_lezen` werkt zonder wachtwoord. De compartimentsnamen staan er níet, want die zijn pas na openen beschikbaar; het winnende voorstel beweerde dat zij uit de kluisheader komen en dat klopt niet. Of de organisatienaam vóór ontgrendeling getoond mag worden is een open vraag: er is geen kolom voor in het kluishoofd, en op een gestolen laptop is dat een metagegeven dat u weggeeft. Voorlopig staat hij er niet.

Onder het invoerveld staat letterlijk dat deze zin niet op een andere kluis wordt geprobeerd. Bij een mislukte poging luidt de tekst niet "verkeerd wachtwoord" maar: *"Deze zin opent de organisatiekluis 9f3a 2c81 niet. Uw persoonlijke dossier heeft een eigen zin."* Er is geen terugval, geen "probeer beide" en geen suggestie dat het misschien de andere zin was, want die suggestie is zelf een orakel. De enige keer dat de schil de kruisproef wél doet, is bij het aanmaken van het persoonlijke dossier: de nieuwe zin wordt één keer tegen de organisatiekluis geprobeerd en het aanmaken wordt geweigerd als hij daar past. Die controle bestaat al in `fg nieuw`; de schil toont haar als zichtbare, gelogde stap met tijdstempel, want een beveiligingsaanname die niemand ziet, is een aanname.

De twee kluizen zijn twee vensters, twee sloten, twee wachtwoordzinnen en twee processtatussen — nooit twee tabbladen. Elk venster draagt een permanente identiteitsband over de volle breedte, in een eigen dekkende tint, en het persoonlijke venster zegt erbij dat de organisatie de inhoud niet kan lezen, exporteren of verwijderen. De scheiding is niet cosmetisch: de capabilities zijn in Tauri v2 per vensterlabel te scopen, dus het FG-venster kent de organisatiecommando's simpelweg niet en omgekeerd. Dat tilt de maatregel tegen capture errors van een huisregel naar een lockout. Er is nooit meer dan één wachtwoordveld tegelijk actief; of dat op Wayland met WebKitGTK betrouwbaar af te dwingen is, staat niet vast en is een testpunt, geen belofte.

De twee handelingen die beide kluizen nodig hebben, `fg spiegelen` en `fg aantonen`, verlopen als tweeluik, en de tweede ontgrendeling wordt pas gevraagd nádat het record is gekozen — zodat de vraag "welke zin?" altijd naast het bestandspad staat waar hij voor bedoeld is. Het tussenscherm toont wat er precies oversteekt: de hash, het tijdstip, en expliciet wat er niet meegaat. Een tweede spiegeling van dezelfde inhoud wordt geweigerd met de eerdere spiegeldatum erbij.

Vóór de werkbak kunnen drie schermen tussenkomen, en die zijn blokkerend omdat de brondocumentatie dat zo vastlegt. SYS-04 (ketenbreuk in het auditspoor) blokkeert alle bewerkingen; SYS-08 (een onafgeronde onomkeerbare handeling) blokkeert tot de gebruiker afronden of terugdraaien kiest; SYS-09 (termijnmodule niet geverifieerd na een update) blokkeert alle termijnberekeningen, wat betekent dat de werkbak dan een geblokkeerde toestand moet kunnen tonen in plaats van een lege lijst. Na een uitval komt eerst het herstelrapport met diff. Geen van deze vier schermen stond in het winnende voorstel, terwijl §3.11 ze alle vier noemt.

Daarna volgt de strook "wat er is gebeurd terwijl de tool dicht was": welke absolute momenten zijn gepasseerd en sinds wanneer de applicatie niet open was. De handeling `KluisGesloten` staat al in het logboek, dus dit is goedkoop, en het voorkomt de duurste misvatting over dit product — dat er iets bewaakt zou worden. De sessie vergrendelt in Rust bij schermvergrendeling, slaapstand, gebruikerswissel en na tien minuten inactiviteit; de zin wordt nooit onthouden. Het klembord wordt na dertig seconden gewist. Dat een gesynchroniseerd klembord *geblokkeerd* wordt, is te sterk gezegd: de uitsluitingsformaten van de gangbare cloudklemborden zijn adviserend. De schil vraagt ze aan, detecteert wat zij kan detecteren, en zegt in het scherm wat zij niet garandeert.

---

## Vastleggen

Het werkblad is één doorlopende pagina met vier tot zes genummerde blokken en één vraagstelling per blok. Springen mag altijd; de indeling is een ordening en geen gevangenis. Concept is een geldige toestand, en verplicht is een eigenschap van de overgang naar vastgesteld, niet van het veld — dat is de belangrijkste maatregel tegen situationele violations, want niemand hoeft iets te verzinnen om verder te komen.

Wat opslaan betekent, verdient een expliciet besluit dat in geen van de drie voorstellen werd genomen. Elk veld opslaan bij het verlaten klinkt vriendelijk, maar `Kluis::bewaar` schrijft per aanroep een volledige nieuwe recordversie én een regel in de hashketen. Een intake van veertien onderdelen met correcties levert dan tientallen ketenregels op waar de opdrachtregel er één schrijft, en het logboek dat moet aantonen wát er is besloten, vult zich met toetsaanslagen. Daarom: veldwijzigingen gaan naar een conceptbuffer die de schil bewaart en die een crash overleeft, en er wordt één keten- en versieregel geschreven per bewuste handeling — het verlaten van een blok, of een statusovergang. Dat vergt een conceptlaag in de kluiskern die er nu niet is, en het is de reden dat het herstelrapport uit de vorige paragraaf überhaupt iets te herstellen heeft.

Conditionele velden verschijnen ingesprongen onder het antwoord dat ze oproept, met de mededeling wat er is toegevoegd en wat de teller doet. Progressieve onthulling geldt uitsluitend voor optionele diepte en voor de herleiding achter een berekende waarde; een wettelijk minimum staat nooit achter een uitklapper, en de component weigert dat ook.

```
  4 SYSTEMEN EN ONTVANGERS                        onderdeel 9 t/m 12 van 14

  Ontvangers buiten de EER
    ( ) geen doorgifte
    (•) doorgifte naar een derde land
    ( ) doorgifte naar een internationale organisatie
    ( ) nog niet vastgesteld — wordt een taak met eigenaar en uiterste datum

    ▸ Hierdoor zijn drie onderdelen aan dit dossier toegevoegd.
      De noemer ging van 11 naar 14 onderdelen.              [ Waarom ]

    Land          [ zoek in beheerde lijst ▾ ]
    Instrument    ( ) adequaatheidsbesluit  art. 45
                  ( ) modelbepalingen       art. 46 lid 2 onder c
                  ( ) bindende bedrijfsvoorschriften  art. 47
                  ( ) uitzondering          art. 49 lid 1
                  De uitkomst "geen instrument nodig" bestaat hier niet.
    Motivering    [                                                     ]
                  Art. 30 lid 1 onder e AVG vraagt om de documentatie van
                  de passende waarborgen.

  ─────────────────────────────────────────────────────────────────────────
  BUITEN BEELD                                          0 velden
  Deze strook staat er altijd, ook op nul, zodat "er viel niets weg" en
  "ik heb de melding niet gezien" niet op elkaar lijken.
```

Zodra de gebruiker terugkeert naar "geen doorgifte", gebeuren er drie dingen tegelijk en zijn ze alle drie zichtbaar: de velden verdwijnen uit beeld maar hun waarden blijven bewaard, ze tellen niet mee in volledigheid, export of controle zolang ze verborgen zijn, en de noemer daalt met een eigen regel op het scherm. Die derde melding is er omdat een stijgende noemer anders als vooruitgang wordt gelezen en een dalende als een cadeau. Wie overal "nee" antwoordt, gaat van elf van zeventien naar elf van elf zonder één feit vast te leggen; dat is de gevaarlijkste eigenschap van een tellergebaseerd ontwerp en zij ontstaat juist doordat we scores hebben afgeschaft.

De noemer is daarom geen weergaveding maar een gegeven met een geschiedenis. Hij staat permanent in de dossierkop met een uitklap die de vervallen onderdelen en hun bewaarde waarden benoemt, hij noemt de oorzaak en het tijdstip, en hij reist mee in elke export — in de registerweergave, in het bestuursstuk en in het toezichthoudersdossier. Dat is werk in het domein en niet in de schil: `Volledigheidsrapport` is vandaag staatloos en herberekent bij elke aanroep. Wat er wél al is, is een benoemde lijst: `ontbrekende_onderdelen()` levert `Ontbrekend` met veld, omschrijving, grondslag en of het onderdeel de vaststelling blokkeert. Daar is de teller-als-query op gebouwd: elk getal op een statuspaneel opent de onderliggende regels, voorgefilterd, met de grondslag als koptekst. Er bestaat geen getal zonder route naar de rijen.

Een afgerond blok dat een wettelijke vervolgstap oproept, maakt die vervolgstap ter plekke aan als taak met eigenaar en termijn, zichtbaar op het moment dat de afleiding ontstaat:

```
  ☑ Bijzondere persoonsgegevens: gezondheidsgegevens
    ▸ Twee dingen toegevoegd aan dit dossier:
      • Uitzonderingsgrond art. 9 lid 2 — verplicht vóór vaststellen [ Invullen ]
      • DPIA-criterium 4 "gevoelige gegevens" aangezet
        (1 van 9; twee of meer wijst op een waarschijnlijk hoog risico —
        dat is een aanwijzing, geen rekensom)                  [ Toets openen ]
    ▸ Raakt ook: effectbeoordeling DPIA-0009 gaat op "herziening nodig"
      (art. 35 lid 11).                                        [ Ernaast lezen ]
```

Die laatste regel — "Raakt ook", met dossiernummers, vóór uitvoering — is uit het dossier-eerst-voorstel overgenomen, en in het geraakte dossier landt een nawerkingsstrook die niet verdwijnt door lezen maar alleen door bevestigen of afhandelen. Het patroon bestaat al half in de CLI, waar `register vul` de criteria vóór en na vergelijkt en meldt dat de DPIA op herziening gaat; wat ontbreekt is de vooruitblik vóór de schrijfhandeling.

Waar de vaststelling wordt geweigerd, verschijnt de weigering bij het veld of bij de knop, in dezelfde vorm door de hele applicatie, en met altijd twee uitwegen:

```
    Bewaartermijn    [ getal ] [ eenheid ▾ ] vanaf [ startgebeurtenis ▾ ]
                     grondslag [ ▾ ]
                     ─────────────────────────────────────────────────────
                     Vaststellen is hier geblokkeerd. De bewaartermijn
                     ontbreekt. Artikel 30 lid 1 onder f AVG vraagt om de
                     beoogde termijnen waarbinnen de categorieën gegevens
                     worden gewist.  BEW-01 · blokkerend · functionaris
                     [ Termijn invullen ]  [ Nog te bepalen vastleggen ]
                     De tweede knop legt reden, uiterste datum en eigenaar
                     vast en wordt een taak in de werkbak. Deze weigering
                     staat in het logboek en telt als gevangen fout.
```

Verplichte motiveringen staan overal waar een open norm wordt ingevuld en zijn niet leeg te laten: bij de risicoweging, bij het gemotiveerd accepteren van een signaal, bij een verificatieperiode tussen signaal en kennisname, bij een afwijking van een bibliotheeknorm en bij elke "nog te bepalen". Een acceptatie verdwijnt niet uit het zicht; zij blijft telbaar in de dossierkop met eigenaar en herbeoordelingsdatum, en het aandeel geaccepteerde signalen per regel is een contrastmeting in de maandrapportage. Stijgt dat aandeel structureel, dan gaat de regel op de schop en niet de gebruiker.

Twee soorten waarden krijgen een markering aan de linkerrand van het veld, niet grijs maar met een eigen kenmerk: "overgenomen uit de bibliotheek, niet getoetst" en "geërfd, niet geverifieerd". Grijs leest als uitgeschakeld. Zo'n waarde blokkeert de vaststelling tot iemand haar bevestigt, is niet handmatig te wissen en reist mee in elke export. Datzelfde geldt voor bewijs: er staan twee tellers naast elkaar die nooit worden opgeteld — bewijs aanwezig, en bewijs door een mens beoordeeld met naam en datum — met bij elk stuk het label dat de tool de inhoud niet heeft gelezen en met het geldigheidsvenster erbij, want een verlopen bewijsstuk laat het onderdeel terugzakken naar onbewezen.

Ten slotte de doorlooptijd, die in geen van de drie voorstellen voorkwam terwijl acceptatiecriterium 12 haar per nieuw veld eist. De normen uit §6.5 zijn: een registerregel bijwerken onder de twee minuten, een datalek registreren onder de drie, een verzoek intaken onder de minuut. Die worden vanaf dag één gemeten, en overschrijding is een defect van het scherm, geen gedragsprobleem van de gebruiker.

---

## De onomkeerbare handelingen

Onomkeerbaarheid krijgt één visuele klasse die nergens anders voorkomt: volle breedte, dubbele omlijning, altijd alleen op de eigen regel, nooit naast een goedaardige knop en nooit in dezelfde stijl. De klasse is toegewezen aan een gesloten lijst en de generator weigert haar elders. Het criterium voor die lijst is één vraag: verlaat het de organisatie, of is het binnen de tool niet terug te draaien. Daarmee valt `logboek anker --uitvoer` er wél onder, want dat schrijft een ondertekend anker naar buiten, en `zorgplicht bewijs` valt eruit, want dat trekt een bestand juist naar binnen en heeft een eigen intrekhandeling.

Bevestigen gebeurt nooit met een dialoog en nooit met één vinkje voor alles. Het gebeurt met een time-out waarin elk item afzonderlijk wordt bevestigd met de waarde ernaast — maar de vorm verschilt per handeling, en dat is het punt waar het winnende voorstel alles platsloeg tot "zes vinkjes":

| Handeling | Vorm |
|---|---|
| Melding aan de AP | zes items, tweede persoon of afkoelperiode, uitgesteld verzendvenster van vijf minuten met annuleerknop in de werkbak, en een idempotentiesleutel tegen dubbele indiening |
| Vroegtijdige waarschuwing meldketen (24 uur) | drie items, géén uitstel — snelheid gaat hier voor en het bericht is bewust beperkt van omvang |
| Incidentmelding en eindrapport meldketen (72 uur, één maand) | volledige time-out plus tweede persoon; het 72-uursformulier wordt pas aangeboden nadat het eerste bericht is vastgelegd |
| Bericht aan betrokkenen (art. 34) | tweede persoon, ontvangerslijstcontrole met aantal, en de verzendmethode-lockout: bij meer dan één ontvanger bestaat één bericht met meerdere zichtbare geadresseerden niet |
| Antwoord op een betrokkenenverzoek | time-out plus bijlagecontrole: elke bijlage apart bevestigd met naam, aantal pagina's en herkomstdossier; bijlagen uit een ander dossier zijn niet selecteerbaar |
| Besluit "niet melden" | de zwaarste barrière: complementaire tweede beoordeling of afkoelperiode, met een herzieningsroute (zie hieronder) |
| Export van een volledig dossier | tweede persoon, dubbele invoer van de bestemming, classificatie-lockout waarbij bijzondere gegevens naar een onbeveiligde bestemming geblokkeerd zijn en niet gewaarschuwd, exportstempel en auditspoor |
| Sleutelrotatie of wachtwoordwissel | herstelcode met dubbele invoer, bevestigde back-up vóór uitvoering, tweefasige uitvoering |
| Intrekken van een registerregel | uitdrukkelijk niet onomkeerbaar: geen dialoog, wel dertig seconden inline ongedaan maken |

```
TIME-OUT — Vroegtijdige waarschuwing, meldketen zorgplicht
Dit bericht verlaat de organisatie. Er is geen uitstelvenster: de termijn
verstrijkt vanavond om 21:00 CEST.

☐ Entiteit     Gemeente Noorderveld
☐ Aard         onbevoegde verstrekking, verzuimportaal, significant sinds
               di 18-08 21:00 CEST
☐ Termijn      uiterlijk wo 19-08-2026 21:00 CEST — nog 11 uur 46 minuten

                                     [ Terug ]   [ Klaarzetten ]
```

Hier ligt een eerlijkheidsprobleem dat geen van de drie voorstellen benoemde: **de tool verstuurt niets.** Het programma opent uit zichzelf nooit een netwerkverbinding, en het auditspoor onderscheidt niet voor niets `MeldingKlaargezet` van `MeldingVerzonden`. Wat de knop doet is de tekst klaarzetten en het klaarzetmoment vastleggen; de mens plakt hem daarna in het portaal van de toezichthouder. Het annuleervenster van vijf minuten hoort dus niet vóór het klaarzetten — daar valt niets te annuleren — maar bij het moment waarop de gebruiker verklaart dat hij heeft verzonden. In die vijf minuten kan hij die verklaring intrekken, bijvoorbeeld omdat het portaal de indiening weigerde. En de status wordt pas "Gemeld" na invoer van het referentienummer en de tijdstempel van de ontvangstbevestiging, niet eerder. Of dit de juiste plaats voor het venster is, staat niet vast; het is de beste vertaling die ik zie van een eis die is geschreven voor een tool die zelf indient.

Daar hangt een tweede gat aan. Er bestaat vandaag geen handeling die een verplichting afdoet: `dpofg incident` kent geen `melden`, `gemeld_op` wordt nergens gezet, en `ZORG_EINDRAPPORT` is verankerd op precies dat veld. Zonder die handeling kan de bovenste regel van de werkbak niet worden afgevinkt en kan het eindrapportanker nooit ontstaan. Dat is domeinwerk dat vóór het incidentwerkblad af moet zijn.

De complementaire tweede beoordeling krijgt een eigen scherm, want zonder dat scherm is de vierde laag geen tegenspraak maar bevestigingsdruk. De tweede beoordelaar ziet de onderliggende feiten en de vraag, niet het ingevulde formulier, en zijn oordeel wordt vastgelegd vóórdat hij het andere ziet. Bij verschil zien beiden beide motiveringen en wordt het besluit gezamenlijk vastgelegd. Bij een eenpersoonsfunctionaris vervangt de afkoelperiode die stap: het besluit wordt vastgelegd, een tweede sessie volgt na minimaal dertig minuten of de volgende ochtend waar de termijn dat toelaat, en de eerdere motivering wordt pas ná de herbevestiging zichtbaar. Dat besluit draagt permanent het kenmerk "afkoelperiode, geen tweede persoon", dat kenmerk reist mee in de bundel, en het omkeerpercentage staat in de maandrapportage — nul procent over lange tijd is zelf een bevinding, want het bewijst dat de barrière niet werkt.

"Niet melden" is bovendien geen doodlopende weg. Er komen feiten bij en dan meldt u alsnog, te laat en gemotiveerd; dat is een normale gang van zaken die de motor ook kent via het omkeren van het besluit. Het scherm draagt daarom een herzieningsroute die vastlegt welke feiten het eerdere besluit niet kende. Zonder die route bouwt het ontwerp een barrière tegen het corrigeren van een fout, en dat is het omgekeerde van de bedoeling.

---

## Wat er per rol anders is

De motor kent zeven ontvangerrollen: functionaris, behandelaar, contracteigenaar, systeemeigenaar, security officer, directie en beheerder. Elke controleregel heeft er één, en `dpofg controle --voor <rol>` filtert daar vandaag al op. Die filter mag in de schil niet verdwijnen, want §3.9 regel 4 is een van de vijf antimoeheidsregels: een bewaartermijnsignaal gaat naar de proceseigenaar, een technisch signaal naar de security officer, en de functionaris krijgt de uitzonderingen en de escalaties, niet de ruis.

De werkbak toont daarom standaard wat aan uw eigen rol hangt, met één klik naar alles. De filter raakt de sortering niet — die blijft vast en niet om te draaien — en hij verbergt niets stilzwijgend: de voetregel telt wat er buiten uw filter valt, net zoals hij telt wat er buiten de lijst valt. Eigenaarschap hangt aan de rol en de persoon is de bezetting; de regel toont dus "eigenaar: security officer (M. de Wit)" en niet andersom. "Geen eigenaar" bestaat niet als rusttoestand: een verplichting zonder rol is een blokkerende taak "eigenaar toewijzen" die zelf een eigenaar heeft, namelijk de functionaris, en een termijn. De motor is hier al strenger dan het winnende voorstel: `crates/dpofg-domain/src/correctie.rs` weigert een correctie waarbij rol óf bezetting ontbreekt, met als motivering dat een correctie zonder eigenaar een voornemen is dat vanzelf verdwijnt.

Hier moet een grens scherp getrokken worden. Er is geen rollen- en aanstellingsregister, geen authenticatie per rol en geen gelijktijdigheid: de actor komt uit de omgevingsvariabele van de gebruiker, met in de code de opmerking dat er nog geen rollenmodel is. De rolfilter is dus een weergave en geen toegangsbeperking, en de schil moet dat zeggen in plaats van het te suggereren. Zolang dat register er niet is, betekent "eigenaar: security officer" dat iemand die rol heeft opgeschreven, en niets meer.

Datzelfde geldt voor escalatie. §3.5 eist escalatie naar een ándere persoon in plaats van herhaling naar dezelfde, en dat is verstandig, maar dit product is lokaal, eenpersoons en netwerkloos: er is geen kanaal. De schil lost dat op de enige eerlijke manier op — zij maakt een overdrachtsstuk met de feiten, de klok, het anker en de vraag, klaar om te versturen buiten de tool, en zij legt de overdracht vast met read-back: de ontvanger reproduceert de kritieke waarden actief uit keuzelijsten in plaats van een samenvatting af te vinken, en afwijkingen tussen wat de zender vastlegde en wat de ontvanger reproduceert worden aan beiden getoond. De escalatieknop is dus geen verzendknop maar een opdracht aan uzelf, met een vastlegging.

Het persoonlijke FG-venster is de enige plek waar `advies`, `reactie`, `escaleren`, `onafhankelijkheid` en `toon` bestaan. In het organisatievenster bestaan die velden niet — niet uitgeschakeld, maar afwezig — zodat een advies daar niet per ongeluk kan belanden. Omgekeerd kent het FG-venster geen registerscherm. De enige brug is het spiegelrecord, en die brug draagt alleen een hash.

---

## Bouwvolgorde

Elke stap moet iets opleveren dat de opdrachtregel nog niet kan. Op één na, en die uitzondering staat vooraan omdat zij anders halverwege alsnog moet worden gedaan, dan met zestien schermen eroverheen.

**Stap 0 — een servicecrate uit de bedieningsschil snijden, met een handelingsmanifest.** De ruim twaalfduizend regels in `crates/dpofg-cli/src/opdrachten/` zijn geen API maar clap-code met `println!`, wachtwoordinvoer en foutafhandeling door elkaar; zoeken op kenmerk, bewaren met handeling en omschrijving, en het loggen van een geblokkeerde controle zitten dáár en niet in het domein. De schil kan die functies niet aanroepen. Deze stap levert de gebruiker niets en dat moet ook zo gezegd worden; wat zij oplevert is dat elke handeling straks één keer bestaat in plaats van twee, en dat het manifest de bouwlint kan voeden waarmee een knop zonder opdracht de bouw laat falen.

**Stap 1 — `dpofg werkbak --json` in de kern.** Een uniform verplichtingstype over de dossiersoorten heen, met anker, ankertype, grondslag, rekenregel, eigenaarsrol en een voldaan-toestand, plus de ontbrekende afdoenhandelingen — te beginnen bij het incident, waar melden vandaag niet bestaat. Golden tests per dossiersoort, want elk regime heeft eigen ankers, opschorting, verlenging en intrekking. Dit is het grootste stuk nieuw werk in het hele plan en het zit in `dpofg-domain`, niet in de schil. Het levert onmiddellijk iets op dat er nu niet is: één lijst op de opdrachtregel, en daarmee cron, mail, een agenda-export en een afdruk voor de dagafsluiting — precies het gat dat een gesloten grafische schil openlaat.

**Stap 2 — het slot, de dertien kluisloze schermen en de systeemcontrole.** Meteen bruikbaar zonder dat er een kluis bestaat, het bewijst dat de brug tussen schil en Rust klopt vóór er één sleutel in het spel is, en het legt de tweevensterscheiding met capabilities vast vóórdat er een tweede kluis is. Hier landen ook de blokkadeschermen en het herstelrapport.

**Stap 3 — de werkbak zelf.** Vijf banden plus verstreken plus de strook in uitvoering, de voetregel, de rolfilter, het zoekfilter, meervoudige sporen per dossier, de statuspanelen per soort met klikbare tellers, en de klok in Rust. Dit is het eerste scherm dat iets doet wat de opdrachtregel niet kan: bijblijven terwijl u iets anders doet.

**Stap 4 — één werkblad, en dat is het incident.** Daar zitten de vier tot vijf sporen, de verificatieperiode tussen signaal en kennisname, de verplichte weging met motivering, en het besluit "niet melden" met tweede persoon of afkoelperiode. Daar hoort de meldtekstopbouw bij als kernmodule in `dpofg-report`: per kanaal de velden in de volgorde van het formulier, met een kopieerknop per veld en een teller die zegt wat er voor dít formulier nog ontbreekt. Dat is waarschijnlijk het grootste werkelijke tijdvoordeel op een lekdag, en het bestaat vandaag nergens.

**Stap 5 — de gedeelde componenten, vóór het tweede werkblad.** De weigering bij het veld, de dossierkop met teller en noemergeschiedenis, de buiten-beeldstrook, de time-out in zijn drie vormen, de complementaire beoordeling en de herkomstmarkering. Ze nu vastleggen is het verschil tussen zestien consistente schermen en zestien varianten.

**Stap 6 — uitleveren.** `dossier <map>` en `prognose --export` met de vijf waarborgen, de niet over te slaan inhoudslijst en de weglatingen in het manifest.

**Stap 7 — de nulmeting, vanaf dag één meegebouwd.** Doorlooptijd per taak, gebruik van inline ongedaan maken binnen dertig seconden, dossierverwisseling binnen vijf minuten, onderbrekende meldingen per week, aandeel dummywaarden per veld, en de ladderverdeling uit §6.5. Zonder nulmeting is elke latere uitspraak dat dit ontwerp werkt ongefundeerd.

Bewust nog niet: de overige vijftien werkbladen, het prognosescherm, het ketenvenster en het persoonlijke FG-venster als volwaardige werkomgeving. Voor die handelingen staat het dossier wél in de werkbak, met klok en grondslag, en eronder de exacte aanroep met een kopieerknop. Dat is eerlijker dan zestien halve schermen. Er zit alleen een addertje onder dat niemand tot nu toe benoemde: er ligt een exclusief slot op `kluis.lock` plus single-instance, dus zolang de schil open is, kan `dpofg` de kluis niet openen. De delegatieroute is dus vandaag: schil sluiten, sleutel opnieuw afleiden, opdracht draaien, schil openen. Dat is onaanvaardbaar als dagelijkse route en het is een van de open punten hieronder.

Over de omvang: dit laat zich niet betrouwbaar in weken uitdrukken. Er is nul frontend — geen `package.json`, geen `tauri.conf.json`, geen raamwerkkeuze in enig document — en `Cargo.lock` bevat vandaag geen `tauri` en geen `tokio`. Het plan begroot voor "schil, allowlist op de brug, inhoudsbeleid, ontgrendelscherm" anderhalve week en voor de werkbakmodule anderhalve week, en legt op elke ontwikkelschatting een opslag van veertig procent. Stap 1 alleen al is groter dan die twee samen. Bij één ontwikkelaar en dertig productieve uren per week is een kwartaal een realistischer orde van grootte dan zes tot acht weken, en ook dat is een schatting zonder stackkeuze eronder — wat wil zeggen: een schatting die pas na de stackbeslissing iets waard is.

---

## Wat dit ontwerp kan misleiden

**Een lege werkbak leest als "ik ben bij", terwijl hij alleen zegt dat er geen klok loopt.** Werk zonder termijn zakt naar de onderste band en dan onder de vouw. Daartegen staat de voetregel, permanent, niet in te klappen, met een telling en een route. Er bestaat geen weergave van de werkbak zonder die regel. Wat de voetregel níet oplost is dat de gebruiker hem leest en niets doet; die faalmodus wordt gevangen door de volledigheidsmaten in de maandrapportage.

**De sortering op onherstelbaarheid wordt gelezen als sortering op belang.** Wie alleen de bovenkant afwerkt, verliest structureel de eisen met lange doorlooptijd: pentests, contractvernieuwing, opleiding, bestuursvaststelling. De vervalprognose van dertig dagen vangt een deel daarvan, en waar iemand een doorlooptijd heeft opgegeven promoveert de regel eerder, met de rekensom erbij. Waar niemand een doorlooptijd heeft opgegeven, blijft het risico bestaan, en dat is dan een gat in de gegevens, geen gat in de sortering — dat verschil moet zichtbaar blijven in plaats van weggemasseerd met een standaardwaarde.

**Eén werkvoorraad suggereert één klok per dossier.** Daarom levert één dossier net zoveel regels als het lopende klokken heeft, elk met `spoor n van m`, een eigen grondslag en een eigen ankertype — en het ankertype wordt bij naam genoemd, niet samengetrokken tot "kennisname", want het domein onderscheidt kennisname, ontvangst van een verwerkersmelding, vaststelling van een hoog risico, vaststelling van significantie en verzending van de melding. Wie die woorden gelijkschakelt in de weergave, herintroduceert precies de fout die de motor afvangt.

**Een verzorgde schil verleent gezag aan een voorlopige motor.** Een gekozen lezing van een omstreden termijnregel, een verouderd kennispakket en een geërfde registerregel zien er in nette typografie even stellig uit als een adequaatheidsbesluit. Daartegen: de consolidatiedatum en het aantal te verifiëren punten staan in de kop van de werkbak en niet in een "Over"-scherm; een afgeleide termijn die uit een keuze tussen twee lezingen volgt draagt die keuze met datum in de regel én in de export; en de schil citeert nooit een artikelnummer dat het kennispakket niet levert. Waar het pakket "meldketen zorgplicht, eerste bericht" zegt, staat dat er — met het voorbehoud erbij. Dat is lelijker en het is het enige eerlijke.

**Focusmodus verbergt de bak, en de bak is de plek waar de klok afloopt.** De focusmodus onderdrukt en buffert alles behalve één ding: een klok die tijdens de sessie in de bovenste band belandt. Die verschijnt in een eigen strook onder de identiteitsband en niet ín die band, want de band is juist de maatregel tegen het verwarren van twee gelijkende dossiers en een mededeling over dossier B ondermijnt haar functie. Die onderbreking telt mee in het waarschuwingsbudget van vijf per week; komt zij daar structureel bovenuit, dan is dat een defect van het ontwerp.

**Het compartimentsverhaal kan een scheiding suggereren die er niet is.** Alle compartimentsleutels hangen onder één kluissleutel onder één wachtwoordzin. De kop zegt daarom wat waar is: berekend over alle compartimenten van deze kluis. Zodra een compartiment een eigen wachtwoord krijgt, verandert die regel in de vorm die het winnende voorstel al beschreef, en dan pas — niet eerder.

**Nieuw in dit ontwerp, en dus nieuw risico.** De rolfilter suggereert routering die niet is afgedwongen; daarom staat er bij dat het een weergave is en telt de voetregel wat de filter wegneemt. Het leespaneel verzwakt de focusdiscipline die het winnende voorstel juist streng maakte; daarom is het alleen-lezen en breekt bewerken wél zichtbaar. De meldtekstmodule kan de indruk wekken dat de tool indient; daarom heet de knop "klaarzetten" en verandert de status pas na een referentienummer. En de ladderverantwoording zelf kan een ritueel worden dat achteraf wordt ingevuld; het enige tegengif dat ik zie is dat de ladderverdeling een meetwaarde in de maandrapportage is en niet alleen een tabel in dit document.

---

## Wat nog niet vaststaat

**De stackkeuze.** Er is geen frontendraamwerk gekozen, geen toestandsmodel en geen bouwstraat voor de schil. Zonder die keuze zijn alle schattingen in dit stuk indicaties. Nodig: de stackbeslissing die het plan aan het einde van week drie voorziet, plus een proef met de Playwright- en `tauri-driver`-opstelling op WebKitGTK en op de Windows-webview, met aparte referentiebeelden per motor.

**De delegatieroute naast het exclusieve kluisslot.** Zolang de schil open is, kan de opdrachtregel de kluis niet openen, terwijl de eerste versie de gebruiker juist naar de opdrachtregel stuurt. Er zijn drie richtingen: de schil geeft het slot tijdelijk vrij, de schil voert de gedelegeerde handeling zelf uit via dezelfde servicefunctie met de opdrachtregel als weergave, of de opdrachtregel praat via een lokale socket met de open sessie. De derde is het prettigst en het meest risicovol. Nodig: een besluit, want de eerste versie hangt eraan.

**Rollen en aanstellingen.** Zonder register is "eigenaar" een tekst en is de rolfilter een weergave. Nodig: de recordsoort waarop de controleregels toch al wachten, met de vraag of één actieve houder per kluis het uitgangspunt blijft.

**Compartiment met een eigen wachtwoordzin.** Zolang die er niet is, kan het onzichtbaarheidsverhaal niet worden waargemaakt en moet de schil zwijgen over sleutels die zij niet mist.

**De doorlooptijd per maatregel.** Wie levert dat getal, en waar wordt het opgeslagen? Zonder herkomst is de promotiedatum verzonnen precisie. Nodig: een veld met eigenaar en datum, of het definitieve besluit dat de vervalprognose bij dertig dagen blijft.

**De klokafwijkingscontrole.** De monotone teller staat in de documentatie beschreven als de tijd sinds de start van het proces; op Linux loopt die tijdens slaapstand niet door, dus een laptop die acht uur slaapt levert een verschil dat ver boven de drempel van vijf minuten ligt en dat een niet te onderdrukken auditvermelding oplevert. Nodig: een ontwerp dat slaapstand onderscheidt van een verzette klok — waarschijnlijk door bij het sluiten een ondertekend wandkloktijdstempel weg te schrijven en dat bij het openen te vergelijken — en de vaststelling dat dit mechanisme vandaag nog nergens in de code bestaat.

**De conceptbuffer en de korrel van het ketenlogboek.** Eén ketenregel per bewuste handeling in plaats van per veld vergt een conceptlaag in de kluiskern die er niet is, en `Kluis::bewaar` is vandaag strikt eenfasig. Datzelfde geldt voor het tweefasige journaal dat SYS-08 veronderstelt. Nodig: een besluit over waar concepten leven en hoe zij versleuteld worden.

**De semantiek van het annuleervenster.** Als de tool niet indient, waar hoort het venster van vijf minuten dan? Dit ontwerp zet het bij de verklaring "verzonden". Dat is een interpretatie van een eis die voor een indienende tool is geschreven, en zij hoort tegen de brondocumentatie te worden gelegd voordat zij gebouwd wordt.

**Het klembord.** Wissen na dertig seconden vernietigt ook wat de gebruiker intussen zelf heeft gekopieerd, en het blokkeren van een gesynchroniseerd klembord is met de beschikbare mechanismen adviserend. Nodig: een proef per platform en een eerlijke formulering in het scherm.

**Schaal.** Er staat geen enkele schaaltest in de testmap. De werkbak leest per dossiersoort records één voor één als losse envelop; bij vijfduizend registerregels is onbekend hoe dat zich houdt, en de "al gezien"-verzameling voor de focusmodus moet afsluiten, slaapstand en herstart overleven. Nodig: een meting vóór stap 3, niet erna.

**Bulkwerk.** Vijf geërfde regels bevestigen of drie bewaartermijnen invullen is op de opdrachtregel een lus van drie regels shell en in dit ontwerp drie keer openen, invullen en verlaten. Bij een overname of een jaarlijkse herziening is dat het verschil tussen een middag en een week. Of er een bulkhandeling komt, en hoe die zich verhoudt tot de weigering bij het veld en tot de motiveringsplicht, is niet beslist.

**Ten slotte de telling zelf.** Dit document noemt twaalf kluisloze weergaven, achttien te verifiëren punten in het voorbehoud, achtenzeventig controleregels waarvan er veertien geen evaluatiefunctie hebben, tweeëntwintig opdrachtgroepen en zeven ontvangerrollen. Die getallen zijn vandaag nagerekend tegen de broncode, maar ze horen niet in een ontwerpdocument thuis te blijven staan: ze horen uit het manifest te komen, zodat een verschoven telling een bouwfout is en geen leesfout. Dat is dezelfde discipline die dit ontwerp van zijn gebruikers vraagt.
