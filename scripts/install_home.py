#!/usr/bin/env python3
from pathlib import Path
Path("src/handlers/home.rs").write_text(Path("scripts/home.rs.stub").read_text())
