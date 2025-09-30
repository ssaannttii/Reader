#!/usr/bin/env python3
"""Extrae capítulos y párrafos de un EPUB en JSON."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import List

try:
    from ebooklib import epub  # type: ignore
except ImportError as exc:  # pragma: no cover
    raise SystemExit("EbookLib es requerido: pip install ebooklib") from exc

try:
    from bs4 import BeautifulSoup  # type: ignore
except ImportError as exc:  # pragma: no cover
    raise SystemExit("BeautifulSoup4 es requerido: pip install beautifulsoup4") from exc

CLEAN_RE = re.compile(r"\s+")


def clean_paragraph(text: str) -> str:
    text = CLEAN_RE.sub(" ", text).strip()
    return text


def extract_chapter(item: epub.EpubHtml) -> dict:
    soup = BeautifulSoup(item.get_body_content(), "html.parser")
    for tag in soup.select("script, style, footnote, sup"):
        tag.decompose()

    title = soup.title.string.strip() if soup.title and soup.title.string else item.get_name()
    paragraphs: List[str] = []
    for element in soup.find_all(["p", "div"]):
        text = element.get_text(separator=" ")
        cleaned = clean_paragraph(text)
        if cleaned:
            paragraphs.append(cleaned)

    return {"title": title, "paragraphs": paragraphs}


def extract(epub_path: Path) -> dict:
    try:
        book = epub.read_epub(str(epub_path))
    except Exception as exc:  # pragma: no cover
        raise SystemExit(json.dumps({"ok": False, "code": "EPUB_OPEN_FAIL", "message": str(exc)}))

    chapters = []
    for item in book.get_items_of_type(epub.ITEM_DOCUMENT):
        chapters.append(extract_chapter(item))

    return {"ok": True, "chapters": chapters}


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("Uso: epub_extract.py <ruta_epub>")

    epub_path = Path(sys.argv[1])
    if not epub_path.exists():
        raise SystemExit(json.dumps({"ok": False, "code": "EPUB_NOT_FOUND", "message": str(epub_path)}))

    try:
        payload = extract(epub_path)
    except SystemExit:
        raise
    except Exception as exc:  # pragma: no cover
        raise SystemExit(json.dumps({"ok": False, "code": "EPUB_PARSE_FAIL", "message": str(exc)}))

    print(json.dumps(payload, ensure_ascii=False))


if __name__ == "__main__":
    main()
