@echo off
setlocal enabledelayedexpansion

REM ------------------------------------------------------------------
REM  Piper quickstart helper for Windows.
REM  Usage:
REM    run_piper_demo.bat "Texto a locutar"
REM    run_piper_demo.bat -f path\al\archivo.txt
REM  Variables opcionales:
REM    PIPER_VOICE -> Ruta completa al modelo .onnx a usar.
REM  The script expects Piper and the es_ES voices under runtime/ and assets/.
REM ------------------------------------------------------------------

set PROJECT_ROOT=%~dp0..\..
set RUNTIME_DIR=%PROJECT_ROOT%\runtime
set PIPER_EXE=%RUNTIME_DIR%\piper\piper.exe
set VOICES_DIR=%PROJECT_ROOT%\assets\voices\es_ES
set DEFAULT_VOICE=
set OUTPUT_WAV=%RUNTIME_DIR%\out.wav

if not exist "%PIPER_EXE%" (
  echo [ERROR] Piper no encontrado en %PIPER_EXE%.
  echo         Descarga piper_windows_amd64.zip desde:
  echo         https://github.com/rhasspy/piper/releases/latest
  echo         y extrae piper.exe en runtime\piper\
  exit /b 1
)

if not exist "%VOICES_DIR%" (
  echo [ERROR] No se encuentra la carpeta de voces:
  echo         %VOICES_DIR%
  echo         Crea la carpeta y coloca allí al menos un modelo es_ES (.onnx).
  exit /b 1
)

for %%F in ("%VOICES_DIR%\*.onnx") do (
  set DEFAULT_VOICE=%%~fF
  goto voice_found
)

echo [ERROR] No se encontró ninguna voz .onnx en %VOICES_DIR%.
echo         Descarga una voz es_ES de 22.05 kHz (calidad high) y colócala allí.
exit /b 1

:voice_found

if not "%PIPER_VOICE%"=="" (
  if exist "%PIPER_VOICE%" (
    set DEFAULT_VOICE=%PIPER_VOICE%
  ) else (
    echo [WARN] Ignorando PIPER_VOICE, no se encontró "%PIPER_VOICE%".
  )
)

echo [INFO] Usando voz: %DEFAULT_VOICE%

set INPUT_TEXT=
set INPUT_FILE=

if /i "%1"=="-f" (
  if "%2"=="" (
    echo [ERROR] Debes indicar un archivo de entrada tras -f.
    exit /b 1
  )
  if not exist "%2" (
    echo [ERROR] No puedo encontrar "%2".
    exit /b 1
  )
  set INPUT_FILE=%~f2
) else if not "%1"=="" (
  set INPUT_TEXT=%~1
) else (
  echo Introduce el texto a locutar y presiona ENTER.
  set /p INPUT_TEXT=Texto: 
)

if not "%INPUT_TEXT%"=="" (
  if not exist "%RUNTIME_DIR%" mkdir "%RUNTIME_DIR%"
  set INPUT_FILE=%RUNTIME_DIR%\tmp_input.txt
  >"%INPUT_FILE%" echo %INPUT_TEXT%
  echo [INFO] Texto guardado temporalmente en "%INPUT_FILE%".
)

if "%INPUT_FILE%"=="" (
  echo [ERROR] No se recibió texto a leer.
  exit /b 1
)

echo [INFO] Generando audio con Piper...
"%PIPER_EXE%" -m "%DEFAULT_VOICE%" --sentence-break "500" -f "%OUTPUT_WAV%" < "%INPUT_FILE%"
if errorlevel 1 (
  echo [ERROR] Piper terminó con errores.
  exit /b 1
)

echo [OK] Audio generado en "%OUTPUT_WAV%".
echo Ábrelo con tu reproductor favorito.

endlocal
