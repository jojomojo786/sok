from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
def main():
    p = ROOT / "src/models/search_help.rs"
    p.write_text(Path(__file__).with_name("search_help.rs.snippet").read_text())
if __name__ == "__main__": main()
