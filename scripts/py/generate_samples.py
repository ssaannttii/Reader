#!/usr/bin/env python3
"""Create lightweight demo PDF/EPUB assets without committing binaries."""
from __future__ import annotations

from pathlib import Path

try:
    import fitz  # type: ignore
except ImportError as exc:  # pragma: no cover - evaluated on Windows
    raise SystemExit("PyMuPDF (fitz) is required: pip install pymupdf") from exc

try:
    from ebooklib import epub  # type: ignore
except ImportError as exc:  # pragma: no cover
    raise SystemExit("EbookLib is required: pip install ebooklib") from exc

ROOT = Path(__file__).resolve().parents[2]
SAMPLES_DIR = ROOT / "samples"

PDF_TEXT = """Reader

Este es un documento de ejemplo utilizado para las pruebas de importación.
Contiene varios párrafos con acentos, eñes y símbolos básicos.

Gracias por probar el MVP de Reader."""

EPUB_PARAGRAPHS = [
    "Bienvenido al capítulo de ejemplo.",
    "Este contenido sirve para validar la importación de EPUBs.",
    "La lectura funciona completamente sin conexión.",
]


def ensure_samples_dir() -> None:
    SAMPLES_DIR.mkdir(parents=True, exist_ok=True)


def create_pdf(path: Path) -> None:
    doc = fitz.open()
    page = doc.new_page(width=595, height=842)  # A4 points
    text_rect = fitz.Rect(72, 72, 595 - 72, 842 - 72)
    page.insert_textbox(text_rect, PDF_TEXT, fontsize=12, fontname="helv")
    doc.set_metadata({"title": "Muestra Reader", "author": "Reader"})
    doc.save(path)


def create_epub(path: Path) -> None:
    book = epub.EpubBook()
    book.set_identifier("reader-demo")
    book.set_title("Muestra Reader")
    book.set_language("es")

    intro = epub.EpubHtml(title="Introducción", file_name="intro.xhtml", lang="es")
    intro.content = """
        <h1>Introducción</h1>
        <p>Reader genera este EPUB ligero para pruebas.</p>
    """

    chapter = epub.EpubHtml(title="Capítulo 1", file_name="capitulo1.xhtml", lang="es")
    chapter.content = """
        <h1>Capítulo 1</h1>
        <p>{}</p>
        <p>{}</p>
        <p>{}</p>
    """.format(*EPUB_PARAGRAPHS)

    book.add_item(intro)
    book.add_item(chapter)

    book.toc = (intro, chapter)
    book.add_item(epub.EpubNcx())
    book.add_item(epub.EpubNav())

    book.spine = ["nav", intro, chapter]

    epub.write_epub(path, book)


def main() -> None:
    ensure_samples_dir()
    pdf_path = SAMPLES_DIR / "sample.pdf"
    epub_path = SAMPLES_DIR / "sample.epub"
    create_pdf(pdf_path)
    create_epub(epub_path)
    print(f"Created {pdf_path} and {epub_path}")


if __name__ == "__main__":
    main()
