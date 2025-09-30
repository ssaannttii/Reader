@echo off
setlocal enabledelayedexpansion

rem Wrapper para tener el demo de Piper a mano desde la raíz del repo.
rem Delegamos en scripts\windows\run_piper_demo.bat para no duplicar lógica.

set "REPO_ROOT=%~dp0"
pushd "%REPO_ROOT%" >nul

set "DEMO_SCRIPT=scripts\windows\run_piper_demo.bat"
if not exist "%DEMO_SCRIPT%" (
    echo [ERROR] No se encontró %%DEMO_SCRIPT%%. Asegúrate de clonar el repo completo.
    popd >nul
    exit /b 1
)

call "%DEMO_SCRIPT%" %*
set "ERRORLEVEL_COPY=%ERRORLEVEL%"

popd >nul
exit /b %ERRORLEVEL_COPY%
