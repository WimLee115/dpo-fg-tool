# Formaatspecificatie

Deze specificatie beschrijft de bestandsformaten van `dpo-fg-tool` zo, dat een dossier te controleren is **zonder** software van de organisatie die het aanlevert.

Dat is de bestaansreden van dit document. Een bewering dat een dossier niet is gewijzigd, is waardeloos als zij alleen te controleren is met een programma van degene die de bewering doet. Wie deze specificatie leest, kan de controle zelf naprogrammeren.

---

## 1. Het ketenlogboek

### 1.1 Een logboekregel

```json
{
  "volgnummer": 42,
  "gebeurtenis": {
    "handeling": "record_vastgesteld",
    "actor": { "id": "u1", "naam": "A. de Vries", "rol": "fg" },
    "tijdstip": "2026-08-18T09:14:00Z",
    "onderwerp_soort": "verwerking",
    "onderwerp_id": "0198f2c1-...",
    "compartiment": "algemeen",
    "omschrijving": "registerregel vastgesteld",
    "motivering": null,
    "inhoud_voor": null,
    "inhoud_na": null
  },
  "vorige_hash": "…64 hex…",
  "hash": "…64 hex…"
}
```

Volgnummers beginnen bij 1 en lopen zonder gaten op.

### 1.2 De hashberekening

De hash van een regel is:

```
BLAKE3_derive_key(
    context = "dpo-fg-tool audit-keten v1",
    materiaal = volgnummer_be64
             ‖ len_be32(vorige_hash) ‖ vorige_hash_utf8
             ‖ len_be64(gebeurtenis_json) ‖ gebeurtenis_json
)
```

waarbij:

| Onderdeel | Betekenis |
|---|---|
| `volgnummer_be64` | het volgnummer als 8 bytes, meest significante byte eerst |
| `len_be32(x)` | de lengte van `x` in bytes als 4 bytes, meest significante byte eerst |
| `gebeurtenis_json` | het `gebeurtenis`-object, geserialiseerd in de veldvolgorde zoals hierboven |
| `vorige_hash` van de eerste regel | 64 nullen: `0000…0000` |

De hash wordt hexadecimaal in kleine letters weergegeven.

**Waarom de lengteprefixen.** Zonder die prefixen zouden de waardenparen (`ab`, `c`) en (`a`, `bc`) dezelfde bytereeks opleveren, en daarmee dezelfde hash. Het volgnummer zit in de berekening zodat een regel niet naar een andere plaats in de keten kan worden verplaatst.

### 1.3 Wat een geslaagde controle aantoont

| Bewering | Onderbouwd door |
|---|---|
| deze regels staan in deze volgorde | de hashketen |
| regel N is niet gewijzigd | de hashketen |
| regel N is niet verwijderd | volgnummers plus de hashketen |
| er is aan het eind niets weggehaald | **alleen** een anker |
| regel N is op tijdstip T geschreven | **niets in het logboek**; alleen een extern bewaard anker begrenst het moment |

Die laatste twee rijen zijn de reden dat elk verificatierapport zijn reikwijdte meedraagt.

---

## 2. Het anker

```json
{
  "kluis_id": "kluis-1",
  "volgnummer": 42,
  "hash": "…64 hex…",
  "tijdstip": "2026-08-18T12:00:00Z",
  "sleutel": "…64 hex…",
  "handtekening": "…128 hex…",
  "bewaarplaats": "notulen directieoverleg 2026-08-18"
}
```

De handtekening is Ed25519 over:

```
"dpo-fg-tool ketenanker v1"
  ‖ len_be32(kluis_id) ‖ kluis_id_utf8
  ‖ volgnummer_be64
  ‖ len_be32(hash) ‖ hash_utf8
  ‖ unix_seconden_be64
  ‖ nanoseconden_be32
```

`sleutel` is de publieke sleutel (32 bytes, hex), `handtekening` de handtekening (64 bytes, hex).

**Wat een anker wel en niet aantoont.** Het toont aan dat de houder van de bijbehorende privésleutel heeft verklaard dat de keten op dat punt stond. Of die sleutel toebehoort aan wie u verwacht, volgt hier niet uit — dat volgt uit de plaats waar het anker buiten het systeem is bewaard.

---

## 3. Het dossiermanifest

```json
{
  "manifest": {
    "formaatversie": 1,
    "aanleiding": "uitvraag van 12 augustus 2026",
    "bestemd_voor": "de toezichthouder",
    "samengesteld_op": "2026-08-18T09:00:00Z",
    "samengesteld_door": "A. de Vries",
    "periode_van": null,
    "periode_tot": null,
    "keten_volgnummer": 412,
    "keten_hash": "…64 hex…",
    "anker_omschrijving": "regel 400 op 18-08-2026 12:00 UTC, bewaard in: notulen",
    "reikwijdte": "De keten is bevestigd tot en met regel 400…",
    "kennispakket_code": "nl-start",
    "kennispakket_versie": "0.1-start",
    "kennispakket_consolidatiedatum": "2026-08-18",
    "programmaversie": "0.1.0",
    "stukken": [
      {
        "naam": "verwerking-0412-K.json",
        "soort": "verwerking",
        "bron_id": "0198f2c1-…",
        "bron_versie": 3,
        "hash": "…64 hex…",
        "omvang": 2481,
        "bewerkt": false,
        "bewerking_grondslag": null
      }
    ],
    "weggelaten": [
      {
        "omschrijving": "verwerkingen met de status concept",
        "reden": "concepten zijn nog niet vastgesteld…",
        "aantal": 3
      }
    ],
    "voorbehoud": "De integriteit van dit dossier is te controleren met…"
  },
  "ondertekenaar": "…64 hex…",
  "handtekening": "…128 hex…"
}
```

De handtekening is Ed25519 over:

```
"dpo-fg-tool dossiermanifest v1" ‖ len_be64(manifest_json) ‖ manifest_json
```

De hash van elk stuk is `BLAKE3` over de bytes zoals ze in de bundel staan, hexadecimaal in kleine letters.

**Het voorbehoud wordt meegetekend.** Wie de tekst afzwakt, verbreekt de handtekening. Dat is bewust: een dossier dat meer suggereert dan het waarmaakt, is bij een inspectie erger dan geen dossier.

**Weglatingen horen erin.** Wat bewust buiten het dossier is gelaten, staat met aantal en reden in het manifest. Verzwijgen wat er ontbreekt is de snelste manier om het vertrouwen in een dossier te verliezen.

---

## 4. Zelf controleren

De meegeleverde binary `dpofg-verify` doet dit alles, leest uitsluitend, en vraagt geen wachtwoord:

```sh
dpofg-verify dossier pad/naar/manifest.json
dpofg-verify logboek pad/naar/logboek.json --anker pad/naar/anker.json
dpofg-verify anker pad/naar/anker.json
```

Afsluitcodes:

| Code | Betekenis |
|---|---|
| `0` | de controle is geslaagd |
| `1` | een bestand kon niet worden gelezen of is niet leesbaar |
| `2` | het dossier is gelezen maar klopt niet |

Het onderscheid tussen 1 en 2 telt voor wie dit in een script draait: een onleesbaar bestand is iets anders dan een dossier dat niet klopt.

Wie de binary niet vertrouwt, programmeert de controle na op basis van dit document. Dat is de bedoeling.

---

## 5. Versleuteling van de kluis

Het kluisbestand hoeft niet gecontroleerd te worden door een derde — het gaat de organisatie niet uit. Voor de volledigheid staat de opbouw hier wel.

| Laag | Algoritme |
|---|---|
| Sleutelafleiding uit de wachtwoordzin | Argon2id, parameters in het kluishoofd |
| Wikkeling van de kluissleutel | XChaCha20-Poly1305 |
| Wikkeling van elke compartimentsleutel | XChaCha20-Poly1305, met de kluissleutel |
| Versleuteling van records en bijlagen | XChaCha20-Poly1305, met de compartimentsleutel |

Elke envelop is gebonden aan veld, record en compartiment. Die drie waarden gaan als bijbehorende gegevens (AAD) mee in de authenticatie, met lengteprefixen. Een versleuteld veld dat naar een ander record wordt gekopieerd, is daardoor onleesbaar.

Compacte weergave van een envelop: `versie(1) ‖ nonce(24) ‖ cijfertekst+tag`.

---

*Deze specificatie hoort bij formaatversie 1. Wijzigingen krijgen een nieuw versienummer; de oude versie blijft leesbaar.*
