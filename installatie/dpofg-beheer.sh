#!/usr/bin/env bash
#
# Installeren, bijwerken en verwijderen van dpo-fg-tool op Linux en macOS.
#
# Twee regels sturen alles wat hier gebeurt.
#
# De eerste: dit script raakt uw gegevens niet aan. De kluis, het persoonlijke
# dossier en de reservekopieën staan buiten de programmamap en worden bij het
# verwijderen niet meegenomen — niet stilzwijgend, en ook niet na een vraag met
# een standaardantwoord. Wie ze weg wil hebben, krijgt te zien waar ze staan en
# moet zelf het woord intypen.
#
# De tweede: elke handeling zegt eerst wat zij gaat doen en waar. Een
# installatieprogramma dat pas achteraf laat zien wat het heeft aangeraakt, is
# op een werkplek met bijzondere persoonsgegevens niet acceptabel.

set -euo pipefail

# --------------------------------------------------------------------------
# Weergave
# --------------------------------------------------------------------------

if [[ -t 1 && -z "${GEEN_KLEUR:-}" ]]; then
  VET=$'\e[1m'; ZACHT=$'\e[2m'; GROEN=$'\e[32m'; GEEL=$'\e[33m'; ROOD=$'\e[31m'; BLAUW=$'\e[34m'; UIT=$'\e[0m'
else
  VET=''; ZACHT=''; GROEN=''; GEEL=''; ROOD=''; BLAUW=''; UIT=''
fi

# De streep wordt teken voor teken gezet en niet met `tr`: dat werkt op bytes,
# en een e met trema telt dan voor twee, een liggend streepje voor drie.
streep() {
  local aantal="$1" uit=''
  while (( aantal-- > 0 )); do uit+='─'; done
  printf '%s' "$uit"
}

kop() { printf '\n%s%s%s\n%s%s%s\n' "$VET" "$1" "$UIT" "$ZACHT" "$(streep "${#1}")" "$UIT"; }
gelukt()   { printf '%s✓%s %s\n' "$GROEN" "$UIT" "$1"; }
let_op()   { printf '%s▸%s %s\n' "$GEEL" "$UIT" "$1"; }
blokkade() { printf '%s■%s %s\n' "$ROOD" "$UIT" "$1" >&2; }
terzijde() { printf '%s  %s%s\n' "$ZACHT" "$1" "$UIT"; }
stap()     { printf '%s→%s %s\n' "$BLAUW" "$UIT" "$1"; }

afbreken() { blokkade "$1"; exit 1; }

# --------------------------------------------------------------------------
# Waar alles staat
# --------------------------------------------------------------------------

case "$(uname -s)" in
  Darwin)
    PROGRAMMAMAP="${HOME}/Applications/dpo-fg-tool"
    GEGEVENSMAP="${HOME}/Library/Application Support/dpo-fg-tool"
    KOPPELMAP=''
    ;;
  Linux)
    PROGRAMMAMAP="${XDG_DATA_HOME:-${HOME}/.local/share}/dpo-fg-tool/programma"
    GEGEVENSMAP="${XDG_DATA_HOME:-${HOME}/.local/share}/dpo-fg-tool"
    KOPPELMAP="${XDG_DATA_HOME:-${HOME}/.local/share}/applications"
    ;;
  *)
    afbreken "dit script werkt op Linux en macOS. Voor Windows staat er een eigen script in installatie/dpofg-beheer.ps1"
    ;;
esac

PADMAP="${HOME}/.local/bin"
WORTEL="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# --------------------------------------------------------------------------
# Vragen
# --------------------------------------------------------------------------

# Er wordt uit /dev/tty gelezen en niet uit de invoer, zodat een script dat
# antwoorden doorpijpt geen vraag kan beantwoorden die het niet gesteld heeft.
# Staat er geen terminal, dan wordt dat gezegd in plaats van dat de leesregel
# met een foutmelding uit bash afgaat.
# De volgorde van de omleidingen telt: stderr moet dicht staan vóór /dev/tty
# wordt geopend, anders schrijft de schil zelf alsnog een foutregel.
heeft_terminal() { : 2>/dev/null </dev/tty; }

geen_terminal() {
  blokkade 'hiervoor is een terminal nodig; deze opdracht vraagt om een antwoord'
  terzijde 'zonder terminal draaien kan met ONBEWAAKT=1, behalve waar een woord moet worden ingetypt'
}

# Een gewone ja-of-nee-vraag. De standaard staat in hoofdletters.
vraag() {
  local vraag="$1" standaard="${2:-n}" antwoord
  local hint='[j/N]'
  [[ "$standaard" == 'j' ]] && hint='[J/n]'
  if [[ -n "${ONBEWAAKT:-}" ]]; then
    [[ "$standaard" == 'j' ]] && return 0 || return 1
  fi
  heeft_terminal || { geen_terminal; return 1; }
  read -r -p "$(printf '%s?%s %s %s ' "$BLAUW" "$UIT" "$vraag" "$hint")" antwoord </dev/tty || antwoord=''
  antwoord="${antwoord:-$standaard}"
  [[ "${antwoord,,}" == 'j' || "${antwoord,,}" == 'ja' ]]
}

# Een bevestiging die niet met één toets te geven is.
#
# Dit is geen extra drempel om de drempel: bij een herhaalde bevestiging wordt
# klikken een verlengstuk van de klik ervóór, en dan beschermt zij niets meer.
# Wie het woord moet overtypen, leest wat er staat.
vraag_woord() {
  local vraag="$1" woord="$2" antwoord
  if [[ -n "${ONBEWAAKT:-}" ]]; then
    blokkade "deze handeling vraagt om een bevestiging en kan niet onbewaakt draaien"
    return 1
  fi
  heeft_terminal || { geen_terminal; return 1; }
  printf '%s\n' "$vraag"
  read -r -p "$(printf 'Typ %s%s%s om door te gaan: ' "$VET" "$woord" "$UIT")" antwoord </dev/tty || antwoord=''
  [[ "$antwoord" == "$woord" ]]
}

# --------------------------------------------------------------------------
# Wat er nodig is
# --------------------------------------------------------------------------

heeft() { command -v "$1" >/dev/null 2>&1; }

controleer_gereedschap() {
  local ontbreekt=()
  heeft cargo || ontbreekt+=('cargo (Rust)')
  heeft pnpm  || heeft corepack || ontbreekt+=('pnpm of corepack (Node.js)')
  heeft node  || ontbreekt+=('node')

  if [[ "$(uname -s)" == 'Linux' ]]; then
    if heeft pkg-config; then
      pkg-config --exists webkit2gtk-4.1 || ontbreekt+=('libwebkit2gtk-4.1-dev')
    else
      ontbreekt+=('pkg-config')
    fi
  fi

  if (( ${#ontbreekt[@]} > 0 )); then
    blokkade 'er ontbreekt gereedschap om te kunnen bouwen:'
    for item in "${ontbreekt[@]}"; do terzijde "- ${item}"; done
    printf '\n'
    terzijde 'Op Debian en Ubuntu:'
    terzijde '  sudo apt install build-essential pkg-config libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev'
    terzijde 'Rust: https://rustup.rs · Node: https://nodejs.org'
    return 1
  fi
  gelukt 'al het benodigde gereedschap staat er'
}

# --------------------------------------------------------------------------
# Bouwen
# --------------------------------------------------------------------------

bouwen() {
  kop 'Bouwen'
  stap 'de schil bouwen'
  ( cd "${WORTEL}/schil" && (heeft pnpm || corepack enable) && pnpm install --frozen-lockfile && pnpm exec vite build ) \
    || afbreken 'de schil kon niet worden gebouwd'

  stap 'de toepassing bouwen (dit duurt de eerste keer een aantal minuten)'
  ( cd "$WORTEL" && cargo build --release -p dpofg-schil -p dpofg-cli -p dpofg-verify ) \
    || afbreken 'de toepassing kon niet worden gebouwd'

  gelukt 'gebouwd'
}

# --------------------------------------------------------------------------
# Installeren
# --------------------------------------------------------------------------

huidige_versie() {
  [[ -f "${PROGRAMMAMAP}/versie" ]] && cat "${PROGRAMMAMAP}/versie" || printf 'niet geïnstalleerd'
}

nieuwe_versie() {
  grep -m1 '^version' "${WORTEL}/Cargo.toml" | sed 's/.*"\(.*\)".*/\1/'
}

plaatsen() {
  mkdir -p "$PROGRAMMAMAP" "$PADMAP"
  for binair in dpofg-schil dpofg dpofg-verify; do
    if [[ -f "${WORTEL}/target/release/${binair}" ]]; then
      install -m 755 "${WORTEL}/target/release/${binair}" "${PROGRAMMAMAP}/${binair}"
      ln -sf "${PROGRAMMAMAP}/${binair}" "${PADMAP}/${binair}"
      terzijde "${binair} → ${PROGRAMMAMAP}/${binair}"
    fi
  done
  nieuwe_versie > "${PROGRAMMAMAP}/versie"

  if [[ -n "$KOPPELMAP" ]]; then
    mkdir -p "$KOPPELMAP"
    cat > "${KOPPELMAP}/nl.dpofgtool.app.desktop" <<SNEDE
[Desktop Entry]
Type=Application
Name=dpo-fg-tool
Comment=Werkplatform voor de functionaris gegevensbescherming
Exec=${PROGRAMMAMAP}/dpofg-schil
Terminal=false
Categories=Office;
StartupWMClass=dpo-fg-tool
SNEDE
    heeft update-desktop-database && update-desktop-database "$KOPPELMAP" >/dev/null 2>&1 || true
    terzijde "menu-item → ${KOPPELMAP}/nl.dpofgtool.app.desktop"
  fi
}

installeren() {
  kop 'dpo-fg-tool installeren'
  terzijde "programma  → ${PROGRAMMAMAP}"
  terzijde "opdrachten → ${PADMAP}"
  terzijde "gegevens   → ${GEGEVENSMAP}  (blijft altijd staan)"
  printf '\n'

  controleer_gereedschap || exit 1
  vraag 'Nu bouwen en installeren?' j || { let_op 'afgebroken; er is niets gewijzigd'; exit 0; }

  bouwen
  kop 'Plaatsen'
  plaatsen
  gelukt "dpo-fg-tool $(nieuwe_versie) is geïnstalleerd"

  if [[ ":${PATH}:" != *":${PADMAP}:"* ]]; then
    let_op "${PADMAP} staat niet in uw PATH; de opdracht 'dpofg' werkt dan nog niet"
    terzijde "voeg toe aan ~/.bashrc of ~/.zshrc:  export PATH=\"\$PATH:${PADMAP}\""
  fi

  printf '\n'
  terzijde 'Start de schil met:   dpofg-schil'
  terzijde 'De opdrachtregel:     dpofg --help'
  terzijde 'Maak eerst een kluis: dpofg kluis nieuw'
}

# --------------------------------------------------------------------------
# Bijwerken
# --------------------------------------------------------------------------

bijwerken() {
  kop 'dpo-fg-tool bijwerken'
  local oud nieuw
  oud="$(huidige_versie)"; nieuw="$(nieuwe_versie)"
  terzijde "geïnstalleerd: ${oud}"
  terzijde "in deze map:   ${nieuw}"

  if [[ "$oud" == 'niet geïnstalleerd' ]]; then
    let_op 'er staat nog niets geïnstalleerd; gebruik "installeren"'
    exit 1
  fi
  printf '\n'
  let_op 'Uw kluis wordt niet aangeraakt. Bijwerken vervangt alleen de programmabestanden.'
  terzijde 'Maak vóór een grote sprong een reservekopie van uw kluisbestand; die staat los van dit script.'
  printf '\n'

  controleer_gereedschap || exit 1
  vraag 'Nu bouwen en bijwerken?' j || { let_op 'afgebroken; er is niets gewijzigd'; exit 0; }

  bouwen
  kop 'Vervangen'
  plaatsen
  gelukt "bijgewerkt van ${oud} naar ${nieuw}"
}

# --------------------------------------------------------------------------
# Verwijderen
# --------------------------------------------------------------------------

omvang_van() {
  [[ -e "$1" ]] && du -sh "$1" 2>/dev/null | cut -f1 || printf '0'
}

# Twee smaken stat. Er wordt op het besturingssysteem gekozen en niet op het
# mislukken van de eerste poging: GNU-stat kent `-f` ook, maar dan als
# bestandssysteeminformatie, en slaagt dus met een onbruikbaar antwoord.
gewijzigd_op() {
  if [[ "$(uname -s)" == 'Darwin' ]]; then
    stat -f '%Sm' -t '%Y-%m-%d' "$1" 2>/dev/null
  else
    stat -c '%y' "$1" 2>/dev/null | cut -d' ' -f1
  fi
}

verwijderen() {
  kop 'dpo-fg-tool verwijderen'

  if [[ ! -d "$PROGRAMMAMAP" ]]; then
    let_op 'er staat niets geïnstalleerd op deze plaats'
    terzijde "verwacht op ${PROGRAMMAMAP}"
    exit 0
  fi

  printf 'Wat er wordt verwijderd:\n'
  terzijde "het programma      ${PROGRAMMAMAP}  ($(omvang_van "$PROGRAMMAMAP"))"
  terzijde "de opdrachten      ${PADMAP}/dpofg, dpofg-schil, dpofg-verify"
  [[ -n "$KOPPELMAP" ]] && terzijde "het menu-item      ${KOPPELMAP}/nl.dpofgtool.app.desktop"
  printf '\n'
  printf 'Wat er blijft staan:\n'
  terzijde "uw gegevens        ${GEGEVENSMAP}  ($(omvang_van "$GEGEVENSMAP"))"
  if [[ -d "$GEGEVENSMAP" ]]; then
    local aantal
    aantal="$(find "$GEGEVENSMAP" -maxdepth 1 -name '*.dpofg' 2>/dev/null | wc -l | tr -d ' ')"
    terzijde "                   waaronder ${aantal} kluisbestand(en)"
  fi
  printf '\n'

  vraag 'Het programma verwijderen?' n || { let_op 'afgebroken; er is niets gewijzigd'; exit 0; }

  rm -rf "$PROGRAMMAMAP"
  for binair in dpofg-schil dpofg dpofg-verify; do rm -f "${PADMAP}/${binair}"; done
  [[ -n "$KOPPELMAP" ]] && rm -f "${KOPPELMAP}/nl.dpofgtool.app.desktop"
  gelukt 'het programma is verwijderd'

  if [[ -d "$GEGEVENSMAP" ]] && find "$GEGEVENSMAP" -mindepth 1 -print -quit 2>/dev/null | grep -q .; then
    printf '\n'
    terzijde "Uw gegevens staan nog op ${GEGEVENSMAP}."
    terzijde 'Zonder wachtwoordzin zijn ze onleesbaar, en zonder reservekopie zijn ze weg zodra u ze wist.'
    printf '\n'
    if vraag 'Wilt u die gegevens óók wissen?' n; then
      printf '\n'
      blokkade 'Dit is onomkeerbaar. Er is geen prullenbak en geen herstelmogelijkheid.'
      terzijde 'Alles wat in de kluis staat — het register, de incidenten, het bewijs, het logboek — verdwijnt.'
      terzijde 'Het persoonlijke dossier van de functionaris verdwijnt daarmee ook.'
      printf '\n'
      if vraag_woord 'Bevestig dat u dit wilt.' 'WISSEN'; then
        rm -rf "$GEGEVENSMAP"
        gelukt 'de gegevens zijn gewist'
      else
        let_op 'niet gewist; uw gegevens staan er nog'
      fi
    else
      terzijde 'De gegevens blijven staan. Bij een herinstallatie vindt de tool ze vanzelf terug.'
    fi
  fi
}

# --------------------------------------------------------------------------
# Stand
# --------------------------------------------------------------------------

stand() {
  kop 'Stand van zaken'
  local oud; oud="$(huidige_versie)"
  # Uitlijnen met eigen opvulling, want printf telt bytes en geen tekens.
  regel() {
    local naam="$1" waarde="$2" opvulling=''
    local tekort=$(( 22 - ${#naam} ))
    while (( tekort-- > 0 )); do opvulling+=' '; done
    printf '%s%s %s\n' "$naam" "$opvulling" "$waarde"
  }
  regel 'geïnstalleerde versie' "$oud"
  regel 'versie in deze map' "$(nieuwe_versie)"
  regel 'programmamap' "$PROGRAMMAMAP"
  regel 'gegevensmap' "$GEGEVENSMAP"
  printf '\n'

  if [[ -d "$GEGEVENSMAP" ]]; then
    local kluizen
    kluizen="$(find "$GEGEVENSMAP" -maxdepth 1 -name '*.dpofg' 2>/dev/null | wc -l | tr -d ' ')"
    if (( kluizen > 0 )); then
      gelukt "${kluizen} kluisbestand(en) gevonden"
      # Geen `find -printf`: dat is GNU-eigen en ontbreekt op macOS.
      local pad
      for pad in "$GEGEVENSMAP"/*.dpofg; do
        [[ -e "$pad" ]] || continue
        terzijde "$(basename "$pad")  $(omvang_van "$pad")  gewijzigd $(gewijzigd_op "$pad")"
      done
    else
      let_op 'er staat nog geen kluis; maak er een met: dpofg kluis nieuw'
    fi
  else
    let_op 'er is nog geen gegevensmap; die ontstaat bij de eerste kluis'
  fi

  printf '\n'
  if [[ "$oud" != 'niet geïnstalleerd' && "$oud" != "$(nieuwe_versie)" ]]; then
    let_op "er staat een nieuwere versie klaar in deze map; werk bij met: $0 bijwerken"
  fi
}

# --------------------------------------------------------------------------
# Het menu
# --------------------------------------------------------------------------

menu() {
  kop 'dpo-fg-tool — beheer'
  terzijde "geïnstalleerd: $(huidige_versie) · in deze map: $(nieuwe_versie)"
  printf '\n'
  printf '  %s1%s  installeren\n' "$VET" "$UIT"
  printf '  %s2%s  bijwerken\n' "$VET" "$UIT"
  printf '  %s3%s  stand van zaken\n' "$VET" "$UIT"
  printf '  %s4%s  verwijderen\n' "$VET" "$UIT"
  printf '  %s5%s  stoppen\n' "$VET" "$UIT"
  printf '\n'
  local keuze
  heeft_terminal || { geen_terminal; exit 1; }
  read -r -p "$(printf '%s?%s Uw keuze [1-5]: ' "$BLAUW" "$UIT")" keuze </dev/tty || keuze='5'
  case "$keuze" in
    1) installeren ;;
    2) bijwerken ;;
    3) stand ;;
    4) verwijderen ;;
    5) printf 'Tot ziens.\n' ;;
    *) blokkade 'onbekende keuze' ; exit 1 ;;
  esac
}

gebruik() {
  cat <<UITLEG
dpo-fg-tool — installeren, bijwerken en verwijderen

Gebruik: $0 [opdracht]

  installeren   bouwt en plaatst het programma
  bijwerken     vervangt de programmabestanden; raakt uw kluis niet aan
  stand         toont wat er staat en waar
  verwijderen   haalt het programma weg; uw gegevens blijven staan tenzij u
                uitdrukkelijk anders bevestigt
  hulp          deze uitleg

Zonder opdracht verschijnt een menu.

Omgevingsvariabelen:
  ONBEWAAKT=1   beantwoordt de gewone vragen met hun standaard. Handelingen die
                een overgetypt woord vragen, blijven geweigerd.
  GEEN_KLEUR=1  laat de kleuren weg.
UITLEG
}

case "${1:-}" in
  installeren|install) installeren ;;
  bijwerken|update)    bijwerken ;;
  verwijderen|uninstall) verwijderen ;;
  stand|status)        stand ;;
  hulp|--help|-h)      gebruik ;;
  '')                  menu ;;
  *) blokkade "onbekende opdracht: $1"; printf '\n'; gebruik; exit 1 ;;
esac
