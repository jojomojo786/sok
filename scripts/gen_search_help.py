from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
def w(rel, text):
    (ROOT / rel).write_text(text)
