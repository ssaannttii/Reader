# Guía de empaquetado Codex

## Cómo probar UI

1. Ejecuta `npm install` dentro de `ui/` y luego `npm run dev`.
2. En otra terminal, lanza `cargo tauri dev` para abrir la ventana de escritorio.
3. Navega por las pestañas Biblioteca, Lector y Diccionario asegurándote de que los
   temas claro/oscuro y los controles respondan correctamente.
4. Ajusta los deslizadores y confirma que el siguiente párrafo sintetizado respeta
   los nuevos valores.

## Flujos de importación

1. Ejecuta `python scripts/py/generate_samples.py` y luego usa la pestaña
   **Biblioteca** para importar `samples/sample.pdf` o `samples/sample.epub`.
2. Verifica que la vista previa muestre texto y que el botón “Añadir al Lector”
   habilite el contenido en la pestaña **Lector**.
3. Reproduce al menos dos párrafos consecutivos; confirma que el resaltado avanza.
4. Exporta el resultado desde el menú correspondiente (WAV y MP3).
