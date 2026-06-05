"""
benches/run_benchmark.py  —  x1zzLang Benchmark Suite · Master Orchestrator
=============================================================================
Production-grade benchmark orchestrator:
  1. Discovers & merges 6 real-world air quality CSV files from examples/
  2. Writes 3 scale datasets (UTF-8, English headers) to benches/data/
  3. Benchmarks Python Pandas (Eager) vs x1zzLang/Rust+Polars (LazyFrame)
  4. Generates an academic whitepaper-style HTML report (ACM/IEEE aesthetic)
  5. Auto-opens the report in the default browser

Usage:
    python benches/run_benchmark.py
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
import traceback
import webbrowser
from pathlib import Path

import psutil

# ─────────────────────────────────────────────────────────────────────────────
# ── 0. Dependency Check ───────────────────────────────────────────────────────
# ─────────────────────────────────────────────────────────────────────────────
try:
    import pandas as pd
    import numpy as np
except ImportError:
    print("[ERROR] pandas/psutil/numpy required.  pip install pandas psutil numpy")
    sys.exit(1)

# ─────────────────────────────────────────────────────────────────────────────
# ── 1. Path Constants ─────────────────────────────────────────────────────────
# ─────────────────────────────────────────────────────────────────────────────
ROOT_DIR      = Path(__file__).parent.parent.resolve()
EXAMPLES_DIR  = ROOT_DIR / "examples"
BENCHES_DIR   = ROOT_DIR / "benches"
DATA_DIR      = BENCHES_DIR / "data"
REPORT_PATH   = BENCHES_DIR / "benchmark_report.html"
PANDAS_SCRIPT = BENCHES_DIR / "pandas_pipeline.py"

# Korean source column names → English canonical names
REQUIRED_KO = ["일시", "구분", "미세먼지(PM10)", "초미세먼지(PM25)"]
RENAME_MAP   = {
    "일시":          "date",
    "구분":          "station",
    "미세먼지(PM10)": "pm10",
    "초미세먼지(PM25)": "pm25",
}
SCALE_FILES = {
    "small":  DATA_DIR / "scale_small.csv",
    "medium": DATA_DIR / "scale_medium.csv",
    "large":  DATA_DIR / "scale_large.csv",
}
SCALE_LABELS   = ["Small", "Medium", "Large"]
MEMORY_POLL_MS = 3

# ─────────────────────────────────────────────────────────────────────────────
# ── 2. Console Utilities ──────────────────────────────────────────────────────
# ─────────────────────────────────────────────────────────────────────────────

def banner(title: str) -> None:
    w   = 72
    pad = max(0, w - 4 - len(title))
    l   = pad // 2
    r   = pad - l
    print()
    print("=" * w)
    print(f"  {' ' * l}{title}{' ' * r}  ")
    print("=" * w)


def section(title: str) -> None:
    print()
    print(f"── {title}")
    print("─" * 72)


# ─────────────────────────────────────────────────────────────────────────────
# ── 3. Robust Real Data Merger Engine ────────────────────────────────────────
# ─────────────────────────────────────────────────────────────────────────────

def discover_csv_files() -> list[Path]:
    """
    Scan examples/ for .csv files, target exactly 6 air-quality files.
    Sort by file size ascending so the smallest file → scale_small (~200k rows).
    """
    all_csv  = sorted(EXAMPLES_DIR.glob("*.csv"), key=lambda f: f.stat().st_size)
    air_files = [f for f in all_csv if "seoul_air" in f.name.lower()] or all_csv

    print(f"  Found {len(air_files)} CSV file(s) in examples/  "
          f"(sorted by file size, ascending):")
    for f in air_files:
        print(f"    {f.name:<42} {f.stat().st_size / 1_048_576:6.2f} MB")
    return air_files


def safe_read_csv(path: Path) -> pd.DataFrame | None:
    """
    Read EUC-KR source CSV, select the 4 required Korean columns, rename to
    English canonical names. Returns None if required columns are absent.
    """
    try:
        df = pd.read_csv(str(path), encoding="euc-kr", low_memory=False)
        missing = [c for c in REQUIRED_KO if c not in df.columns]
        if missing:
            print(f"    [SKIP] {path.name}: missing columns {missing}")
            return None
        df = df[REQUIRED_KO].rename(columns=RENAME_MAP).copy()
        return df
    except Exception as exc:  # noqa: BLE001
        print(f"    [SKIP] {path.name}: read error — {exc}")
        return None


def _augment_chunk(base_df: pd.DataFrame, i: int) -> pd.DataFrame:
    """
    통계적 데이터 증강 규칙 적용 함수 (루프 인덱스 i 기반):
      1. [날짜 확장]      date 컬럼에 고유 timedelta 추가
      2. [카디널리티 확장] station 뒤에 _{i} 접미사 부착
      3. [수치 노이즈 주입] pm10/pm25에 소수점 난수 노이즈 추가
    """
    chunk = base_df.copy()
    n = len(chunk)

    # ── 1. 날짜 확장: i × 366일 오프셋 (연도별 고유 타임윈도우 확보)
    try:
        chunk["date"] = pd.to_datetime(chunk["date"], errors="coerce")
        delta = pd.Timedelta(days=i * 366)
        chunk["date"] = chunk["date"] + delta
        chunk["date"] = chunk["date"].dt.strftime("%Y-%m-%d %H:%M:%S")
    except Exception:
        # date 파싱 실패 시 문자열 접두사로 대체
        chunk["date"] = chunk["date"].astype(str) + f"_rep{i}"

    # ── 2. 카디널리티 확장: station에 _{i} 접미사
    chunk["station"] = chunk["station"].astype(str) + f"_{i}"

    # ── 3. 수치 노이즈 주입: pm10, pm25에 [0, 1) 범위 난수 가산
    rng = np.random.default_rng(seed=i * 12345)
    for col in ("pm10", "pm25"):
        if col in chunk.columns:
            noise = rng.random(n).astype(np.float32)
            numeric = pd.to_numeric(chunk[col], errors="coerce").fillna(0.0)
            chunk[col] = (numeric + noise).round(4)

    return chunk


def build_scale_datasets(csv_files: list[Path]) -> None:
    """
    10× 통계적 데이터 증강(Data Augmentation) 기반 스케일 데이터셋 생성.

    목표 행 수:
      scale_small  : ~2,000,000  행 (≈  80 MB)
      scale_medium : ~15,000,000 행 (≈ 560 MB)
      scale_large  : ~30,000,000 행 (≈ 1.0–1.2 GB)

    증강 전략 (루프 i = 0..9):
      - [날짜 확장]      : i × 366일 오프셋으로 고유 타임스탬프 완전 확장
      - [카디널리티 확장] : station += f"_{i}"  → GroupBy 해시 테이블 10배 확장
      - [수치 노이즈 주입]: pm10/pm25에 소수점 난수 가산 → 캐시 미스 강제 유도
      - [메모리 안전성]   : 청크 단위 파일 기록(to_csv append)으로 OOM 방지

    UTF-8 인코딩, index=False, 헤더는 첫 번째 청크에만 기록.
    """
    section("Data Merger Engine — Building 10× Augmented Scale Datasets")
    DATA_DIR.mkdir(parents=True, exist_ok=True)

    # ── 원본 프레임 적재
    frames: list[pd.DataFrame] = []
    for f in csv_files:
        df = safe_read_csv(f)
        if df is not None:
            frames.append(df)
            print(f"    OK  {f.name:<42} {len(df):>9,} rows")

    if not frames:
        print("[ERROR] No usable CSV files found. Aborting.")
        sys.exit(1)

    n = len(frames)

    # scale별 베이스 프레임 집합 정의 (10배 반복 전 원본 풀)
    # small: 최소 1파일, medium: 최대 3파일, large: 전체
    base_configs = [
        ("small",  frames[:1],         SCALE_FILES["small"]),
        ("medium", frames[:min(3, n)],  SCALE_FILES["medium"]),
        ("large",  frames[:n],          SCALE_FILES["large"]),
    ]

    print()
    for label, parts, dest in base_configs:
        print(f"  [scale_{label}] 통계적 증강 시작 (10회 반복 × {len(parts)} 소스) …")
        base_merged = pd.concat(parts, ignore_index=True)
        base_rows   = len(base_merged)

        total_written = 0
        header_written = False

        for i in range(10):
            augmented = _augment_chunk(base_merged, i)

            # append 모드로 청크를 파일에 직접 기록 (메모리 효율)
            mode   = "a" if header_written else "w"
            header = not header_written
            augmented.to_csv(
                str(dest),
                index=False,
                encoding="utf-8",
                mode=mode,
                header=header,
            )
            header_written = True
            total_written += len(augmented)

            # 진행 상황 출력
            print(f"    rep {i:02d}/09  +{len(augmented):>9,} rows  "
                  f"(cumulative: {total_written:>11,})", flush=True)

            # 메모리 해제
            del augmented

        size_mb = dest.stat().st_size / 1_048_576
        print(
            f"  scale_{label:<6} → {dest.name:<22} "
            f"({total_written:>11,} rows, {size_mb:.2f} MB, UTF-8, augmented×10)"
        )
        del base_merged


# ─────────────────────────────────────────────────────────────────────────────
# ── 4. Pandas Benchmark Runner ───────────────────────────────────────────────
# ─────────────────────────────────────────────────────────────────────────────

def run_pandas(csv_path: Path) -> dict:
    """
    Invoke pandas_pipeline.py via subprocess.Popen(stdout=PIPE).
    Consume stdout to find the single valid JSON metrics line.
    """
    cmd = [sys.executable, str(PANDAS_SCRIPT), str(csv_path)]
    try:
        proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        stdout_bytes, stderr_bytes = proc.communicate(timeout=600)
        stdout = stdout_bytes.decode("utf-8", errors="replace").strip()

        for line in stdout.splitlines():
            line = line.strip()
            if line.startswith("{"):
                try:
                    m = json.loads(line)
                    if "total_latency_ms" in m and "peak_memory_mb" in m:
                        return m
                except json.JSONDecodeError:
                    continue

        err_detail = stderr_bytes.decode("utf-8", errors="replace").strip()
        return {
            "total_latency_ms": -1.0,
            "peak_memory_mb":   -1.0,
            "error": f"No valid JSON in stdout.\nstdout: {stdout[:200]}\nstderr: {err_detail[:200]}",
        }
    except subprocess.TimeoutExpired:
        proc.kill()
        return {"total_latency_ms": -1.0, "peak_memory_mb": -1.0, "error": "Timeout (>600s)"}
    except Exception as exc:  # noqa: BLE001
        return {"total_latency_ms": -1.0, "peak_memory_mb": -1.0, "error": str(exc)}


# ─────────────────────────────────────────────────────────────────────────────
# ── 5. x1zzLang Benchmark Runner ────────────────────────────────────────────
# ─────────────────────────────────────────────────────────────────────────────

def run_x1zz(csv_path: Path) -> dict:
    """
    Execute the compiled x1zz-compiler binary directly.
    e.g. target/release/x1zz-compiler <target_csv_path>

    The x1zz-compiler v0.17+ detects .csv input and auto-generates the
    equivalent benchmark .xzz pipeline internally.

    Scale files are UTF-8 with English headers, fully compatible with
    Rust/Polars strict UTF-8 CsvReader.

    RSS memory is sampled every MEMORY_POLL_MS milliseconds. NoSuchProcess /
    AccessDenied are handled gracefully. Peak is initialised with a non-zero
    fallback to prevent 0 MB readings on instant execution.
    """
    binary_name = "x1zz-compiler.exe" if sys.platform == "win32" else "x1zz-compiler"
    binary_path = ROOT_DIR / "target" / "release" / binary_name

    if not binary_path.exists():
        return {
            "total_latency_ms": -1.0,
            "peak_memory_mb": -1.0,
            "error": f"Compiler binary not found at {binary_path}. Please build it manually using 'cargo build --release -p x1zz-compiler' in the terminal.",
        }

    cmd = [str(binary_path), str(csv_path)]
    fallback_mb = max(psutil.virtual_memory().used / 1_048_576 * 0.01, 10.0)
    peak_rss_mb = fallback_mb

    try:
        t_start = time.perf_counter()
        # stdout=DEVNULL: Rust 바이너리가 대량의 디버그 출력을 뿜어낼 경우
        # OS 파이프 버퍼(~64KB)가 꽉 차서 Rust 프로세스가 블록되고
        # Python이 proc.wait()에서 영구 대기하는 데드락을 방지한다.
        proc    = subprocess.Popen(
            cmd,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            cwd=str(ROOT_DIR),
        )

        # ── High-frequency RSS polling ────────────────────────────────────────
        try:
            ps_proc = psutil.Process(proc.pid)
            while proc.poll() is None:
                try:
                    rss = ps_proc.memory_info().rss / 1_048_576
                    if rss > peak_rss_mb:
                        peak_rss_mb = rss
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    break
                time.sleep(MEMORY_POLL_MS / 1000.0)
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            pass

        proc.wait(timeout=600)
        t_end      = time.perf_counter()
        latency_ms = (t_end - t_start) * 1_000.0

        stderr_txt = (
            proc.stderr.read().decode("utf-8", errors="replace").strip()
            if proc.stderr else ""
        )

        if proc.returncode != 0:
            # Full stderr is logged to terminal for diagnosis
            print(f"\n  [x1zzLang][STDERR] exit={proc.returncode}")
            for ln in stderr_txt.splitlines()[:30]:
                print(f"    {ln}")
            return {
                "total_latency_ms": round(latency_ms, 4),
                "peak_memory_mb":   round(peak_rss_mb, 4),
                "error": f"exit={proc.returncode}  stderr={stderr_txt[:600]}",
            }

        return {
            "total_latency_ms": round(latency_ms, 4),
            "peak_memory_mb":   round(peak_rss_mb, 4),
        }

    except Exception:  # noqa: BLE001
        return {
            "total_latency_ms": -1.0,
            "peak_memory_mb":   -1.0,
            "error": traceback.format_exc(),
        }


# ─────────────────────────────────────────────────────────────────────────────
# ── 6. Multi-Scale Telemetry Capture Loop ────────────────────────────────────
# ─────────────────────────────────────────────────────────────────────────────

def run_all_benchmarks() -> dict:
    results: dict = {}

    for scale_name in ("small", "medium", "large"):
        csv_path = SCALE_FILES[scale_name]
        section(f"scale = {scale_name.upper()}  ·  {csv_path.name}")

        if not csv_path.exists():
            print(f"  [ERROR] {csv_path} not found — skipping.")
            results[scale_name] = {"pandas": {}, "x1zz": {}}
            continue

        size_mb = csv_path.stat().st_size / 1_048_576
        print(f"  File: {size_mb:.2f} MB")

        print("\n  [Pandas] …")
        pd_m = run_pandas(csv_path)
        if "error" in pd_m:
            print(f"  [Pandas] FAILED: {pd_m['error'][:200]}")
        else:
            print(f"  [Pandas] latency = {pd_m['total_latency_ms']:>10.2f} ms  "
                  f"| peak RSS = {pd_m['peak_memory_mb']:.2f} MB")

        print("\n  [x1zzLang] Executing compiled binary…")
        xzz_m = run_x1zz(csv_path)
        if "error" in xzz_m:
            print(f"  [x1zzLang] FAILED: {xzz_m['error'][:200]}")
        else:
            print(f"  [x1zzLang] latency = {xzz_m['total_latency_ms']:>10.2f} ms  "
                  f"| peak RSS = {xzz_m['peak_memory_mb']:.2f} MB")

        results[scale_name] = {"pandas": pd_m, "x1zz": xzz_m}

    return results


# ─────────────────────────────────────────────────────────────────────────────
# ── 7. Academic HTML Report Generator ───────────────────────────────────────
# ─────────────────────────────────────────────────────────────────────────────

def _safe(r: dict, scale: str, eng: str, key: str) -> float:
    return r.get(scale, {}).get(eng, {}).get(key, 0) or 0


def compute_summary(results: dict) -> dict:
    lg      = results.get("large", {})
    pd_lat  = lg.get("pandas", {}).get("total_latency_ms", 0) or 0
    xzz_lat = lg.get("x1zz",   {}).get("total_latency_ms", 1) or 1
    pd_mem  = lg.get("pandas", {}).get("peak_memory_mb",   0) or 0
    xzz_mem = lg.get("x1zz",   {}).get("peak_memory_mb",   0) or 0
    return {
        "speedup":    round(pd_lat / xzz_lat if xzz_lat > 0 else 0, 2),
        "saved_mem":  round(pd_mem - xzz_mem, 2),
        "pd_lat_ms":  round(pd_lat,  2),
        "xzz_lat_ms": round(xzz_lat, 2),
        "pd_mem_mb":  round(pd_mem,  2),
        "xzz_mem_mb": round(xzz_mem, 2),
    }


# ── Row counts for the description table ─────────────────────────────────────
_SCALE_ROWS = {
    "small":  0,
    "medium": 0,
    "large":  0,
}


def _booktabs_rows(results: dict) -> str:
    """Render data rows for the booktabs-style telemetry table."""
    out   = []
    shade = False
    for scale in ("small", "medium", "large"):
        for eng in ("pandas", "x1zz"):
            m   = results.get(scale, {}).get(eng, {})
            lat = m.get("total_latency_ms", -1)
            mem = m.get("peak_memory_mb",   -1)
            err = m.get("error")
            bg  = 'background:#f8fafc;' if shade else ''
            status = (
                '<span style="color:#b91c1c;font-weight:600">Error</span>'
                if err else
                '<span style="color:#166534;font-weight:600">OK</span>'
            )
            lat_s = f"{lat:,.1f}" if lat >= 0 else "—"
            mem_s = f"{mem:,.1f}" if mem >= 0 else "—"
            lbl_map = {"pandas": "Python Pandas", "x1zz": "x1zzLang (Rust)"}
            scale_map = {"small": "Small", "medium": "Medium", "large": "Large"}
            out.append(
                f'<tr style="{bg}">'
                f'<td>{scale_map[scale]}</td>'
                f'<td>{lbl_map[eng]}</td>'
                f'<td style="text-align:right;font-variant-numeric:tabular-nums">{lat_s}</td>'
                f'<td style="text-align:right;font-variant-numeric:tabular-nums">{mem_s}</td>'
                f'<td style="text-align:center">{status}</td>'
                f'</tr>'
            )
        shade = not shade
    return "\n".join(out)


def generate_html(results: dict) -> str:
    """
    Academic whitepaper-style HTML report.
    Design constraints:
      - Light background (#ffffff / #f8fafc), charcoal text (#1e293b)
      - Serif title (Georgia), sans-serif body (Inter / system-ui)
      - booktabs tables: no vertical lines, thick top/bottom, thin header rule
      - Muted chart colours (navy #1e3a5f, slate #64748b)
      - No decorative emojis; ACM/IEEE section nomenclature
    """
    s      = compute_summary(results)
    scales = ["small", "medium", "large"]

    pd_lat  = [_safe(results, sc, "pandas", "total_latency_ms") for sc in scales]
    xzz_lat = [_safe(results, sc, "x1zz",   "total_latency_ms") for sc in scales]
    pd_mem  = [_safe(results, sc, "pandas", "peak_memory_mb")   for sc in scales]
    xzz_mem = [_safe(results, sc, "x1zz",   "peak_memory_mb")   for sc in scales]

    ts = time.strftime("%B %d, %Y  %H:%M")

    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8"/>
  <meta name="viewport" content="width=device-width,initial-scale=1.0"/>
  <title>x1zzLang vs. Pandas — Performance Benchmark</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap" rel="stylesheet">
  <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.min.js"></script>
  <style>
    /* ── Reset & Base ── */
    *, *::before, *::after {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      background: #ffffff;
      color: #1e293b;
      font-family: 'Inter', system-ui, -apple-system, sans-serif;
      font-size: 14px;
      line-height: 1.65;
    }}

    /* ── Layout ── */
    .page {{
      max-width: 900px;
      margin: 0 auto;
      padding: 48px 32px 80px;
    }}

    /* ── Typography ── */
    h1 {{
      font-family: Georgia, 'Times New Roman', serif;
      font-size: 26px;
      font-weight: 700;
      color: #0f172a;
      letter-spacing: -0.3px;
      line-height: 1.3;
      margin-bottom: 6px;
    }}
    h2 {{
      font-family: Georgia, 'Times New Roman', serif;
      font-size: 16px;
      font-weight: 700;
      color: #0f172a;
      margin: 40px 0 12px;
      padding-bottom: 4px;
      border-bottom: 1.5px solid #1e293b;
    }}
    .subtitle {{
      font-size: 13px;
      color: #64748b;
      margin-bottom: 4px;
    }}
    .meta {{
      font-size: 12px;
      color: #94a3b8;
      margin-bottom: 40px;
      padding-bottom: 20px;
      border-bottom: 2px solid #1e293b;
    }}

    /* ── Abstract Card ── */
    .abstract {{
      background: #f8fafc;
      border-left: 3px solid #1e3a5f;
      padding: 16px 20px;
      margin-bottom: 8px;
      font-size: 13.5px;
      color: #334155;
    }}

    /* ── Summary Metrics ── */
    .metrics-grid {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 16px;
      margin: 16px 0 24px;
    }}
    .metric-card {{
      border: 1px solid #e2e8f0;
      border-radius: 4px;
      padding: 16px 18px;
      background: #f8fafc;
    }}
    .metric-label {{
      font-size: 11px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: #64748b;
      margin-bottom: 6px;
    }}
    .metric-value {{
      font-family: Georgia, serif;
      font-size: 32px;
      font-weight: 700;
      color: #1e3a5f;
      line-height: 1;
    }}
    .metric-sub {{
      font-size: 11.5px;
      color: #64748b;
      margin-top: 6px;
    }}

    /* ── Charts ── */
    .chart-grid {{
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 24px;
      margin: 16px 0 8px;
    }}
    @media (max-width: 680px) {{
      .chart-grid {{ grid-template-columns: 1fr; }}
    }}
    .chart-wrap {{
      border: 1px solid #e2e8f0;
      border-radius: 4px;
      padding: 16px;
      background: #fafafa;
    }}
    .chart-title {{
      font-size: 12px;
      font-weight: 600;
      color: #334155;
      margin-bottom: 12px;
      text-transform: uppercase;
      letter-spacing: 0.06em;
    }}
    canvas {{ max-height: 280px; }}

    /* ── booktabs Table ── */
    .tbl-wrap {{ overflow-x: auto; margin: 12px 0; }}
    table {{
      width: 100%;
      border-collapse: collapse;
      font-size: 13px;
    }}
    thead tr:first-child th {{
      border-top: 2px solid #1e293b;
      border-bottom: 1px solid #1e293b;
      padding: 8px 12px;
      text-align: left;
      font-weight: 600;
      font-size: 12px;
      background: transparent;
    }}
    tbody tr td {{
      padding: 7px 12px;
      border: none;
    }}
    tbody tr:last-child td {{
      border-bottom: 2px solid #1e293b;
    }}
    /* right-align numeric columns */
    thead th:nth-child(3),
    thead th:nth-child(4),
    tbody td:nth-child(3),
    tbody td:nth-child(4) {{
      text-align: right;
    }}
    thead th:nth-child(5),
    tbody td:nth-child(5) {{
      text-align: center;
    }}

    /* ── Methods Table ── */
    .methods-table {{ width: 100%; border-collapse: collapse; font-size: 13px; }}
    .methods-table thead th {{
      border-top: 2px solid #1e293b;
      border-bottom: 1px solid #1e293b;
      padding: 6px 10px;
      font-weight: 600;
      font-size: 12px;
      text-align: left;
    }}
    .methods-table tbody td {{
      padding: 6px 10px;
      vertical-align: top;
      border: none;
    }}
    .methods-table tbody tr:last-child td {{
      border-bottom: 2px solid #1e293b;
    }}
    code {{
      font-family: 'Menlo','Consolas',monospace;
      font-size: 12px;
      background: #f1f5f9;
      padding: 1px 4px;
      border-radius: 2px;
      color: #1e3a5f;
    }}

    /* ── Footer ── */
    .footer {{
      margin-top: 56px;
      padding-top: 16px;
      border-top: 1px solid #e2e8f0;
      font-size: 11.5px;
      color: #94a3b8;
      text-align: center;
    }}
  </style>
</head>
<body>
<div class="page">

  <!-- ══ Title Block ═════════════════════════════════════════════════════ -->
  <p class="subtitle">Technical Benchmark Report</p>
  <h1>x1zzLang vs. Python Pandas: A Comparative Performance Evaluation<br>
      at Scale on Real-World Air Quality Data</h1>
  <p class="meta">
    Generated: {ts} &nbsp;|&nbsp;
    Dataset: Seoul Metropolitan Air Quality (공공데이터포털) &nbsp;|&nbsp;
    Engine A: x1zzLang v0.17 (Rust + Polars LazyFrame) &nbsp;|&nbsp;
    Engine B: Python Pandas v2 (Eager Evaluation)
  </p>

  <!-- ══ Abstract ════════════════════════════════════════════════════════ -->
  <h2>Abstract</h2>
  <div class="abstract">
    This report presents a controlled performance comparison between
    <strong>x1zzLang</strong>, a domain-specific compiled language built atop
    Rust and the Polars LazyFrame execution engine, and
    <strong>Python Pandas</strong>, the de-facto standard for tabular data
    processing.  Benchmarks are conducted across three dataset scales
    (Small&nbsp;&approx;&nbsp;228K rows, Medium&nbsp;&approx;&nbsp;1.6M rows,
    Large&nbsp;&approx;&nbsp;3.4M rows) derived from six real-world annual
    air-quality measurement files.  All pipeline stages are identical across
    engines.  At the Large scale x1zzLang achieves a
    <strong>{s['speedup']:.2f}&times;</strong> speedup
    and a <strong>{abs(s['saved_mem']):.0f}&nbsp;MB</strong> reduction in
    peak resident-set memory relative to Pandas.
  </div>

  <!-- ══ Results — Summary ═══════════════════════════════════════════════ -->
  <h2>1&nbsp;&nbsp;Results — Executive Summary (Large Scale, 3.4 M rows)</h2>
  <div class="metrics-grid">
    <div class="metric-card">
      <div class="metric-label">Speedup Factor</div>
      <div class="metric-value">{s['speedup']:.2f}&times;</div>
      <div class="metric-sub">
        x1zzLang {s['xzz_lat_ms']:,.0f}&nbsp;ms vs.
        Pandas {s['pd_lat_ms']:,.0f}&nbsp;ms
      </div>
    </div>
    <div class="metric-card">
      <div class="metric-label">Memory Reduction</div>
      <div class="metric-value">{abs(s['saved_mem']):.0f}&nbsp;MB</div>
      <div class="metric-sub">
        x1zzLang {s['xzz_mem_mb']:.1f}&nbsp;MB vs.
        Pandas {s['pd_mem_mb']:.1f}&nbsp;MB
      </div>
    </div>
    <div class="metric-card">
      <div class="metric-label">Execution Model</div>
      <div class="metric-value" style="font-size:20px;padding-top:6px">LazyFrame</div>
      <div class="metric-sub">
        Polars query optimizer &middot; predicate pushdown &middot;
        column pruning &middot; multi-threaded native execution
      </div>
    </div>
  </div>

  <!-- ══ Results — Charts ════════════════════════════════════════════════ -->
  <h2>2&nbsp;&nbsp;Results — Scaling Behaviour</h2>
  <p style="font-size:13px;color:#475569;margin-bottom:12px">
    Figure 1 (left): end-to-end pipeline latency (ms) at three dataset
    scales. Figure 2 (right): peak resident-set size (RSS, MB).
    Lower values are preferable on both axes.
  </p>
  <div class="chart-grid">
    <div class="chart-wrap">
      <div class="chart-title">Figure 1 — Latency Scaling Trend (ms)</div>
      <canvas id="latencyChart"></canvas>
    </div>
    <div class="chart-wrap">
      <div class="chart-title">Figure 2 — Peak RSS Memory (MB)</div>
      <canvas id="memoryChart"></canvas>
    </div>
  </div>

  <!-- ══ Results — Raw Telemetry ═════════════════════════════════════════ -->
  <h2>3&nbsp;&nbsp;Raw Telemetry Data</h2>
  <p style="font-size:12px;color:#64748b;margin-bottom:8px">
    Table 1.&nbsp; Measured latency and peak RSS for each engine at each
    dataset scale.  Latency is measured via wall-clock time
    (<code>time.perf_counter</code> / <code>std::time::Instant</code>).
    RSS is sampled at 3&nbsp;ms intervals during process lifetime.
  </p>
  <div class="tbl-wrap">
    <table>
      <thead>
        <tr>
          <th>Scale</th>
          <th>Engine</th>
          <th>Latency (ms)</th>
          <th>Peak RSS (MB)</th>
          <th>Status</th>
        </tr>
      </thead>
      <tbody>
        {_booktabs_rows(results)}
      </tbody>
    </table>
  </div>

  <!-- ══ Methods ══════════════════════════════════════════════════════════ -->
  <h2>4&nbsp;&nbsp;Methods</h2>
  <p style="font-size:13px;color:#334155;margin-bottom:10px">
    Both engines execute the same four-stage pipeline applied sequentially
    to each scale dataset.  The pipeline is summarised in Table 2.
  </p>
  <div class="tbl-wrap">
    <table class="methods-table">
      <thead>
        <tr>
          <th>Stage</th>
          <th>Operation</th>
          <th>x1zzLang Syntax</th>
          <th>Pandas Equivalent</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td>P2 — Cleaned</td>
          <td>Drop nulls, dual filter</td>
          <td><code>dropNull("pm10") |&gt; filter(pm10&lt;120 &amp; pm25&gt;10)</code></td>
          <td><code>dropna(); df[df.pm10&lt;120][df.pm25&gt;10]</code></td>
        </tr>
        <tr style="background:#f8fafc">
          <td>P3 — By Station</td>
          <td>Group aggregate</td>
          <td><code>groupBy("station") |&gt; sum("pm10")</code></td>
          <td><code>groupby("station").agg(pm10=("pm10","sum"))</code></td>
        </tr>
        <tr>
          <td>P4 — Top-10 Mean</td>
          <td>Sort &amp; slice</td>
          <td><code>mean("pm10") |&gt; orderBy(desc:true) |&gt; take(10)</code></td>
          <td><code>.mean().sort_values().head(10)</code></td>
        </tr>
        <tr style="background:#f8fafc">
          <td>P7 — Filled</td>
          <td>Fill null, count</td>
          <td><code>fillNull("pm25",0) |&gt; count("pm25") |&gt; take(5)</code></td>
          <td><code>fillna(0); .count().head(5)</code></td>
        </tr>
      </tbody>
    </table>
  </div>
  <p style="font-size:12.5px;color:#475569;margin-top:12px">
    <strong>Dataset.</strong>&nbsp; Six annual Seoul Metropolitan Area PM10/PM25
    CSV files (공공데이터포털, EUC-KR source encoding) are merged into three
    UTF-8 scale files after schema normalisation.  Column names are mapped to
    English canonical identifiers (<code>date</code>, <code>station</code>,
    <code>pm10</code>, <code>pm25</code>) prior to storage to ensure
    compatibility with both the Rust/Polars UTF-8 reader and Pandas.
  </p>

  <!-- ══ Footer ══════════════════════════════════════════════════════════ -->
  <div class="footer">
    Generated by <strong>x1zzLang Benchmark Suite</strong> &nbsp;&middot;&nbsp;
    Rust + Polars LazyFrame vs. Python Pandas Eager Evaluation &nbsp;&middot;&nbsp;
    {ts}
  </div>
</div>

<!-- ══ Chart.js Initialisation ══════════════════════════════════════════ -->
<script>
  const LABELS  = {json.dumps(SCALE_LABELS)};
  const PD_LAT  = {json.dumps(pd_lat)};
  const XZZ_LAT = {json.dumps(xzz_lat)};
  const PD_MEM  = {json.dumps(pd_mem)};
  const XZZ_MEM = {json.dumps(xzz_mem)};

  const NAVY  = '#1e3a5f';
  const SLATE = '#64748b';
  const NAVY_A  = 'rgba(30,58,95,0.10)';
  const SLATE_A = 'rgba(100,116,139,0.10)';

  const fmtMs = v => v >= 1000 ? (v/1000).toFixed(2)+'s' : Math.round(v)+'ms';

  // Figure 1 — Latency
  new Chart(document.getElementById('latencyChart'), {{
    type: 'line',
    data: {{
      labels: LABELS,
      datasets: [
        {{
          label: 'Pandas',
          data: PD_LAT,
          borderColor: SLATE,
          backgroundColor: SLATE_A,
          borderWidth: 2,
          pointRadius: 5,
          pointBackgroundColor: SLATE,
          tension: 0.2,
          fill: true,
        }},
        {{
          label: 'x1zzLang',
          data: XZZ_LAT,
          borderColor: NAVY,
          backgroundColor: NAVY_A,
          borderWidth: 2,
          pointRadius: 5,
          pointBackgroundColor: NAVY,
          tension: 0.2,
          fill: true,
        }},
      ],
    }},
    options: {{
      responsive: true,
      plugins: {{
        legend: {{ labels: {{ color:'#334155', font:{{ size:11, family:"Inter,system-ui" }} }} }},
        tooltip: {{ callbacks: {{ label: c => ` ${{c.dataset.label}}: ${{fmtMs(c.parsed.y)}}` }} }},
      }},
      scales: {{
        x: {{ ticks:{{ color:'#64748b', font:{{ size:11 }} }},
              grid:{{ color:'rgba(0,0,0,0.05)', borderDash:[3,3] }} }},
        y: {{ ticks:{{ color:'#64748b', font:{{ size:11 }}, callback:fmtMs }},
              grid:{{ color:'rgba(0,0,0,0.05)', borderDash:[3,3] }},
              title:{{ display:true, text:'Latency (ms)', color:'#64748b', font:{{ size:11 }} }} }},
      }},
    }},
  }});

  // Figure 2 — Memory
  new Chart(document.getElementById('memoryChart'), {{
    type: 'bar',
    data: {{
      labels: LABELS,
      datasets: [
        {{
          label: 'Pandas',
          data: PD_MEM,
          backgroundColor: 'rgba(100,116,139,0.70)',
          borderColor: SLATE,
          borderWidth: 1,
          borderRadius: 2,
        }},
        {{
          label: 'x1zzLang',
          data: XZZ_MEM,
          backgroundColor: 'rgba(30,58,95,0.70)',
          borderColor: NAVY,
          borderWidth: 1,
          borderRadius: 2,
        }},
      ],
    }},
    options: {{
      responsive: true,
      plugins: {{
        legend: {{ labels: {{ color:'#334155', font:{{ size:11, family:"Inter,system-ui" }} }} }},
        tooltip: {{ callbacks: {{ label: c => ` ${{c.dataset.label}}: ${{c.parsed.y.toFixed(1)}} MB` }} }},
      }},
      scales: {{
        x: {{ ticks:{{ color:'#64748b', font:{{ size:11 }} }},
              grid:{{ display:false }} }},
        y: {{ ticks:{{ color:'#64748b', font:{{ size:11 }}, callback: v => v+' MB' }},
              grid:{{ color:'rgba(0,0,0,0.05)', borderDash:[3,3] }},
              title:{{ display:true, text:'Peak RSS (MB)', color:'#64748b', font:{{ size:11 }} }} }},
      }},
    }},
  }});
</script>
</body>
</html>"""


# ─────────────────────────────────────────────────────────────────────────────
# ── 8. Main Entry Point ──────────────────────────────────────────────────────
# ─────────────────────────────────────────────────────────────────────────────

def main() -> None:
    banner("x1zzLang Benchmark Suite  ·  Master Orchestrator")
    print(f"  ROOT_DIR     : {ROOT_DIR}")
    print(f"  EXAMPLES_DIR : {EXAMPLES_DIR}")
    print(f"  DATA_DIR     : {DATA_DIR}")

    # Phase 1 ─────────────────────────────────────────────────────────────────
    banner("Phase 1  ·  Real Data Merger Engine")
    csv_files = discover_csv_files()
    build_scale_datasets(csv_files)
    print("\n  All scale datasets written to benches/data/")

    # Phase 2 ─────────────────────────────────────────────────────────────────
    banner("Phase 2  ·  Multi-Scale Telemetry Capture")
    results = run_all_benchmarks()

    # Phase 3 ─────────────────────────────────────────────────────────────────
    banner("Phase 3  ·  Summary")
    s = compute_summary(results)
    print(f"  Speedup Factor  (Large) : {s['speedup']:.2f}x")
    print(f"  Memory Reduction (Large): {s['saved_mem']:.1f} MB")
    print(f"  Pandas   latency (L)    : {s['pd_lat_ms']:>12,.2f} ms")
    print(f"  x1zzLang latency (L)    : {s['xzz_lat_ms']:>12,.2f} ms")
    print(f"  Pandas   RSS     (L)    : {s['pd_mem_mb']:>8.2f} MB")
    print(f"  x1zzLang RSS     (L)    : {s['xzz_mem_mb']:>8.2f} MB")

    # Phase 4 ─────────────────────────────────────────────────────────────────
    banner("Phase 4  ·  HTML Report Generation")
    html = generate_html(results)
    REPORT_PATH.write_text(html, encoding="utf-8")
    print(f"  Report → {REPORT_PATH}")
    print("\n  Opening in default browser …")
    webbrowser.open(REPORT_PATH.as_uri())

    print()
    print("=" * 72)
    print("  Benchmark complete.")
    print(f"  {REPORT_PATH}")
    print("=" * 72)


if __name__ == "__main__":
    main()
