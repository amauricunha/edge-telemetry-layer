#!/usr/bin/env python3
"""
validate_csv.py — Verifica critérios de aceitação AC-01, AC-02, AC-03, AC-04, AC-08
do Gate F1 da SPEC_F1_Telemetria_CAN_OBD.md.

Uso:
    python tools/validate_csv.py <arquivo.csv> [arquivo2.csv ...]

Exemplos:
    python tools/validate_csv.py S_0001.CSV
    python tools/validate_csv.py S_0001.CSV S_0002.CSV S_0003.CSV
"""

import sys
import os
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

import pandas as pd
import numpy as np


# =============================================================================
# Configurações de threshold conforme SPEC_F1 Seção 7
# =============================================================================
THRESHOLD_OBD_LATENCY_MEAN_MS = 10.0   # AC-02
THRESHOLD_OBD_JITTER_STD_MS   = 3.0    # AC-03
THRESHOLD_MIN_SAMPLES_SESSION  = 24_000  # AC-08 Dual-Source (31 Hz DBC + 10 Hz OBD ≈ 41 Hz × 600s: ~24.600)
THRESHOLD_MIN_SAMPLES_TOTAL    = 72_000  # AC-08 Dual-Source (3 sessões × ~24.000: ~73.800)

SCHEMA_COLUMNS = [
    "timestamp_ms", "source", "can_id",
    "speed_kmh", "rpm", "throttle_pct", "engine_load_pct",
    "maf_g_s", "coolant_temp_c", "session_label", "obd_latency_ms",
]
MANDATORY_COLUMNS = ["timestamp_ms", "source", "can_id", "session_label"]
VALID_SOURCES = {"CAN_DBC", "OBD_PID", "DIAG"}


def _status(ok: bool, msg: str = "") -> str:
    return f"✅ OK{' — ' + msg if msg else ''}" if ok else f"❌ FALHA{' — ' + msg if msg else ''}"


def validate_file(filepath: str) -> dict:
    """Valida um único arquivo CSV e retorna dict de resultados por critério."""
    results = {}
    path = Path(filepath)
    results["arquivo"] = path.name

    # ── Leitura ────────────────────────────────────────────────────────────
    try:
        cols = ["timestamp_ms", "source", "can_id", "speed_kmh", "rpm", "throttle_pct", "engine_load_pct", "maf_g_s", "coolant_temp_c", "session_label", "obd_latency_ms", "extra"]
        with open(filepath, 'r') as f:
            first_line = f.readline()
        skip = 1 if "timestamp_ms" in first_line else 0
        df = pd.read_csv(filepath, names=cols, skiprows=skip, dtype=str)
        is_diag = df["source"] == "DIAG"
        df.loc[is_diag, "session_label"] = df.loc[is_diag, "obd_latency_ms"]
        df.loc[is_diag, "obd_latency_ms"] = df.loc[is_diag, "extra"]
        df = df.drop(columns=["extra"])
    except Exception as e:
        results["LEITURA"] = {"status": f"❌ ERRO ao ler CSV: {e}", "valor": "—"}
        return results

    # ── AC-04: Schema / Integridade ────────────────────────────────────────
    missing_cols = [c for c in SCHEMA_COLUMNS if c not in df.columns]
    if missing_cols:
        results["AC-04_schema"] = {
            "status": f"❌ FALHA — colunas ausentes: {missing_cols}",
            "valor": f"{len(missing_cols)} colunas faltando",
        }
    else:
        nulos_obrigatorios = df[MANDATORY_COLUMNS].isnull().sum().sum()
        results["AC-04_integridade"] = {
            "status": _status(nulos_obrigatorios == 0,
                              f"{nulos_obrigatorios} nulos em colunas obrigatórias"),
            "valor": f"{nulos_obrigatorios} campos nulos indevidos",
        }

    # ── AC-01: Taxa de recepção DBC ────────────────────────────────────────
    dbc_rows = df[df["source"] == "CAN_DBC"]
    obd_rows = df[df["source"] == "OBD_PID"]
    total_data = len(dbc_rows) + len(obd_rows)

    if total_data > 0:
        # Estima frames esperados pela duração da sessão
        data_rows = df[df["source"].isin(["CAN_DBC", "OBD_PID"])]
        if "timestamp_ms" in data_rows.columns and not data_rows["timestamp_ms"].isnull().all():
            ts = pd.to_numeric(data_rows["timestamp_ms"], errors="coerce").dropna()
            if len(ts) >= 2:
                duration_s = (ts.max() - ts.min()) / 1000.0
                # Esperado: 0x100 @ 10 Hz + 0x200 @ 20 Hz = 30 DBC frames/s
                expected_dbc = max(1, duration_s * 30)
                reception_rate = min(100.0, len(dbc_rows) / expected_dbc * 100.0)
                results["AC-01_recepcao_frames"] = {
                    "status": _status(reception_rate >= 99.0, f"{reception_rate:.1f}%"),
                    "valor": f"{len(dbc_rows)} DBC frames em {duration_s:.0f}s (esperado ≈{expected_dbc:.0f})",
                }

    # ── AC-02 / AC-03: Latência e Jitter OBD-II ───────────────────────────
    if len(obd_rows) > 0 and "obd_latency_ms" in df.columns:
        latencias = pd.to_numeric(obd_rows["obd_latency_ms"], errors="coerce").dropna()
        latencias = latencias[latencias > 0]

        if len(latencias) >= 5:
            media = latencias.mean()
            desvio = latencias.std()
            results["AC-02_latencia_media"] = {
                "status": _status(media < THRESHOLD_OBD_LATENCY_MEAN_MS,
                                  f"{media:.2f} ms (limite: <{THRESHOLD_OBD_LATENCY_MEAN_MS} ms)"),
                "valor": f"média={media:.2f} ms | n={len(latencias)}",
            }
            results["AC-03_jitter_obd"] = {
                "status": _status(desvio < THRESHOLD_OBD_JITTER_STD_MS,
                                  f"σ={desvio:.2f} ms (limite: <{THRESHOLD_OBD_JITTER_STD_MS} ms)"),
                "valor": f"std={desvio:.2f} ms | min={latencias.min():.1f} | max={latencias.max():.1f}",
            }
        else:
            results["AC-02_latencia_media"] = {
                "status": "⚠️  INSUFICIENTE — menos de 5 amostras OBD",
                "valor": f"{len(latencias)} amostras",
            }

    # Verificar que linhas CAN_DBC não têm obd_latency_ms preenchido
    if "obd_latency_ms" in df.columns:
        dbc_com_latencia = dbc_rows["obd_latency_ms"].notna().sum()
        results["AC-04_obd_latency_campo"] = {
            "status": _status(dbc_com_latencia == 0,
                              f"{dbc_com_latencia} linhas CAN_DBC com obd_latency_ms indevido"),
            "valor": f"{dbc_com_latencia} linhas CAN_DBC com latência preenchida",
        }

    # ── AC-08: Volume mínimo ────────────────────────────────────────────────
    total_linhas = len(df[df["source"] != "DIAG"])  # Excluir linhas DIAG da contagem
    results["AC-08_volume_sessao"] = {
        "status": _status(
            total_linhas >= THRESHOLD_MIN_SAMPLES_SESSION,
            f"{total_linhas} amostras (meta: ≥{THRESHOLD_MIN_SAMPLES_SESSION:,})",
        ),
        "valor": f"{total_linhas:,} amostras de dados",
    }

    # ── Sumário estatístico ─────────────────────────────────────────────────
    results["_stats"] = {
        "total_linhas": len(df),
        "dbc_frames": len(dbc_rows),
        "obd_frames": len(obd_rows),
        "diag_events": len(df[df["source"] == "DIAG"]) if "source" in df.columns else 0,
        "sessoes": df["session_label"].unique().tolist() if "session_label" in df.columns else [],
    }

    return results


def print_report(results: dict):
    """Imprime relatório formatado de um arquivo."""
    sep = "=" * 65
    print(f"\n{sep}")
    print(f"  RELATÓRIO DE VALIDAÇÃO — {results.get('arquivo', 'N/A')}")
    print(sep)

    stats = results.pop("_stats", {})
    arquivo = results.pop("arquivo", "")

    for criterio, r in results.items():
        print(f"  {criterio:<35} {r['status']}")
        print(f"    └─ {r['valor']}")

    if stats:
        print(f"\n  Estatísticas:")
        print(f"    • Total de linhas:   {stats.get('total_linhas', 0):,}")
        print(f"    • Frames DBC:        {stats.get('dbc_frames', 0):,}")
        print(f"    • Frames OBD-II:     {stats.get('obd_frames', 0):,}")
        print(f"    • Eventos DIAG:      {stats.get('diag_events', 0)}")
        print(f"    • Perfis presentes:  {stats.get('sessoes', [])}")
    print(sep)


def validate_total(all_results: list[dict]) -> dict:
    """Verifica critério AC-08 agregado de todas as sessões."""
    total = sum(r.get("AC-08_volume_sessao", {}).get("valor", "0")
                .split()[0].replace(",", "").replace(".", "")
                .isdigit() and
                int(r.get("AC-08_volume_sessao", {}).get("valor", "0")
                    .split()[0].replace(",", ""))
                for r in all_results)

    # Recalcula corretamente
    total_amostras = 0
    for r in all_results:
        v = r.get("AC-08_volume_sessao", {}).get("valor", "0 amostras")
        try:
            total_amostras += int(v.split()[0].replace(",", ""))
        except Exception:
            pass

    return {
        "AC-08_volume_total_3_sessoes": {
            "status": (f"✅ OK — {total_amostras:,} amostras"
                       if total_amostras >= THRESHOLD_MIN_SAMPLES_TOTAL
                       else f"❌ FALHA — {total_amostras:,} amostras (meta: ≥{THRESHOLD_MIN_SAMPLES_TOTAL:,})"),
            "valor": f"{total_amostras:,} amostras no total (meta: ≥{THRESHOLD_MIN_SAMPLES_TOTAL:,})",
        }
    }


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    arquivos = sys.argv[1:]
    all_results = []

    for arq in arquivos:
        if not os.path.isfile(arq):
            print(f"⚠️  Arquivo não encontrado: {arq}")
            continue
        r = validate_file(arq)
        all_results.append(r.copy())
        print_report(r)

    # Relatório consolidado de 3 sessões (AC-08 total)
    if len(all_results) >= 2:
        print(f"\n{'=' * 65}")
        print(f"  RELATÓRIO CONSOLIDADO — {len(all_results)} sessões")
        print(f"{'=' * 65}")
        total = validate_total(all_results)
        for criterio, r in total.items():
            print(f"  {criterio:<35} {r['status']}")
            print(f"    └─ {r['valor']}")
        print(f"{'=' * 65}\n")
