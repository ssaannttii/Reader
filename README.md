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
   run_piper_demo.bat "Hola, esto es una prueba."
   ```
   El resultado se guardará en `runtime\out.wav` y podrás abrirlo con el
   reproductor predeterminado. Si prefieres mantener los scripts agrupados,
   sigue disponible `scripts\windows\run_piper_demo.bat` (el nuevo `run_piper_demo.bat`
   simplemente delega en él para que la UI y futuros flujos de trabajo no se
   vean afectados).

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

## Aplicación de escritorio (Tauri)

El MVP incluido en este repositorio permite gestionar la cola de lectura,
escoger voces y sintetizar párrafos directamente desde la UI.

### Dependencias

1. Instala las herramientas descritas en la sección "Requisitos para la futura
   app Tauri" (Rust estable, Node.js 18+, pnpm o npm, Visual Studio Build Tools
   en Windows y Python 3.10+ para los importadores).
2. Instala las dependencias del frontend:

   ```bash
   cd ui
   pnpm install
   ```

3. Vuelve a la raíz del proyecto y ejecuta el modo desarrollo de Tauri:

   ```bash
   pnpm tauri dev
   ```

   El comando lanzará automáticamente el servidor de Vite del frontend y el
   proceso de Tauri. Desde la ventana podrás:

   - Importar archivos EPUB/PDF/TXT (se abrirá el selector de archivos del
     sistema).
   - Editar párrafos manualmente y añadirlos a la cola.
   - Elegir la voz Piper disponible, ajustar velocidad, tono y volumen.
   - Reproducir, pausar, avanzar al siguiente párrafo y exportar el último WAV
     generado.

> ℹ️  El backend emite el evento `reader://playback-ended` cada vez que Piper
> termina de sintetizar un párrafo. La interfaz lo escucha para avanzar en la
> cola automáticamente.

### Gestión de voces

El comando `list_voices` escanea `assets/voices/` (o la carpeta definida en
`READER_VOICES_DIR`) en busca de modelos `.onnx`. Para cada voz, intenta cargar
el archivo `.onnx.json` asociado para mostrar el idioma y la calidad. Asegúrate
de conservar la pareja `modelo.onnx` + `modelo.onnx.json` en el mismo directorio.

### Diccionario de pronunciación

El backend crea (si no existe) `runtime/dictionary.json`. Puedes editar ese
archivo manualmente con entradas como:

```json
[
  { "word": "AI", "replacement": "ei" }
]
```

Cada palabra se reemplaza de forma insensible a mayúsculas antes de generar el
SSML enviado a Piper.

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
