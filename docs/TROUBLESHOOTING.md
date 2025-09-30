# Solución de problemas

## Python no está disponible

- Verifica que Python 3.9+ esté instalado para el usuario actual.
- Asegúrate de marcar la casilla “Add python.exe to PATH” durante la instalación.
- Si usas Microsoft Store, ejecuta `py -3 --version` y ajusta el script `dev_check.ps1`
  para usar `py -3` en lugar de `python`.

## Pip instala paquetes en otra carpeta

- Comprueba `%USERPROFILE%\AppData\Roaming\Python\Python311\Scripts` y añádelo a PATH.
- Ejecuta `python -m site --user-base` para conocer la ruta exacta y añade `Scripts/`.

## Voces no encontradas

- Ejecuta `scripts\windows\fetch_voices.ps1` para descargar de nuevo las voces.
- Confirma que `assets\voices\es_ES\*.onnx` y sus archivos `.onnx.json` existan.
- Ajusta los permisos de Windows Defender si bloquea la descarga.

## Piper no arranca

- Coloca `piper.exe` en `runtime\piper\` o instala `piper-tts` y usa `python -m piper`.
- Comprueba que el antivirus no haya bloqueado el ejecutable.

## ffmpeg ausente

- Instala FFmpeg desde <https://ffmpeg.org/download.html> y añade `ffmpeg.exe` a PATH.
- Si no está disponible, la exportación MP3 mostrará un mensaje y mantendrá el WAV.

## UI no muestra voces

- En producción, las voces se empaquetan como recursos. Durante desarrollo asegúrate de
  que la ruta `assets/voices/es_ES` exista respecto al directorio del ejecutable.
- Reinicia la app tras añadir nuevas voces para refrescar la lista.

## Archivos de muestra faltantes

- Los binarios `samples/sample.pdf` y `samples/sample.epub` no se incluyen en el repositorio.
- Ejecute `python scripts/py/generate_samples.py` para crearlos localmente antes de probar los importadores.
