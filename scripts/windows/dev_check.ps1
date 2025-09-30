#!/usr/bin/env pwsh
<#!
.SYNOPSIS
    Script de verificación del entorno de desarrollo para Reader en Windows.
.DESCRIPTION
    Comprueba dependencias mínimas (Python/pip), instala Piper, descarga voces
    esenciales y ejecuta una demo corta confirmando que la síntesis funciona.
    Finalmente, valida que la UI y el backend Tauri compilan en modo "dry run".
    Todos los pasos emiten mensajes claros y el script retorna 0 en caso de
    éxito o 1 si falla alguna comprobación.
#>

[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "[OK] $Message" -ForegroundColor Green
}

function Write-Failure {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Fail {
    param([string]$Message)
    Write-Failure $Message
    exit 1
}

try {
    $repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..\..')
    Write-Info "Repositorio detectado en $repoRoot"

    $python = Get-Command python -ErrorAction SilentlyContinue
    if (-not $python) {
        Fail "Python no está disponible en PATH. Instálalo desde https://www.python.org/downloads/"
    }

    $pyVersion = & python -c "import sys; print('.'.join(map(str, sys.version_info[:3])))"
    Write-Info "Python detectado: versión $pyVersion"
    $majorMinor = $pyVersion.Split('.')
    if ([int]$majorMinor[0] -lt 3 -or ([int]$majorMinor[0] -eq 3 -and [int]$majorMinor[1] -lt 9)) {
        Fail "Se requiere Python 3.9 o superior."
    }

    Write-Info "Actualizando pip e instalando piper-tts (scope de usuario)"
    & python -m pip install --upgrade pip piper-tts --user | Write-Output
    Write-Success "pip y piper-tts instalados/actualizados"

    $voices = @('es_ES-carlfm-x_low', 'es_ES-mls_10246-low')
    $voiceTarget = Join-Path $repoRoot 'assets\voices\es_ES'
    if (-not (Test-Path $voiceTarget)) {
        Write-Info "Creando carpeta de voces: $voiceTarget"
        New-Item -ItemType Directory -Path $voiceTarget | Out-Null
    }

    $downloadArgs = @('-m', 'piper.download_voices', '--output-dir', $voiceTarget, '--language', 'es_ES') + $voices
    Write-Info "Descargando voces Piper: $($voices -join ', ')"
    & python @downloadArgs | Write-Output
    Write-Success "Voces disponibles en $voiceTarget"

    $defaultVoice = Join-Path $voiceTarget 'es_ES-carlfm-x_low.onnx'
    if (-not (Test-Path $defaultVoice)) {
        Fail "No se encontró la voz predeterminada en $defaultVoice"
    }

    $runtimeDir = Join-Path $repoRoot 'runtime'
    if (-not (Test-Path $runtimeDir)) {
        New-Item -ItemType Directory -Path $runtimeDir | Out-Null
    }
    $outPath = Join-Path $runtimeDir 'out.wav'
    Write-Info "Ejecutando demo Piper (Hola)"
    "Hola" | & python -m piper -m $defaultVoice -f $outPath
    if (-not (Test-Path $outPath)) {
        Fail "No se generó $outPath"
    }
    Write-Success "Demo Piper generada en $outPath"

    try {
        Write-Info "Abriendo $outPath con el reproductor predeterminado"
        Start-Process -FilePath $outPath
    } catch {
        Write-Info "No se pudo abrir automáticamente el WAV: $($_.Exception.Message)"
    }

    $uiDir = Join-Path $repoRoot 'ui'
    if (Test-Path (Join-Path $uiDir 'package.json')) {
        Write-Info "Instalando dependencias UI (npm ci)"
        Push-Location $uiDir
        if (Test-Path 'package-lock.json') {
            npm ci | Write-Output
        } else {
            npm install | Write-Output
        }
        Pop-Location
    } else {
        Write-Info "No se encontró UI completa. Se ejecutará cargo tauri build --dry-run"
    }

    Push-Location $repoRoot
    Write-Info "Validando compilación del backend (cargo tauri build --dry-run)"
    cargo tauri build --dry-run | Write-Output
    Pop-Location

    Write-Success "Entorno verificado correctamente"
    exit 0
} catch {
    Write-Failure $_.Exception.Message
    exit 1
}
