# Beveiligingsbeleid

`dpo-fg-tool` beheert het gevoeligste materiaal van een organisatie: het verwerkingsregister, datalekdossiers, risicobeoordelingen en incidentmeldingen. Beveiliging is daarom geen bijzaak maar het uitgangspunt van het ontwerp.

## Kwetsbaarheid melden

Meld een vermoedelijke kwetsbaarheid **niet** via een openbare issue of pull request.

Gebruik in plaats daarvan het beveiligde meldkanaal van deze repository: **Security → Report a vulnerability**. Is dat kanaal in jouw situatie niet beschikbaar, neem dan rechtstreeks contact op met de beheerder van de repository (`WimLee115`) en vraag om een vertrouwelijk kanaal voordat je details deelt.

Vermeld in de melding:

1. Een beschrijving van de kwetsbaarheid en de vermoedelijke impact.
2. De versie, het besturingssysteem en de build waarop je het hebt vastgesteld.
3. Reproduceerbare stappen of een minimaal voorbeeld.
4. Eventuele voorgestelde mitigatie.

Neem in de melding **geen** echte persoonsgegevens of productiegegevens op.

## Wat je mag verwachten

| Stap | Termijn |
|---|---|
| Ontvangstbevestiging | binnen 5 werkdagen |
| Eerste inhoudelijke beoordeling | binnen 15 werkdagen |
| Statusupdate | ten minste elke 30 dagen zolang de melding openstaat |
| Openbaarmaking | in overleg, na beschikbaarheid van een oplossing |

Meldingen worden vertrouwelijk behandeld. Wie een kwetsbaarheid verantwoord meldt, wordt op verzoek vermeld in de release-aantekeningen.

## Verantwoord onderzoek

Onderzoek is welkom binnen de volgende grenzen:

- Test uitsluitend op je eigen installatie en met eigen testgegevens.
- Geen aanvallen op de beschikbaarheid, geen spam, geen social engineering.
- Geen toegang tot, wijziging van of verwijdering van gegevens van derden.
- Ga niet verder dan nodig om het probleem aan te tonen.

## Ondersteunde versies

Tijdens de ontwikkelfase wordt uitsluitend de laatste release ondersteund. Na de eerste stabiele release wordt dit overzicht bijgewerkt.

## Ontwerpmaatregelen

De volgende maatregelen zijn onderdeel van het productontwerp en worden bij elke release gecontroleerd:

- Versleutelde opslag in rust met een uit een wachtwoord afgeleide sleutel.
- Manipulatiebestendig auditspoor over alle wijzigingen.
- Geen telemetrie en geen uitgaand netwerkverkeer zonder expliciete toestemming van de gebruiker.
- Vastgezette afhankelijkheden, een softwarestuklijst (SBOM) per release en ondertekende releases.
- Geautomatiseerde controle op bekende kwetsbaarheden in afhankelijkheden.

## De ondertekenidentiteit van een installatie

Elk kluisbestand draagt sinds de uitgave met vast sleutelbeheer één Ed25519-sleutelpaar. Daarmee worden alle ankers en dossiermanifesten van die kluis ondertekend; de publieke helft is met `dpofg kluis sleutel` op te vragen en hoort langs een ánder kanaal dan het dossier te worden gepubliceerd.

Wat dat betekent voor het dreigingsmodel:

- **De privésleutel bestaat blijvend.** Voordien werd per stuk een sleutelpaar gemaakt en meteen weggegooid; er viel niets te stelen. Nu wel. Het zaad staat uitsluitend versleuteld in het kluisbestand, gewikkeld onder de kluissleutel, en komt nooit onversleuteld op schijf.
- **Kluisbestand plus wachtwoordzin is voldoende om te ondertekenen.** Wie beide heeft, kan ankers en dossiers produceren die niet van die van de organisatie te onderscheiden zijn. Behandel een reservekopie van de kluis daarom als sleutelmateriaal, en niet alleen als gegevens.
- **Er is geen intrekking.** Raakt een sleutel weg, dan is de enige weg: een nieuwe kluis, de nieuwe sleutel publiceren, en de oude sleutel ernaast blijven vermelden met de datum vanaf wanneer hij niet meer geldt. Reeds uitgeleverde dossiers blijven met de oude sleutel controleerbaar; dat is onvermijdelijk, want de handtekening eronder verandert niet.
- **Een kopie van een kluis draagt dezelfde identiteit.** Cryptografisch zijn origineel en kopie niet te onderscheiden. Wel zichtbaar: twee kopieën die beide doorwerken leveren ankers op met hetzelfde volgnummer, een verschillende hash en dezelfde ondertekenaar — een vork die aan de installatie is toe te schrijven.

## Grenzen van het dreigingsmodel

Buiten scope vallen aanvallen waarbij het onderliggende besturingssysteem al volledig is gecompromitteerd met de rechten van de gebruiker op het moment dat de kluis is ontgrendeld, alsook fysieke aanvallen op ontgrendelde apparatuur. Deze zijn beschreven in het dreigingsmodel in de projectdocumentatie.
