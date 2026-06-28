import subprocess
import sys

def test_no_dead_code():
    result = subprocess.run([sys.executable, "-m", "vulture", "src/cpkb", "--min-confidence", "100"], capture_output=True, text=True)
    assert result.returncode == 0, f"Vulture found dead code:\n{result.stdout}"
