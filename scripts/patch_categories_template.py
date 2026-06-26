#!/usr/bin/env python3
from pathlib import Path
path = Path(__file__).resolve().parents[1] / "templates" / "categories.html"
h = path.read_text()
start = h.find("<div class=\"all_cats\">")
end = h.find("<div id=\"ajax_content\">")
assert start >= 0 and end > start
