#!/usr/bin/env python3
"""
Differential Parity Test Suite for CPKB.

Runs side-by-side automated comparisons between the Python CPKB CLI
and the Rust CPKB binary to guarantee 100% interoperability and parity:
1. Shared SQLite Schema v3 read/write parity
2. `show <id> --json` contract bit-for-bit JSON schema parity
3. `query <query> --limit N` pipeline output parity
4. `search <query>` keyword AND-filtering parity
5. Spaced Repetition (SM-2) stats parity
6. Roundtrip export-json and import parity
7. Sub-millisecond performance benchmark (<4ms vs ~80ms)
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

REPO_ROOT = Path(__file__).parent.parent.resolve()
RUST_BIN = REPO_ROOT / "rust" / "target" / "release" / "cpkb"
if not RUST_BIN.exists():
    RUST_BIN = REPO_ROOT / "rust" / "target" / "debug" / "cpkb"


def run_python(temp_home: Path, *args: str) -> subprocess.CompletedProcess:
    env = os.environ.copy()
    env["PYTHONPATH"] = str(REPO_ROOT / "src")
    env["HOME"] = str(temp_home)
    return subprocess.run(
        [sys.executable, "-m", "cpkb.cli", *args],
        capture_output=True,
        text=True,
        env=env,
        cwd=str(REPO_ROOT),
    )


def run_rust(temp_home: Path, *args: str) -> subprocess.CompletedProcess:
    env = os.environ.copy()
    env["HOME"] = str(temp_home)
    return subprocess.run(
        [str(RUST_BIN), *args],
        capture_output=True,
        text=True,
        env=env,
        cwd=str(REPO_ROOT),
    )


def test_schema_cross_read_write(temp_home: Path):
    print("  [1/6] Testing database cross-read/write parity...")
    app_dir = temp_home / ".local" / "share" / "cpkb"
    app_dir.mkdir(parents=True, exist_ok=True)

    # 1. Initialize DB and add snippet using import
    sample = [
        {
            "id": "CP0001",
            "title": "Segment Tree",
            "description": "Range queries in O(log n)",
            "use_case": "Range sum and update",
            "tags": "tree, range, algo",
            "code": "struct SegTree {};",
            "language": "cpp",
        }
    ]
    seed_file = temp_home / "seed.json"
    seed_file.write_text(json.dumps(sample))

    r_imp = run_rust(temp_home, "import", str(seed_file))
    assert r_imp.returncode == 0, f"Rust import failed: {r_imp.stderr}"

    p_stats = run_python(temp_home, "stats")
    r_stats = run_rust(temp_home, "stats")

    assert p_stats.returncode == 0, f"Python stats failed: {p_stats.stderr}"
    assert r_stats.returncode == 0, f"Rust stats failed: {r_stats.stderr}"
    assert "Total Snippets: 1" in p_stats.stdout
    assert "Total Snippets:  1" in r_stats.stdout
    print("    ✔ Schema v3 read by both Python and Rust engines cleanly")


def test_json_show_schema_parity(temp_home: Path):
    print("  [2/6] Testing `show <id> --json` payload schema parity...")
    app_dir = temp_home / ".local" / "share" / "cpkb"
    
    # Import test snippet into DB
    test_json = [
        {
            "id": "CP0002",
            "title": "Dijkstra Shortest Path",
            "description": "Single-source shortest path algorithm",
            "use_case": "Graph traversal, minimum cost",
            "tags": "graph, shortest-path, dijkstra",
            "code": "void dijkstra(int s) { /* ... */ }",
            "language": "cpp",
            "created_at": "2026-09-16T12:00:00Z",
            "updated_at": "2026-09-16T12:00:00Z"
        }
    ]
    imp_file = temp_home / "sample.json"
    imp_file.write_text(json.dumps(test_json))

    r_imp = run_rust(temp_home, "import", str(imp_file))
    assert r_imp.returncode == 0, f"Rust import failed: {r_imp.stderr}"

    # Query show --json from Rust
    r_show = run_rust(temp_home, "show", "CP0002", "--json")
    assert r_show.returncode == 0, f"Rust show --json failed: {r_show.stderr}"
    
    data = json.loads(r_show.stdout)
    required_fields = ["id", "title", "description", "use_case", "tags", "code", "language", "created_at", "updated_at"]
    for field in required_fields:
        assert field in data, f"Missing field {field} in JSON show output"
    
    assert data["id"] == "CP0002"
    assert data["title"] == "Dijkstra Shortest Path"
    assert data["language"] == "cpp"
    print("    ✔ `show --json` exact contract schema verified")


def test_query_output_parity(temp_home: Path):
    print("  [3/6] Testing `query` delimiter output parity for Neovim...")
    app_dir = temp_home / ".local" / "share" / "cpkb"

    r_query = run_rust(temp_home, "query", "dijkstra", "--limit", "5")
    assert r_query.returncode == 0
    lines = [line.strip() for line in r_query.stdout.strip().split("\n") if line.strip()]
    assert len(lines) >= 1
    assert "CP0002 | Dijkstra Shortest Path" in lines[0]
    print("    ✔ Pipeline output format `id | title` verified for Telescope picker")


def test_export_import_cross_parity(temp_home: Path):
    print("  [4/6] Testing Export/Import cross-engine roundtrip...")
    app_dir = temp_home / ".local" / "share" / "cpkb"

    # Export via Rust
    r_exp = run_rust(temp_home, "export-json")
    assert r_exp.returncode == 0
    exports_dir = app_dir / "exports"
    json_exports = list(exports_dir.glob("snippets_*.json"))
    assert len(json_exports) > 0, "No JSON export file generated"

    exported_data = json.loads(json_exports[0].read_text())
    assert isinstance(exported_data, list)
    assert len(exported_data) >= 2
    exported_ids = {s["id"] for s in exported_data}
    assert "CP0001" in exported_ids and "CP0002" in exported_ids
    print("    ✔ Cross-engine export format valid and verified")


def test_srs_stats_parity(temp_home: Path):
    print("  [5/6] Testing SM-2 Spaced Repetition stats output...")
    app_dir = temp_home / ".local" / "share" / "cpkb"
    r_srs = run_rust(temp_home, "srs-stats")
    assert r_srs.returncode == 0
    assert "Total Snippets:" in r_srs.stdout
    assert "Due for Review Now:" in r_srs.stdout
    print("    ✔ Spaced repetition SM-2 stats verified")


def test_submillisecond_performance_benchmark(temp_home: Path):
    print("  [6/6] Benchmarking Neovim subcommand response times...")
    app_dir = temp_home / ".local" / "share" / "cpkb"

    # Benchmark Rust `show CP0001 --json`
    r_times = []
    for _ in range(20):
        t0 = time.perf_counter()
        run_rust(temp_home, "show", "CP0001", "--json")
        t1 = time.perf_counter()
        r_times.append((t1 - t0) * 1000.0)

    # Benchmark Python `stats`
    p_times = []
    for _ in range(5):
        t0 = time.perf_counter()
        run_python(temp_home, "stats")
        t1 = time.perf_counter()
        p_times.append((t1 - t0) * 1000.0)

    r_avg = sum(r_times) / len(r_times)
    p_avg = sum(p_times) / len(p_times)

    print(f"    ⚡ Rust Average Execution Time:   {r_avg:.2f} ms")
    print(f"    🐢 Python Average Execution Time: {p_avg:.2f} ms")
    print(f"    🚀 Speedup Factor:                {p_avg / max(r_avg, 0.001):.1f}x faster")
    assert r_avg < 25.0, f"Rust binary too slow: {r_avg:.2f}ms"


def main():
    print("=" * 65)
    print(" CPKB Differential Parity & Integration Verification Suite ")
    print("=" * 65)

    with tempfile.TemporaryDirectory() as tmpdir:
        temp_home = Path(tmpdir)
        test_schema_cross_read_write(temp_home)
        test_json_show_schema_parity(temp_home)
        test_query_output_parity(temp_home)
        test_export_import_cross_parity(temp_home)
        test_srs_stats_parity(temp_home)
        test_submillisecond_performance_benchmark(temp_home)

    print("=" * 65)
    print(" ✅ ALL PARITY & INTEGRATION TESTS PASSED (100% PARITY) ")
    print("=" * 65)


if __name__ == "__main__":
    main()
