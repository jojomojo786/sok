#!/usr/bin/env python3
import json,re,sys
from pathlib import Path
p=Path("docs/raw/live-inventory-2026-06-26/video-sample__desktop.json")
d=json.loads(p.read_text())
m=d["main"]
print("v-tags", "v-tags" in m)
print("leave_comment", "leave_comment" in m)
print("more_video", "more_video" in m)
i=m.find("video-meta"); print(m[i:i+2000] if i>=0 else "none")
