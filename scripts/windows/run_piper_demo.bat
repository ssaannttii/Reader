@echo off
setlocal enabledelayedexpansion

chcp 65001 >nul

set SCRIPT_DIR=%~dp0
set REPO_ROOT=%SCRIPT_DIR%..\..
set DEFAULT_VOICE=%REPO_ROOT%\assets\voices\es_ES\es_ES-carlfm-x_low.onnx
set OUTPUT=%REPO_ROOT%\runtime\out.wav
set PIPER_BIN=%REPO_ROOT%\runtime\piper\piper.exe

if "%~1"=="" (
    echo Uso: %~nx0 "Texto a sintetizar"
    echo Opciones: --voice=path\al\modelo.onnx --out=path\salida.wav --sentence-break=550 --length-scale=1.0 --noise-scale=0.5 --noise-w=0.9
    exit /b 1
)

set TEXT=%~1
shift

:parse_args
if "%~1"=="" goto args_done
for /f "tokens=1,2 delims==" %%A in ("%~1") do (
    set KEY=%%A
    set VALUE=%%B
)
if /i "!KEY!"=="--voice" set DEFAULT_VOICE=!VALUE!
if /i "!KEY!"=="--out" set OUTPUT=!VALUE!
if /i "!KEY!"=="--sentence-break" set SENTENCE_BREAK=!VALUE!
if /i "!KEY!"=="--length-scale" set LENGTH_SCALE=!VALUE!
if /i "!KEY!"=="--noise-scale" set NOISE_SCALE=!VALUE!
if /i "!KEY!"=="--noise-w" set NOISE_W=!VALUE!
shift
goto parse_args

:args_done
if not defined SENTENCE_BREAK set SENTENCE_BREAK=550
if not defined LENGTH_SCALE set LENGTH_SCALE=1.0
if not defined NOISE_SCALE set NOISE_SCALE=0.5
if not defined NOISE_W set NOISE_W=0.9

if not exist "%DEFAULT_VOICE%" (
    echo [ERROR] No se encontro la voz en "%DEFAULT_VOICE%"
    exit /b 2
)

if not exist "%~dp0..\..\runtime" mkdir "%~dp0..\..\runtime"

if exist "%PIPER_BIN%" (
    set CMD="%PIPER_BIN%"
) else (
    set CMD=python -m piper
)

echo [INFO] Sintetizando con !CMD! y voz "%DEFAULT_VOICE%"
cmd /c "echo !TEXT!| !CMD! -m \"%DEFAULT_VOICE%\" -f \"%OUTPUT%\" --sentence-break !SENTENCE_BREAK! --length-scale !LENGTH_SCALE! --noise-scale !NOISE_SCALE! --noise-w !NOISE_W!"
if errorlevel 1 (
    echo [ERROR] Piper devolvio un error.
    exit /b 3
)

if exist "%OUTPUT%" (
    echo [OK] Archivo generado en "%OUTPUT%"
    start "" "%OUTPUT%"
) else (
    echo [ERROR] Piper no genero "%OUTPUT%"
    exit /b 4
)

exit /b 0
