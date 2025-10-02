# Estado actual del proyecto

Este inventario complementa la hoja de ruta del repositorio y deja constancia
de qué piezas siguen pendientes para cumplir con el MVP descrito en
`docs/CODEX_PACKAGE.md`.

## Backend Tauri (Rust)
- `src-tauri/src/main.rs` no existe todavía, por lo que no hay punto de entrada
  para registrar comandos ni inicializar la aplicación.
- Los módulos `cmds`, `audio`, `dict` y `ssml` únicamente contienen comentarios
  de marcador de posición sin funciones exportadas ni lógica que pueda invocar
  Piper desde Rust.

## Frontend (`ui/`)
- No se ha inicializado un proyecto (faltan `package.json`, fuentes React/Svelte,
  Vite, etc.).
- No hay implementación de la cola de lectura, controles de reproducción,
  selector de voz, importadores de documentos ni exportación a WAV/MP3 que se
  describen en la documentación.

## Scripts auxiliares
- La carpeta `scripts/py/` continúa vacía; no existen scripts de extracción para
  EPUB o PDF ni utilidades de limpieza de texto.
- Tampoco hay automatizaciones de QA (tests end-to-end, linters o pipelines)
  asociadas a esas herramientas.

## Conclusión
El repositorio actual sirve como base documental y ofrece una demo CLI de Piper,
pero aún no cumple los requisitos funcionales del MVP de escritorio. Estas
áreas deben desarrollarse antes de considerar el proyecto listo para entrega.
