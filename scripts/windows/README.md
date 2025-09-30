# Scripts para Windows

Estos scripts facilitan la puesta en marcha rápida del MVP basado en Piper.

## `run_piper_demo.bat`

Script por lotes que invoca `piper.exe` con la voz española instalada en
`assets/voices/es_ES/`. El script detecta automáticamente el primer modelo
`.onnx` disponible y permite sobreescribirlo con la variable `PIPER_VOICE`.
Puedes pasar el texto directamente como argumento o leerlo desde un archivo
usando `-f`.

### Requisitos previos

1. **Descargar Piper**
   - Ve a <https://github.com/rhasspy/piper/releases/latest> y baja el ZIP
     `piper_windows_amd64`.
   - Extrae `piper.exe` en `runtime/piper/` dentro del proyecto.
2. **Descargar una voz es_ES (22.05 kHz, calidad high)**
   - En la misma página de releases, ubica una voz como
     `es_ES-carlfm-high.onnx` o `es_ES-aisa-high.onnx`.
   - Coloca el archivo `.onnx` en `assets/voices/es_ES/`.

### Ejemplos de uso

```bat
:: Texto en línea
scripts\windows\run_piper_demo.bat "Hola, esto es una prueba."

:: Texto desde archivo
scripts\windows\run_piper_demo.bat -f textos\capitulo1.txt

:: Forzar otra voz (modelo .onnx)
set PIPER_VOICE=assets\voices\es_ES\es_ES-aisa-high.onnx
scripts\windows\run_piper_demo.bat "Probando otra voz"
```

El audio se guardará en `runtime/out.wav`. Puedes reproducirlo con cualquier
player, por ejemplo, doble clic en el Explorador o usando `start runtime\out.wav`.
