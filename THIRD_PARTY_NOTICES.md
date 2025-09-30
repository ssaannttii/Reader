# Aviso de terceros

Este proyecto utiliza o integra los siguientes componentes de terceros:

- **Piper 1 (GPLv3)** – motor de síntesis de voz. Código fuente y binarios disponibles en
  <https://github.com/OH-Human-Voice/piper>. Se distribuye bajo la licencia GPLv3.
- **Modelos de voz Piper (ONNX)** – cada voz incluye su propia licencia. Consulta los archivos
  `.onnx.json` dentro de `assets/voices/` para obtener los detalles específicos.
- **PyMuPDF** – Licencia AGPL/Commercial. Úsalo conforme a sus términos cuando ejecutes
  `scripts/py/pdf_extract.py`.
- **EbookLib** – Licencia GPLv3. Utilizada en `scripts/py/epub_extract.py`.
- **BeautifulSoup4** – Licencia MIT. Dependencia auxiliar para limpiar EPUB.

Asegúrate de revisar las licencias completas antes de redistribuir el software.
