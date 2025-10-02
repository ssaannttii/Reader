@echo off
setlocal enabledelayedexpansion

for %%I in ("%~dp0..\..") do set "PROJECT_ROOT=%%~fI"
set "VENV_DIR=%PROJECT_ROOT%\.reader_venv"
set "PIPER_DIR=%PROJECT_ROOT%\runtime\piper"
set "VOICES_DIR=%PROJECT_ROOT%\assets\voices\es_ES"
set "BUNDLED_VOICE=%PROJECT_ROOT%\es_ES-carlfm-x_low.onnx"
set "BUNDLED_META=%PROJECT_ROOT%\es_ES-carlfm-x_low.onnx.json"
set "RUN_DEMO=%PROJECT_ROOT%\run_piper_demo.bat"
set "HAS_VOICES=0"

set "SCRIPT_TITLE=Reader - Instalador automático"
title %SCRIPT_TITLE%

echo ==================================================
echo     %SCRIPT_TITLE%
echo ==================================================
echo.

echo [1/5] Buscando Python 3.10 o superior...
call :ensure_python || goto :error

echo.
echo [2/5] Creando entorno virtual dedicado...
call :ensure_venv || goto :error

echo.
echo [3/5] Instalando Piper dentro del proyecto...
call :install_piper || goto :error

echo.
echo [4/5] Verificando voces disponibles...
call :ensure_voice || goto :error

echo.
echo [5/5] Todo listo. ¿Quieres probar Piper ahora?
call :offer_demo

:success
echo.
echo ================================================
echo  ✅ Instalación completada.
echo.
echo  • Piper.exe queda en runtime\piper\
if "%HAS_VOICES%"=="1" (
  echo  • Voces detectadas en assets\voices\es_ES\
) else (
  echo  • No se detectaron voces instaladas.
)
echo  • Ejecuta run_piper_demo.bat cuando quieras volver.
echo ================================================
echo.
pause
endlocal
exit /b 0

:error
echo.
echo ================================================
echo  ❌ Se encontró un problema. Revisa los mensajes.
echo ================================================
echo.
pause
endlocal
exit /b 1

:ensure_python
set "PYTHON_CMD="
where py >nul 2>nul
if not errorlevel 1 (
  set "PYTHON_CMD=py -3"
) else (
  where python >nul 2>nul
  if not errorlevel 1 (
    set "PYTHON_CMD=python"
  )
)

if not defined PYTHON_CMD (
  echo [ERROR] No se encontró Python en PATH.
  echo         Instala Python 3.10+ desde https://www.python.org/downloads/windows/
  exit /b 1
)

call %PYTHON_CMD% -c "import sys; sys.exit(0 if sys.version_info >= (3, 10) else 1)" >nul 2>nul
if errorlevel 1 (
  echo [ERROR] Se requiere Python 3.10 o superior.
  echo         Instala la versión recomendada desde https://www.python.org/downloads/windows/
  exit /b 1
)

echo [OK] Python disponible (^%PYTHON_CMD%^).
exit /b 0

:ensure_venv
set "VENV_PY=%VENV_DIR%\Scripts\python.exe"
if exist "%VENV_PY%" (
  echo [OK] Ya existe el entorno virtual .reader_venv.
  exit /b 0
)

echo [INFO] Creando el entorno virtual...
call %PYTHON_CMD% -m venv "%VENV_DIR%"
if errorlevel 1 (
  echo [ERROR] No se pudo crear el entorno virtual en "%VENV_DIR%".
  exit /b 1
)

echo [OK] Entorno virtual creado.
exit /b 0

:install_piper
set "VENV_PY=%VENV_DIR%\Scripts\python.exe"
set "VENV_PIPER=%VENV_DIR%\Scripts\piper.exe"
if not exist "%VENV_PY%" (
  echo [ERROR] No se encontró python.exe en el entorno virtual.
  exit /b 1
)

echo [INFO] Actualizando pip...
call "%VENV_PY%" -m pip install --upgrade pip setuptools wheel >nul
if errorlevel 1 (
  echo [ERROR] Falló la actualización de pip.
  exit /b 1
)

echo [INFO] Instalando/actualizando piper-tts...
call "%VENV_PY%" -m pip install --upgrade piper-tts >nul
if errorlevel 1 (
  echo [ERROR] No se pudo instalar piper-tts en el entorno virtual.
  exit /b 1
)

if not exist "%PIPER_DIR%" mkdir "%PIPER_DIR%"
if exist "%VENV_PIPER%" (
  copy /Y "%VENV_PIPER%" "%PIPER_DIR%\" >nul
  for %%F in (piper.dll piper.phonemizer.exe) do (
    if exist "%VENV_DIR%\Scripts\%%F" copy /Y "%VENV_DIR%\Scripts\%%F" "%PIPER_DIR%\" >nul
  )
  echo [OK] Piper.exe copiado a runtime\piper\
) else (
  echo [WARN] No se encontró piper.exe tras la instalación.
  echo        Revisa los mensajes de pip y copia el binario manualmente.
)

exit /b 0

:ensure_voice
if not exist "%VOICES_DIR%" mkdir "%VOICES_DIR%"
dir /b "%VOICES_DIR%\*.onnx" >nul 2>nul
if errorlevel 1 (
  if exist "%BUNDLED_VOICE%" (
    echo [INFO] Copiando la voz de ejemplo incluida...
    copy /Y "%BUNDLED_VOICE%" "%VOICES_DIR%\" >nul
    if exist "%BUNDLED_META%" copy /Y "%BUNDLED_META%" "%VOICES_DIR%\" >nul
    echo [OK] Voz base disponible: es_ES-carlfm-x_low.onnx
    set "HAS_VOICES=1"
  ) else (
    echo [WARN] No hay voces instaladas.
    echo        Descarga un modelo es_ES de 22.05 kHz y colócalo en assets\voices\es_ES\
  )
) else (
  echo [OK] Se detectaron voces existentes.
  set "HAS_VOICES=1"
)
exit /b 0

:offer_demo
if not exist "%RUN_DEMO%" (
  echo [INFO] Script run_piper_demo.bat no encontrado. Omite la prueba automática.
  exit /b 0
)

echo ¿Te gustaría generar un audio de prueba ahora? [S/n]
set /p "DEMO_CHOICE=> "
if /i "!DEMO_CHOICE!"=="n" (
  echo [INFO] Puedes ejecutar run_piper_demo.bat más tarde manualmente.
  exit /b 0
)

echo.
echo Escribe el texto a locutar (ENTER para usar el texto por defecto):
set /p "DEMO_TEXT=Texto: "
if "!DEMO_TEXT!"=="" set "DEMO_TEXT=Hola, soy Reader y ya estoy listo."

echo.
echo [INFO] Llamando a run_piper_demo.bat...
call "%RUN_DEMO%" "!DEMO_TEXT!"
if errorlevel 1 (
  echo [WARN] Algo falló al ejecutar Piper. Revisa los mensajes anteriores.
) else (
  echo [OK] Audio generado en runtime\out.wav
)
exit /b 0
