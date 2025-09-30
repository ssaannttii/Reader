#!/usr/bin/env python3
"""Extrae texto de un PDF y lo normaliza para el backend de Reader."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

try:
    import fitz  # type: ignore
except ImportError as exc:  # pragma: no cover - se evalúa en Windows
    raise SystemExit("PyMuPDF (fitz) es requerido: pip install pymupdf") from exc

LINE_HYPHEN = re.compile(r"-\n")
MULTISPACE = re.compile(r"\s+", re.MULTILINE)


def clean_text(raw: str) -> str:
    text = LINE_HYPHEN.sub("", raw)
    text = text.replace("\r", "\n")
    paragraphs = [
        MULTISPACE.sub(" ", block).strip()
        for block in text.split("\n\n")
        if block.strip()
    ]
    return "\n\n".join(paragraphs)


def extract(pdf_path: Path) -> dict:
    try:
        document = fitz.open(pdf_path)
    except Exception as exc:  # pragma: no cover
        raise SystemExit(json.dumps({"ok": False, "code": "PDF_OPEN_FAIL", "message": str(exc)}))

    pages = []
    meta = document.metadata or {}
    for page in document:
        text = page.get_text("text")
        pages.append({"text": clean_text(text)})

    payload = {
        "ok": True,
        "pages": pages,
        "meta": {
            "title": meta.get("title") or pdf_path.stem,
            "author": meta.get("author") or "",
        },
    }
    return payload


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("Uso: pdf_extract.py <ruta_pdf>")

    pdf_path = Path(sys.argv[1])
    if not pdf_path.exists():
        raise SystemExit(json.dumps({"ok": False, "code": "PDF_NOT_FOUND", "message": str(pdf_path)}))

    try:
        payload = extract(pdf_path)
    except SystemExit as exc:
        raise
    except Exception as exc:  # pragma: no cover
        raise SystemExit(json.dumps({"ok": False, "code": "PDF_PARSE_FAIL", "message": str(exc)}))

    print(json.dumps(payload, ensure_ascii=False))


if __name__ == "__main__":
    main()
