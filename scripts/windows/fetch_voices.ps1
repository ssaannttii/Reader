#!/usr/bin/env pwsh
<#!
.SYNOPSIS
    Descarga voces Piper es_ES y valida su integridad básica.
#>

[CmdletBinding()]
param(
    [string[]]$Voices = @('es_ES-carlfm-x_low', 'es_ES-mls_10246-low'),
    [string]$Language = 'es_ES',
    [int]$Retries = 3
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Cyan
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Write-Err {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..\..')
$targetDir = Join-Path $repoRoot "assets\voices\$Language"
if (-not (Test-Path $targetDir)) {
    New-Item -ItemType Directory -Path $targetDir | Out-Null
}

$python = Get-Command python -ErrorAction SilentlyContinue
if (-not $python) {
    Write-Err "Python no está disponible en PATH."
    exit 1
}

foreach ($voice in $Voices) {
    $attempt = 0
    $voiceBase = "$voice.onnx"
    $voiceModel = Join-Path $targetDir $voiceBase
    $voiceMeta = Join-Path $targetDir "$voice.onnx.json"
    while ($attempt -lt $Retries) {
        try {
            $attempt++
            Write-Info "Descargando $voice (intento $attempt de $Retries)"
            & python -m piper.download_voices --language $Language --output-dir $targetDir $voice | Write-Output
            if (-not (Test-Path $voiceModel)) {
                throw "Falta el modelo $voiceModel"
            }
            if (-not (Test-Path $voiceMeta)) {
                throw "Falta el metadato $voiceMeta"
            }

            try {
                $metaJson = Get-Content $voiceMeta -Raw | ConvertFrom-Json
                if ($metaJson.sha256) {
                    Write-Info "Verificando checksum sha256"
                    $hash = Get-FileHash -Path $voiceModel -Algorithm SHA256
                    if ($hash.Hash -ne $metaJson.sha256.ToUpper()) {
                        throw "Checksum inválido para $voiceModel"
                    }
                }
            } catch {
                Write-Warn "No se pudo validar checksum: $($_.Exception.Message)"
            }

            Write-Info "Voz $voice disponible en $voiceModel"
            break
        } catch {
            Write-Warn $_.Exception.Message
            if ($attempt -ge $Retries) {
                Write-Err "Error persistente al descargar $voice"
                exit 1
            }
            Start-Sleep -Seconds 2
        }
    }
}

Write-Info "Voces listas en $targetDir"
