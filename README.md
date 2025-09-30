# Reader

Plan maestro para un lector TTS local, optimizado para Windows, usando
tecnologías libres y de alto rendimiento.

## Empezar en Windows (CLI)

1. **Clona el repositorio**
   ```powershell
   git clone https://github.com/<tu-usuario>/Reader.git
   cd Reader
   ```
2. **Obtén Piper para Windows**
   - La última release de <https://github.com/OHF-Voice/piper1-gpl/releases/latest>
     (verificada manualmente) no incluye binarios listos para Windows.
   - Instala la rueda oficial con `python -m pip install --upgrade piper-tts` y
     copia `piper.exe` desde tu entorno de Python (normalmente
     `%USERPROFILE%\AppData\Local\Programs\Python\Python311\Scripts\piper.exe`
     o `venv\Scripts\piper.exe`) a `runtime\piper\` dentro del repo (crea la
     carpeta si hace falta).
3. **Descarga al menos una voz española**
   - En la misma release, busca un modelo de 22.05 kHz (calidad *high*), por
     ejemplo `es_ES-aisa-high.onnx` o `es_ES-carlfm-high.onnx`.
   - Copia el `.onnx` a `assets\voices\es_ES\` (crea la carpeta si no existe).
4. **Ejecuta la demo por lotes**
   ```bat
   scripts\windows\run_piper_demo.bat "Hola, esto es una prueba."
   ```
   El resultado se guardará en `runtime\out.wav` y podrás abrirlo con el
   reproductor predeterminado.

> ℹ️  Para más opciones (usar archivos de texto, cambiar el modelo con
> `PIPER_VOICE`, etc.) consulta `scripts/windows/README.md`.

## Roadmap funcional

### 1. Motor de voz (Piper)
- Descargar `piper.exe` y voces españolas (22.05 kHz) y colocarlas en
  `assets/voices/es_ES/`.
- Probar vía CLI: `piper.exe -m assets/voices/es_ES/<voz>.onnx --sentence-break "500" -f out.wav`.
- Validar latencia, calidad y afinación de parámetros (velocidad, tono,
  pausas).

### 2. App de escritorio (Tauri 2 + Rust)
- Crear el proyecto Tauri 2 con un frontend ligero (React/Vite o Svelte).
- Implementar comando `speak` que invoque Piper como subproceso y
  reproduzca/guarde el WAV generado.
- Diseñar la pantalla de lectura con textarea, controles de reproducción
  (▶️⏸️⏭️), sliders (velocidad/tono/pausas) y selector de voz.
- Implementar cola de lectura por párrafos para soportar textos largos.

### 3. Importadores (EPUB/PDF)
- Añadir scripts Python en `scripts/py/` que utilicen EbookLib y PyMuPDF.
- Desde el backend Rust, invocar los scripts como procesos cortos y
  recibir JSON con la estructura limpia de capítulos/párrafos.

### 4. Diccionario de pronunciación
- Implementar un módulo (`src-tauri/src/dict`) que gestione un JSON con
  palabras → fonemas.
- Permitir edición desde la UI y aplicar las reglas antes de llamar a
  Piper (eSpeak-NG como referencia fonética).

### 5. Build e instalador
- Configurar `tauri.conf.json` para empaquetar Piper y las voces.
- Generar instaladores `.msi/.exe` para Windows usando el bundler de
  Tauri.

## Requisitos para la futura app Tauri

Cuando quieras compilar la interfaz de escritorio, asegúrate de tener:

- [Rust](https://www.rust-lang.org/tools/install) (toolchain estable).
- [Node.js 18+](https://nodejs.org/) y un gestor de paquetes (npm, pnpm o yarn).
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
  con los componentes de C++ y Windows 10/11 SDK.
- Python 3.10+ (para los scripts de importación EPUB/PDF).

Con las dependencias listas:

```powershell
# Instala dependencias del frontend (cuando exista la UI)
cd ui
npm install

# Levanta la app en modo desarrollo
cd ..
cargo tauri dev
```

Mientras el frontend está en desarrollo, puedes seguir usando Piper mediante
el script por lotes o directamente vía CLI.

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
  scripts/windows/    # Utilidades para Windows (Piper demo)
```

## Roadmap futuro
- MVP: texto pegado + voz es_ES + controles + exportar WAV/MP3.
- Integrar importadores EPUB/PDF, marcadores y modo noche.
- Añadir soporte SSML y mejoras de respiración/pausas.
- Gestionar biblioteca con portadas, progreso y perfiles de voz.
