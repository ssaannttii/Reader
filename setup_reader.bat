@echo off
setlocal

set "REPO_ROOT=%~dp0"
pushd "%REPO_ROOT%" >nul

set "SETUP_SCRIPT=scripts\windows\setup_reader.bat"
if not exist "%SETUP_SCRIPT%" (
    echo [ERROR] No se encontró %%SETUP_SCRIPT%%. Asegúrate de clonar el repo completo.
    popd >nul
    exit /b 1
)

call "%SETUP_SCRIPT%" %*
set "ERRORLEVEL_COPY=%ERRORLEVEL%"

popd >nul
exit /b %ERRORLEVEL_COPY%
