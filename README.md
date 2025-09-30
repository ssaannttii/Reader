# Reader

Plan maestro para un lector TTS local, optimizado para Windows, usando
tecnologías libres y de alto rendimiento.

## 1. Motor de voz (Piper)
- Descargar `piper.exe` y voces españolas (22.05 kHz) y colocarlas en
  `assets/voices/es_ES/`.
- Probar vía CLI: `piper.exe -m assets/voices/es_ES/<voz>.onnx --sentence-break "500" -f out.wav`.
- Validar latencia, calidad y afinación de parámetros (velocidad, tono,
  pausas).

## 2. App de escritorio (Tauri 2 + Rust)
- Crear el proyecto Tauri 2 con un frontend ligero (React/Vite o Svelte).
- Implementar comando `speak` que invoque Piper como subproceso y
  reproduzca/guarde el WAV generado.
- Diseñar la pantalla de lectura con textarea, controles de reproducción
  (▶️⏸️⏭️), sliders (velocidad/tono/pausas) y selector de voz.
- Implementar cola de lectura por párrafos para soportar textos largos.

## 3. Importadores (EPUB/PDF)
- Añadir scripts Python en `scripts/py/` que utilicen EbookLib y PyMuPDF.
- Desde el backend Rust, invocar los scripts como procesos cortos y
  recibir JSON con la estructura limpia de capítulos/párrafos.

## 4. Diccionario de pronunciación
- Implementar un módulo (`src-tauri/src/dict`) que gestione un JSON con
  palabras → fonemas.
- Permitir edición desde la UI y aplicar las reglas antes de llamar a
  Piper (eSpeak-NG como referencia fonética).

## 5. Build e instalador
- Configurar `tauri.conf.json` para empaquetar Piper y las voces.
- Generar instaladores `.msi/.exe` para Windows usando el bundler de
  Tauri.

## Estructura de carpetas
```
reader/
  src-tauri/
    src/
      cmds/           # Comandos Tauri (speak, importadores)
      audio/          # Reproducción de WAV/PCM
      ssml/           # Segmentación y anotaciones
      dict/           # Diccionario fonético
    tauri.conf.json
  ui/                 # Frontend (React/Svelte)
  assets/voices/      # Voces Piper es_ES
  scripts/py/         # Extracción EPUB/PDF
```

## Roadmap futuro
- MVP: texto pegado + voz es_ES + controles + exportar WAV/MP3.
- Integrar importadores EPUB/PDF, marcadores y modo noche.
- Añadir soporte SSML y mejoras de respiración/pausas.
- Gestionar biblioteca con portadas, progreso y perfiles de voz.
