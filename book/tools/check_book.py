#!/usr/bin/env python3
"""Check mdBook's rendered local links, fragments, assets, and chapter coverage.

Run `mdbook build book` first. Uses only Python's standard library; external
URLs are source citations and are deliberately not fetched by this check.
"""

from html.parser import HTMLParser
from pathlib import Path
import re
import sys
from urllib.parse import unquote, urlsplit


class Page(HTMLParser):
    def __init__(self, text):
        super().__init__(convert_charrefs=True)
        self.ids = set()
        self.links = []
        self.feed(text)

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if attrs.get("id"):
            self.ids.add(attrs["id"])
        for attr in ("href", "src"):
            if attrs.get(attr):
                self.links.append(attrs[attr])


def main():
    book = Path(__file__).resolve().parents[1]
    source = book / "src"
    output = book / "html"
    if not (output / "index.html").is_file():
        print("Missing book/html/index.html; run: mdbook build book", file=sys.stderr)
        return 1

    failures = []
    listed = {
        (source / match).resolve()
        for match in re.findall(r"\]\(([^)]+\.md)\)", (source / "SUMMARY.md").read_text())
    }
    chapters = {p.resolve() for p in source.rglob("*.md") if p.name != "SUMMARY.md"}
    for missing in sorted(listed - chapters):
        failures.append(f"SUMMARY references missing chapter: {missing}")
    for orphan in sorted(chapters - listed):
        failures.append(f"Chapter absent from SUMMARY: {orphan}")
    for chapter in sorted(chapters & listed):
        html = output / chapter.relative_to(source).with_suffix(".html")
        if chapter.name == "README.md":
            html = html.with_name("index.html")
        if not html.is_file():
            failures.append(f"Chapter not rendered: {chapter}")

    pages = {p.resolve(): Page(p.read_text()) for p in output.rglob("*.html")}
    checked = 0
    for path, page in pages.items():
        for link in page.links:
            url = urlsplit(link)
            if url.scheme or url.netloc:
                continue
            target_path = unquote(url.path)
            target = (
                output / target_path.lstrip("/")
                if target_path.startswith("/")
                else path.parent / target_path if target_path else path
            ).resolve()
            if target.is_dir():
                target /= "index.html"
            checked += 1
            if not target.is_file() or not target.is_relative_to(output.resolve()):
                failures.append(f"{path.relative_to(output)}: missing/outside target {link}")
            elif url.fragment and target in pages:
                if unquote(url.fragment) not in pages[target].ids:
                    failures.append(f"{path.relative_to(output)}: missing fragment {link}")

    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    print(f"Book check passed: {len(chapters)} chapters, {len(pages)} HTML pages, "
          f"{checked} local links/assets/fragments.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
