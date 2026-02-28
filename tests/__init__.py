from __future__ import annotations

import sys
from pathlib import Path

# Allow running tests without installing the package (src/ layout).
SRC_DIR = (Path(__file__).resolve().parent.parent / "src").resolve()
if str(SRC_DIR) not in sys.path:
    sys.path.insert(0, str(SRC_DIR))
