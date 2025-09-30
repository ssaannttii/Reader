# Codex Implementation Package

## 1. Resumen de cambios realizados (estado actual)

**Repositorios y decisión de motor**

- Adoptado motor **Piper 1 GPL** (proyecto: `OHF-Voice/piper1-gpl`) con instalación vía `pip install piper-tts`.
- Se descarta depender de binarios `.exe` en releases y se documenta el uso de **módulo Python (`python -m piper`)** y, opcionalmente, copia local de `piper.exe` desde `…\PythonXX\Scripts`.

**Estructura del repo**

```
Reader/
  runtime/
    piper/                 # (opcional) copia local de piper.exe
  assets/
    voices/
      es_ES/               # modelos ONNX + JSON (pareja)
  scripts/
    windows/
      run_piper_demo.bat   # demo CLI Windows
    py/                    # futuros extractores PDF/EPUB
  src-tauri/
    src/
      cmds/                # comandos Tauri (por implementar)
      audio/               # reproducción WAV (por implementar)
      ssml/                # segmentación/SSML (por implementar)
      dict/                # diccionario fonético (por implementar)
    tauri.conf.json        # empaquetado/bundler (por preparar)
  ui/                      # frontend (React/Svelte) (por implementar)
  README.md                # actualizado a Windows-first + Piper 1 GPL
```

**README.md actualizado**

- Se explica instalación **vía `pip`**: `python -m pip install --upgrade pip piper-tts`.
- Se documenta cómo localizar `Scripts` y, si se quiere, **copiar `piper.exe` a `runtime\piper\`**.
- Se corrige el origen de voces: **Hugging Face (`rhasspy/piper-voices`)** o **descarga automática** con `python -m piper.download_voices`.
- Se incluye **línea de comandos Windows** de prueba con `python -m piper -m <voz>.onnx -f runtime\out.wav`.
- Se añade nota de **licencia GPLv3** (si se redistribuye el binario con la app, implicaciones GPL).

**Voces y prueba de audio**

- Descargada y probada una voz **es_ES** (ej. `es_ES-carlfm-x_low`), asegurando que el **par** `*.onnx` + `*.onnx.json` esté en `assets\voices\es_ES\`.
- Verificada **síntesis local**:

  ```
  echo Hola... | python -m piper -m assets\voices\es_ES\es_ES-carlfm-x_low.onnx -f runtime\out.wav
  start "" runtime\out.wav
  ```
- Resultado: **funciona** (WAV generado y reproducido correctamente).

**Correcciones de entorno**

- Detectado que Python 3.9 estaba instalando paquetes en **perfil de usuario** (`%APPDATA%\Python\Python39\site-packages`) y no en `Program Files`.
- Documentados comandos para hallar `Scripts` correctos y **no depender del launcher `py`** (usamos `python`).

---

## 2. Objetivo para Codex (alto nivel)

> Entregar un **MVP de escritorio en Windows** (Tauri 2 + Rust + UI web) que:
>
> 1. Lea texto pegado y **sintetice** con Piper (local).
> 2. **Importe PDF/EPUB/TXT**, limpie el texto y lo presente por **párrafos/capítulos**.
> 3. Tenga **UI profesional** con controles (▶/⏸/⏭, velocidad/tono/pausas, selector de voz).
> 4. **Exporte a WAV/MP3** y gestione una **cola de lectura**.
> 5. 100% offline (sin llamadas de red).

---

## 3. Tareas para Codex — detalladas y accionables

### Tarea A — Comando `speak` (Tauri ↔ Piper)

**Archivos:**

- `src-tauri/src/cmds/speak.rs` (nuevo)
- `src-tauri/src/main.rs` (registrar comando)
- `ui/` (hook/servicio TS para `invoke`)

**Qué hacer:**

1. Implementar un comando Tauri `speak` que reciba:

   ```ts
   {
     text: string,
     voicePath: string,           // ruta .onnx
     sentenceBreak?: number,      // ms, default 550
     lengthScale?: number,        // 0.9..1.2, default 1.0
     noiseScale?: number,         // 0.3..0.7, default 0.5
     noiseW?: number,             // 0.8..1.2, default 0.9
     outPath?: string,            // default: runtime/out.wav
     playAfter?: boolean          // si true, reproducir al terminar
   }
   ```
2. Invocar Piper como **subproceso** (dos variantes, autodetectar o configurable):

   - **A)** `python -m piper` (no dependemos del exe).
   - **B)** `runtime\piper\piper.exe` si existe.
3. Pasar `text` por **stdin** a Piper.
4. Escribir WAV en `outPath`.
5. Si `playAfter`, reproducir con módulo de audio (ver Tarea C).
6. Devolver JSON con `{ ok: true, outPath, elapsedMs, stderr? }`.

**Criterios de aceptación:**

- Soporta caracteres UTF-8 (tildes/ñ).
- Gestión de error clara si la voz no existe o falta el `.json`.
- Pruebas con `assert_cmd`:

  - Texto corto, texto multi-párrafo.
  - Ruta de voz válida/ inválida.
  - Flags por defecto y custom.
- Tiempo de síntesis medido y retornado.

---

### Tarea B — UI “Lector” (Windows-first)

**Archivos:**

- `ui/src/pages/Reader.tsx` (nuevo)
- `ui/src/components/Controls.tsx`, `ui/src/state/playerStore.ts` (zustand o similar)

**Qué hacer:**

1. Pantalla “Lector” con:

   - **Textarea/Editor** (o visor de párrafos si ya importamos PDF/EPUB).
   - Controles: ▶/⏸/⏭ (siguiente párrafo), sliders para `velocidad (lengthScale)`, `pausas (sentenceBreak)`, `tono (noiseScale/noiseW)`.
   - Selector de **voz** (listar `assets/voices/es_ES/*.onnx`).
   - **Cola**: dividir el texto en párrafos (separador doble salto de línea) y mantener índice del actual.
2. Integración con el comando `speak` (Tarea A).
3. Vistas: claro/oscuro, tipografía legible, accesibilidad de fuente.

**Criterios de aceptación:**

- Reproduce un párrafo, resalta el párrafo actual, permite avanzar/retroceder.
- Cambios de sliders afectan a la síntesis siguiente.
- Selector de voz persiste en localStorage/config.

---

### Tarea C — Audio local (reproducir WAV)

**Archivos:**

- `src-tauri/src/audio/player.rs` (nuevo)

**Qué hacer:**

- Usar `rodio` o `cpal` para reproducir `out.wav`.
- API Rust expuesta a Tauri:

  - `play(path)`, `stop()`, `is_playing() -> bool`.
- Evitar bloqueo de UI (spawn thread).

**Criterios de aceptación:**

- Reproduce el WAV generado por `speak`.
- Maneja `stop` durante reproducción.
- Devuelve errores útiles si el WAV no existe/corrupto.

---

### Tarea D — Importador **PDF**

**Archivos:**

- `scripts/py/pdf_extract.py` (nuevo)
- `src-tauri/src/cmds/import_pdf.rs` (nuevo)

**Qué hacer:**

1. `pdf_extract.py` con **PyMuPDF** que reciba ruta de PDF y devuelva JSON:

   ```json
   { "pages": [ {"text": "..."} ], "meta": {"title":"...", "author":"..."} }
   ```

   Limpieza mínima: eliminar guiones de fin de línea, normalizar espacios.
2. `import_pdf.rs`: invocar el script como **proceso corto** y parsear JSON.
3. UI: en “Biblioteca”, **drag & drop** o file picker y enviar a “Lector”.

**Criterios de aceptación:**

- PDF multi-página extraído sin cortar palabras.
- Archivos grandes (p. ej. >50 MB) procesan sin bloquear UI (mostrar spinner).
- Errores claros si el PDF está protegido o ilegible.

---

### Tarea E — Importador **EPUB**

**Archivos:**

- `scripts/py/epub_extract.py` (nuevo)
- `src-tauri/src/cmds/import_epub.rs` (nuevo)

**Qué hacer:**

1. `epub_extract.py` con **EbookLib**:

   - Salida:

     ```json
     { "chapters": [ { "title": "…", "paragraphs": ["…","…"] } ] }
     ```
   - Limpieza de HTML (quitar notas al pie, estilos, etc.).
2. Comando Tauri `import_epub` que invoque el script y devuelva JSON.
3. UI: vista de capítulos y botón “Añadir a cola”.

**Criterios de aceptación:**

- Preserva orden lógico de capítulos.
- Sin etiquetas HTML remanentes.
- EPUB con imágenes se ignora (solo texto) sin error.

---

## 4. Requisitos y convenciones

- **Windows-only** por ahora (Linux/macOS en futuro).
- **Sin red**: la app no realiza llamadas HTTP. Todo local.
- **Logs** a fichero rotativo `logs/reader.log`.
- **Errores**: todos los comandos Tauri devuelven `{ok:false, code, message}` en fallos.
- **Tests**: unit (Rust) + smoke tests (scripts PowerShell).
- **UI**: React + Tailwind; estado con zustand; ruta `ui/`.
- **Accesibilidad**: tema claro/oscuro, fuente ajustable, contraste AA.

---

## 5. Entregables y verificación (para Codex)

- PR “feat: Windows MVP — TTS + UI + importadores” con:

  - Código de Tareas A–E integrado.
  - Guía en `README.md` sección “Windows Quickstart”:

    - `npm install` (ui), `cargo tauri dev`, prueba de PDF/EPUB.
  - `scripts/windows/dev_check.ps1` que:

    - Instala deps (si faltan),
    - Ejecuta una demo TTS,
    - Importa un PDF/EPUB de prueba (incluye sample en `samples/`).
- Video/GIF corto (puede ser automatizado) mostrando:

  - Importar un PDF,
  - Leer 2–3 párrafos,
  - Cambiar voz y re-leer,
  - Exportar WAV.

---

## 6. Mensaje para Codex (listo para pegar)

> **Contexto**: Proyecto `Reader` (Windows-first). Motor TTS local **Piper 1 GPL** (`pip install piper-tts`). Voces ONNX+JSON en `assets/voices/es_ES`. Ya podemos sintetizar vía `python -m piper`. Queremos una **UI de escritorio Tauri** con lectura de **PDF/EPUB**, controles y exportación.
>
> **Objetivo**: Implementa Tareas A–E (speak, UI lector, audio player, importador PDF, importador EPUB) con los criterios de aceptación descritos arriba.
>
> **Restricciones**: 100% offline; nada de llamadas de red. Manejo de errores consistente. Logs en `logs/reader.log`.
>
> **QA**: añade tests y `scripts/windows/dev_check.ps1`.
>
> **Entrega**: PR único con instrucciones actualizadas en `README.md` y un sample PDF/EPUB en `samples/`.

---

¿Necesitas también que se genere el `dev_check.ps1` y los stubs de los archivos clave? Se pueden preparar para acelerar el desarrollo.
