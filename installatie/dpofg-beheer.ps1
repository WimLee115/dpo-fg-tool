#Requires -Version 5.1
<#
    Installeren, bijwerken en verwijderen van dpo-fg-tool op Windows.

    Dit script volgt dezelfde twee regels als zijn tegenhanger voor Linux en
    macOS in installatie/dpofg-beheer.sh.

    De eerste: uw gegevens worden niet aangeraakt. De kluis en het persoonlijke
    dossier staan in %APPDATA%\dpo-fg-tool en blijven bij het verwijderen staan
    tenzij u het woord WISSEN overtypt.

    De tweede: elke handeling zegt eerst wat zij gaat doen en waar.

    Gebruik:  .\dpofg-beheer.ps1 [installeren|bijwerken|stand|verwijderen|hulp]
              zonder opdracht verschijnt een menu.

    Omgevingsvariabelen:
      ONBEWAAKT=1   beantwoordt de gewone vragen met hun standaard
      GEEN_KLEUR=1  laat de kleuren weg
#>

[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$Opdracht = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
# De uitvoer bevat accenten en liggende streepjes; zonder dit toont de oudere
# console er vraagtekens voor.
try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch { }

# --------------------------------------------------------------------------
# Weergave
# --------------------------------------------------------------------------

$Kleurloos = ($env:GEEN_KLEUR) -or -not $Host.UI.SupportsVirtualTerminal

function Schrijf {
    param([string]$Tekst, [string]$Kleur = 'Gray')
    if ($Kleurloos) { Write-Host $Tekst } else { Write-Host $Tekst -ForegroundColor $Kleur }
}

function Kop {
    param([string]$Tekst)
    Write-Host ''
    Schrijf $Tekst 'White'
    Schrijf ('─' * $Tekst.Length) 'DarkGray'
}

function Gelukt   { param([string]$T) Schrijf "+ $T" 'Green' }
function LetOp    { param([string]$T) Schrijf "> $T" 'Yellow' }
function Blokkade { param([string]$T) Schrijf "! $T" 'Red' }
function Terzijde { param([string]$T) Schrijf "  $T" 'DarkGray' }
function Stap     { param([string]$T) Schrijf "- $T" 'Cyan' }

function Afbreken {
    param([string]$T)
    Blokkade $T
    exit 1
}

# --------------------------------------------------------------------------
# Waar alles staat
# --------------------------------------------------------------------------

if (-not $env:APPDATA -or -not $env:LOCALAPPDATA) {
    Blokkade 'APPDATA of LOCALAPPDATA is niet gezet; dit script hoort op Windows te draaien'
    Terzijde 'Op Linux en macOS staat er een eigen script in installatie/dpofg-beheer.sh'
    exit 1
}

$Gegevensmap  = Join-Path $env:APPDATA 'dpo-fg-tool'
$Programmamap = Join-Path $env:LOCALAPPDATA 'Programs\dpo-fg-tool'
$Wortel       = Split-Path -Parent $PSScriptRoot
# GetFolderPath geeft een lege tekst terug als de map niet bestaat. Dat mag
# het script niet laten struikelen: zonder startmenu werkt het programma
# verder gewoon, alleen de snelkoppeling vervalt dan.
$Startmenu = [Environment]::GetFolderPath('Programs')
$Snelkoppeling = if ($Startmenu) { Join-Path $Startmenu 'dpo-fg-tool.lnk' } else { '' }
$Binairen     = @('dpofg-schil.exe', 'dpofg.exe', 'dpofg-verify.exe')

# --------------------------------------------------------------------------
# Vragen
# --------------------------------------------------------------------------

function Vraag {
    param([string]$Tekst, [string]$Standaard = 'n')

    $hint = if ($Standaard -eq 'j') { '[J/n]' } else { '[j/N]' }
    if ($env:ONBEWAAKT) { return ($Standaard -eq 'j') }

    $antwoord = Read-Host "? $Tekst $hint"
    if ([string]::IsNullOrWhiteSpace($antwoord)) { $antwoord = $Standaard }
    return ($antwoord.ToLowerInvariant() -in @('j', 'ja'))
}

# Een bevestiging die niet met één toets te geven is. Bij een herhaalde
# bevestiging wordt klikken een verlengstuk van de klik ervóór, en dan
# beschermt zij niets meer. Wie het woord moet overtypen, leest wat er staat.
function VraagWoord {
    param([string]$Tekst, [string]$Woord)

    if ($env:ONBEWAAKT) {
        Blokkade 'deze handeling vraagt om een bevestiging en kan niet onbewaakt draaien'
        return $false
    }
    Write-Host $Tekst
    $antwoord = Read-Host "Typ $Woord om door te gaan"
    return ($antwoord -ceq $Woord)
}

# --------------------------------------------------------------------------
# Wat er nodig is
# --------------------------------------------------------------------------

function Heeft { param([string]$Naam) [bool](Get-Command $Naam -ErrorAction SilentlyContinue) }

function ControleerGereedschap {
    $ontbreekt = @()
    if (-not (Heeft 'cargo')) { $ontbreekt += 'cargo (Rust)' }
    if (-not (Heeft 'pnpm') -and -not (Heeft 'corepack')) { $ontbreekt += 'pnpm of corepack (Node.js)' }
    if (-not (Heeft 'node')) { $ontbreekt += 'node' }

    if ($ontbreekt.Count -gt 0) {
        Blokkade 'er ontbreekt gereedschap om te kunnen bouwen:'
        foreach ($item in $ontbreekt) { Terzijde "- $item" }
        Write-Host ''
        Terzijde 'Rust:            https://rustup.rs'
        Terzijde 'Node.js:         https://nodejs.org'
        Terzijde 'WebView2 hoort bij Windows 11 en recente Windows 10; anders:'
        Terzijde '  https://developer.microsoft.com/microsoft-edge/webview2/'
        return $false
    }
    Gelukt 'al het benodigde gereedschap staat er'
    return $true
}

# --------------------------------------------------------------------------
# Bouwen
# --------------------------------------------------------------------------

function Bouwen {
    Kop 'Bouwen'

    Stap 'de schil bouwen'
    Push-Location (Join-Path $Wortel 'schil')
    try {
        if (-not (Heeft 'pnpm')) { corepack enable }
        pnpm install --frozen-lockfile
        if ($LASTEXITCODE -ne 0) { Afbreken 'de schil kon niet worden gebouwd' }
        pnpm exec vite build
        if ($LASTEXITCODE -ne 0) { Afbreken 'de schil kon niet worden gebouwd' }
    } finally { Pop-Location }

    Stap 'de toepassing bouwen (dit duurt de eerste keer een aantal minuten)'
    Push-Location $Wortel
    try {
        cargo build --release -p dpofg-schil -p dpofg-cli -p dpofg-verify
        if ($LASTEXITCODE -ne 0) { Afbreken 'de toepassing kon niet worden gebouwd' }
    } finally { Pop-Location }

    Gelukt 'gebouwd'
}

# --------------------------------------------------------------------------
# Versies
# --------------------------------------------------------------------------

function HuidigeVersie {
    $pad = Join-Path $Programmamap 'versie'
    if (Test-Path $pad) { (Get-Content $pad -Raw).Trim() } else { 'niet geïnstalleerd' }
}

function NieuweVersie {
    $regel = Select-String -Path (Join-Path $Wortel 'Cargo.toml') -Pattern '^version' | Select-Object -First 1
    if ($regel -and $regel.Line -match '"([^"]+)"') { $Matches[1] } else { 'onbekend' }
}

function Plaatsen {
    New-Item -ItemType Directory -Force -Path $Programmamap | Out-Null
    foreach ($binair in $Binairen) {
        $bron = Join-Path $Wortel "target\release\$binair"
        if (Test-Path $bron) {
            Copy-Item $bron (Join-Path $Programmamap $binair) -Force
            Terzijde "$binair -> $Programmamap\$binair"
        }
    }
    Set-Content -Path (Join-Path $Programmamap 'versie') -Value (NieuweVersie) -NoNewline

    # Een snelkoppeling in het startmenu, zodat de schil niet alleen vanaf de
    # opdrachtregel te starten is.
    try {
        $schil = Join-Path $Programmamap 'dpofg-schil.exe'
        if ($Snelkoppeling -and (Test-Path $schil)) {
            $sh = New-Object -ComObject WScript.Shell
            $lnk = $sh.CreateShortcut($Snelkoppeling)
            $lnk.TargetPath = $schil
            $lnk.WorkingDirectory = $Programmamap
            $lnk.Description = 'Werkplatform voor de functionaris gegevensbescherming'
            $lnk.Save()
            Terzijde "startmenu -> $Snelkoppeling"
        }
    } catch {
        LetOp 'de snelkoppeling in het startmenu kon niet worden gemaakt; het programma werkt verder gewoon'
    }

    ZetInPad
}

# Het pad wordt bijgewerkt voor de gebruiker en niet voor de machine: dit
# script vraagt nergens om beheerdersrechten.
function ZetInPad {
    $pad = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ($null -eq $pad) { $pad = '' }
    if (($pad -split ';') -notcontains $Programmamap) {
        $nieuw = if ($pad.TrimEnd(';')) { "$($pad.TrimEnd(';'));$Programmamap" } else { $Programmamap }
        [Environment]::SetEnvironmentVariable('Path', $nieuw, 'User')
        LetOp "$Programmamap is aan uw PATH toegevoegd; open een nieuw venster voordat 'dpofg' werkt"
    }
}

# --------------------------------------------------------------------------
# Installeren
# --------------------------------------------------------------------------

function Installeren {
    Kop 'dpo-fg-tool installeren'
    Terzijde "programma -> $Programmamap"
    Terzijde "gegevens  -> $Gegevensmap  (blijft altijd staan)"
    Write-Host ''

    if (-not (ControleerGereedschap)) { exit 1 }
    if (-not (Vraag 'Nu bouwen en installeren?' 'j')) {
        LetOp 'afgebroken; er is niets gewijzigd'
        return
    }

    Bouwen
    Kop 'Plaatsen'
    Plaatsen
    Gelukt "dpo-fg-tool $(NieuweVersie) is geïnstalleerd"

    Write-Host ''
    Terzijde 'Start de schil met:   dpofg-schil'
    Terzijde 'De opdrachtregel:     dpofg --help'
    Terzijde 'Maak eerst een kluis: dpofg kluis nieuw'
}

# --------------------------------------------------------------------------
# Bijwerken
# --------------------------------------------------------------------------

function Bijwerken {
    Kop 'dpo-fg-tool bijwerken'
    $oud = HuidigeVersie
    $nieuw = NieuweVersie
    Terzijde "geïnstalleerd: $oud"
    Terzijde "in deze map:   $nieuw"

    if ($oud -eq 'niet geïnstalleerd') {
        LetOp 'er staat nog niets geïnstalleerd; gebruik "installeren"'
        exit 1
    }
    Write-Host ''
    LetOp 'Uw kluis wordt niet aangeraakt. Bijwerken vervangt alleen de programmabestanden.'
    Terzijde 'Maak vóór een grote sprong een reservekopie van uw kluisbestand; die staat los van dit script.'
    Write-Host ''

    if (-not (ControleerGereedschap)) { exit 1 }
    if (-not (Vraag 'Nu bouwen en bijwerken?' 'j')) {
        LetOp 'afgebroken; er is niets gewijzigd'
        return
    }

    Bouwen
    Kop 'Vervangen'
    Plaatsen
    Gelukt "bijgewerkt van $oud naar $nieuw"
}

# --------------------------------------------------------------------------
# Verwijderen
# --------------------------------------------------------------------------

function OmvangVan {
    param([string]$Pad)
    if (-not (Test-Path $Pad)) { return '0' }
    $bytes = (Get-ChildItem $Pad -Recurse -File -ErrorAction SilentlyContinue |
              Measure-Object -Property Length -Sum).Sum
    if (-not $bytes) { return '0' }
    if ($bytes -ge 1MB) { return ('{0:N1} MB' -f ($bytes / 1MB)) }
    return ('{0:N0} kB' -f ($bytes / 1KB))
}

function Kluisbestanden {
    if (-not (Test-Path $Gegevensmap)) { return @() }
    @(Get-ChildItem $Gegevensmap -Filter '*.dpofg' -File -ErrorAction SilentlyContinue)
}

function HaalUitPad {
    $pad = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ($null -eq $pad) { return }
    $delen = @($pad -split ';' | Where-Object { $_ -and $_ -ne $Programmamap })
    [Environment]::SetEnvironmentVariable('Path', ($delen -join ';'), 'User')
}

function Verwijderen {
    Kop 'dpo-fg-tool verwijderen'

    if (-not (Test-Path $Programmamap)) {
        LetOp 'er staat niets geïnstalleerd op deze plaats'
        Terzijde "verwacht op $Programmamap"
        return
    }

    Write-Host 'Wat er wordt verwijderd:'
    Terzijde "het programma      $Programmamap  ($(OmvangVan $Programmamap))"
    Terzijde "de PATH-regel      $Programmamap"
    if ($Snelkoppeling -and (Test-Path $Snelkoppeling)) { Terzijde "de snelkoppeling   $Snelkoppeling" }
    Write-Host ''
    Write-Host 'Wat er blijft staan:'
    Terzijde "uw gegevens        $Gegevensmap  ($(OmvangVan $Gegevensmap))"
    $kluizen = @(Kluisbestanden)
    if ($kluizen.Count -gt 0) { Terzijde "                   waaronder $($kluizen.Count) kluisbestand(en)" }
    Write-Host ''

    if (-not (Vraag 'Het programma verwijderen?' 'n')) {
        LetOp 'afgebroken; er is niets gewijzigd'
        return
    }

    Remove-Item $Programmamap -Recurse -Force
    if ($Snelkoppeling -and (Test-Path $Snelkoppeling)) { Remove-Item $Snelkoppeling -Force }
    HaalUitPad
    Gelukt 'het programma is verwijderd'

    if ((Test-Path $Gegevensmap) -and (Get-ChildItem $Gegevensmap -Force | Select-Object -First 1)) {
        Write-Host ''
        Terzijde "Uw gegevens staan nog op $Gegevensmap."
        Terzijde 'Zonder wachtwoordzin zijn ze onleesbaar, en zonder reservekopie zijn ze weg zodra u ze wist.'
        Write-Host ''
        if (Vraag 'Wilt u die gegevens óók wissen?' 'n') {
            Write-Host ''
            Blokkade 'Dit is onomkeerbaar. Er is geen prullenbak en geen herstelmogelijkheid.'
            Terzijde 'Alles wat in de kluis staat — het register, de incidenten, het bewijs, het logboek — verdwijnt.'
            Terzijde 'Het persoonlijke dossier van de functionaris verdwijnt daarmee ook.'
            Write-Host ''
            if (VraagWoord 'Bevestig dat u dit wilt.' 'WISSEN') {
                Remove-Item $Gegevensmap -Recurse -Force
                Gelukt 'de gegevens zijn gewist'
            } else {
                LetOp 'niet gewist; uw gegevens staan er nog'
            }
        } else {
            Terzijde 'De gegevens blijven staan. Bij een herinstallatie vindt de tool ze vanzelf terug.'
        }
    }
}

# --------------------------------------------------------------------------
# Stand
# --------------------------------------------------------------------------

function Stand {
    Kop 'Stand van zaken'
    $oud = HuidigeVersie
    $nieuw = NieuweVersie
    foreach ($paar in @(
        @('geïnstalleerde versie', $oud),
        @('versie in deze map',    $nieuw),
        @('programmamap',          $Programmamap),
        @('gegevensmap',           $Gegevensmap)
    )) {
        Write-Host ('{0} {1}' -f $paar[0].PadRight(22), $paar[1])
    }
    Write-Host ''

    if (Test-Path $Gegevensmap) {
        $kluizen = @(Kluisbestanden)
        if ($kluizen.Count -gt 0) {
            Gelukt "$($kluizen.Count) kluisbestand(en) gevonden"
            foreach ($k in $kluizen) {
                Terzijde ('{0}  {1:N0} bytes  gewijzigd {2:yyyy-MM-dd}' -f $k.Name, $k.Length, $k.LastWriteTime)
            }
        } else {
            LetOp 'er staat nog geen kluis; maak er een met: dpofg kluis nieuw'
        }
    } else {
        LetOp 'er is nog geen gegevensmap; die ontstaat bij de eerste kluis'
    }

    Write-Host ''
    if ($oud -ne 'niet geïnstalleerd' -and $oud -ne $nieuw) {
        LetOp 'er staat een nieuwere versie klaar in deze map; werk bij met: .\dpofg-beheer.ps1 bijwerken'
    }
}

# --------------------------------------------------------------------------
# Het menu
# --------------------------------------------------------------------------

function Menu {
    Kop 'dpo-fg-tool — beheer'
    Terzijde "geïnstalleerd: $(HuidigeVersie) · in deze map: $(NieuweVersie)"
    Write-Host ''
    Write-Host '  1  installeren'
    Write-Host '  2  bijwerken'
    Write-Host '  3  stand van zaken'
    Write-Host '  4  verwijderen'
    Write-Host '  5  stoppen'
    Write-Host ''
    switch (Read-Host '? Uw keuze [1-5]') {
        '1' { Installeren }
        '2' { Bijwerken }
        '3' { Stand }
        '4' { Verwijderen }
        '5' { Write-Host 'Tot ziens.' }
        default { Afbreken 'onbekende keuze' }
    }
}

function Gebruik {
    Write-Host @'
dpo-fg-tool — installeren, bijwerken en verwijderen

Gebruik: .\dpofg-beheer.ps1 [opdracht]

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
'@
}

switch ($Opdracht.ToLowerInvariant()) {
    'installeren'  { Installeren }
    'install'      { Installeren }
    'bijwerken'    { Bijwerken }
    'update'       { Bijwerken }
    'verwijderen'  { Verwijderen }
    'uninstall'    { Verwijderen }
    'stand'        { Stand }
    'status'       { Stand }
    'hulp'         { Gebruik }
    '--help'       { Gebruik }
    '-h'           { Gebruik }
    ''             { Menu }
    default {
        Blokkade "onbekende opdracht: $Opdracht"
        Write-Host ''
        Gebruik
        exit 1
    }
}
