#!/usr/bin/env python3
"""EPUB importer for Reader.

Uses EbookLib to extract the textual content of an EPUB file and prints the JSON
structure expected by the Rust `ImportResponse` type. HTML is converted to plain
text using a simple HTML parser to avoid external dependencies.
"""

from __future__ import annotations

import json
import sys
from html import unescape
from html.parser import HTMLParser
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional

try:
    from ebooklib import epub
except ImportError:  # pragma: no cover - handled at runtime
    sys.stderr.write("EbookLib is required to import EPUB files.\n")
    sys.exit(1)


BLOCK_TAGS = {
    "p",
    "div",
    "section",
    "article",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "li",
    "ol",
    "ul",
    "blockquote",
    "br",
}


class TextExtractor(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.parts: List[str] = []

    def handle_starttag(self, tag: str, attrs: List[tuple[str, str]]) -> None:
        if tag in BLOCK_TAGS:
            self.parts.append("\n")

    def handle_endtag(self, tag: str) -> None:
        if tag in BLOCK_TAGS:
            self.parts.append("\n")

    def handle_data(self, data: str) -> None:
        if data:
            self.parts.append(unescape(data))

    def get_text(self) -> str:
        text = "".join(self.parts)
        lines = [line.strip() for line in text.splitlines()]
        filtered = [line for line in lines if line]
        return "\n".join(filtered)


def html_to_text(content: bytes) -> str:
    parser = TextExtractor()
    parser.feed(content.decode("utf-8", errors="ignore"))
    parser.close()
    return parser.get_text()


def normalise_metadata(book: "epub.EpubBook") -> Dict[str, Any]:
    metadata: Dict[str, Any] = {}
    for namespace, values in book.metadata.items():
        namespace_dict: Dict[str, Any] = {}
        for name, entries in values.items():
            namespace_dict[name] = [entry[0] for entry in entries if entry]
        metadata[namespace] = namespace_dict
    return metadata


def first_metadata(values: Iterable[str]) -> Optional[str]:
    for value in values:
        if value:
            return value
    return None


def main(argv: List[str]) -> int:
    if len(argv) != 2:
        sys.stderr.write("Usage: import_epub.py <path-to-epub>\n")
        return 1

    path = Path(argv[1])
    if not path.exists():
        sys.stderr.write(f"File not found: {path}\n")
        return 1

    try:
        book = epub.read_epub(path)
    except Exception as exc:  # pragma: no cover - depends on ebooklib internals
        sys.stderr.write(f"Unable to open EPUB: {exc}\n")
        return 1

    sections: List[Dict[str, Any]] = []
    for item in book.get_items_of_type(epub.ITEM_DOCUMENT):
        text = html_to_text(item.get_content())
        if not text:
            continue
        sections.append(
            {
                "id": item.get_name(),
                "heading": getattr(item, "title", None),
                "content": text,
            }
        )

    warnings: List[str] = []
    if not sections:
        warnings.append("No se encontró texto en el EPUB")

    title = first_metadata(value for value, _ in book.get_metadata("DC", "title"))
    language = first_metadata(value for value, _ in book.get_metadata("DC", "language"))

    payload: Dict[str, Any] = {
        "title": title,
        "language": language,
        "sections": sections,
        "metadata": normalise_metadata(book),
        "warnings": warnings,
    }

    json.dump(payload, sys.stdout, ensure_ascii=False)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main(sys.argv))
