#!/usr/bin/env python3
"""PDF importer for Reader.

Reads a PDF file using PyMuPDF (fitz) and emits the JSON structure expected by the
Rust `ImportResponse` type. Each page is converted into a section with its text
content. Basic document metadata is preserved when available.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

try:
    import fitz  # PyMuPDF
except ImportError:  # pragma: no cover - handled at runtime
    sys.stderr.write("PyMuPDF (fitz) is required to import PDFs.\n")
    sys.exit(1)


def page_to_section(page: "fitz.Page", index: int) -> Optional[Dict[str, Any]]:
    text = page.get_text("text").strip()
    if not text:
        return None
    return {
        "id": str(index + 1),
        "heading": None,
        "content": text,
    }


def normalise_metadata(metadata: Dict[str, Any]) -> Dict[str, Any]:
    result: Dict[str, Any] = {}
    for key, value in metadata.items():
        if isinstance(value, (str, int, float, bool)) or value is None:
            result[key] = value
    return result


def main(argv: List[str]) -> int:
    if len(argv) != 2:
        sys.stderr.write("Usage: import_pdf.py <path-to-pdf>\n")
        return 1

    path = Path(argv[1])
    if not path.exists():
        sys.stderr.write(f"File not found: {path}\n")
        return 1

    try:
        document = fitz.open(path)
    except Exception as exc:  # pragma: no cover - depends on fitz internals
        sys.stderr.write(f"Unable to open PDF: {exc}\n")
        return 1

    sections: List[Dict[str, Any]] = []
    for index, page in enumerate(document):
        section = page_to_section(page, index)
        if section:
            sections.append(section)

    warnings: List[str] = []
    if not sections:
        warnings.append("No se encontró texto en el PDF")

    metadata = normalise_metadata(document.metadata or {})
    title = metadata.get("title")
    language = metadata.get("language") or metadata.get("lang")

    payload: Dict[str, Any] = {
        "title": title if title else None,
        "language": language if language else None,
        "sections": sections,
        "metadata": metadata,
        "warnings": warnings,
    }

    json.dump(payload, sys.stdout, ensure_ascii=False)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main(sys.argv))
