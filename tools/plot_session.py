#!/usr/bin/env python3
"""
plot_session.py — Gera as Figuras 5 e 6 do artigo eTech.

Figura 5: Boxplot de latência OBD-II nos três cenários (Eco, Normal, Esportivo)
Figura 6: Série temporal de velocidade, RPM e throttle nos primeiros 60 s do Cenário C3 (Esportivo)

Uso:
    python tools/plot_session.py <eco.csv> <normal.csv> <esportivo.csv>

    Onde cada arquivo corresponde a um cenário de condução.
    Se apenas um arquivo for passado, gera somente a Figura 6 daquele arquivo.

Saída:
    docs/figuras/figura_05_boxplot_latencia.png    (300 DPI)
    docs/figuras/figura_06_serie_temporal.png      (300 DPI)
"""

import sys
import os
import warnings
from pathlib import Path

import numpy as np
import pandas as pd
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.gridspec import GridSpec

warnings.filterwarnings("ignore")

# =============================================================================
# Configuração visual do artigo (estilo IEEE/eTech)
# =============================================================================
plt.rcParams.update({
    "font.family": "DejaVu Sans",
    "font.size": 10,
    "axes.labelsize": 11,
    "axes.titlesize": 12,
    "xtick.labelsize": 9,
    "ytick.labelsize": 9,
    "legend.fontsize": 9,
    "figure.dpi": 100,
    "axes.grid": True,
    "grid.alpha": 0.3,
    "grid.linestyle": "--",
})

COLORS = {
    "Eco": "#2ca02c",       # verde
    "Normal": "#1f77b4",    # azul
    "Esportivo": "#d62728", # vermelho
    "speed": "#1f77b4",
    "rpm": "#ff7f0e",
    "throttle": "#2ca02c",
}

OUTPUT_DIR = Path("docs/figuras")


def load_csv_safe(path: str, label: str) -> pd.DataFrame | None:
    """Carrega CSV e adiciona coluna _label."""
    try:
        cols = ["timestamp_ms", "source", "can_id", "speed_kmh", "rpm", "throttle_pct", "engine_load_pct", "maf_g_s", "coolant_temp_c", "session_label", "obd_latency_ms", "extra"]
        with open(path, 'r') as f:
            first_line = f.readline()
        skip = 1 if "timestamp_ms" in first_line else 0
        if skip == 0:
            print(f"  ! Aviso: {path} sem cabeçalho. Aplicando schema manual.")

        df = pd.read_csv(path, names=cols, skiprows=skip, dtype=str)
        
        is_diag = df["source"] == "DIAG"
        df.loc[is_diag, "session_label"] = df.loc[is_diag, "obd_latency_ms"]
        df.loc[is_diag, "obd_latency_ms"] = df.loc[is_diag, "extra"]
        
        df = df.drop(columns=["extra"])
        
        df["_label"] = label
        print(f"  ✓ {label}: {path} ({len(df):,} linhas)")
        return df
    except Exception as e:
        print(f"  ✗ Erro ao carregar {path}: {e}")
        return None


def figura_05_boxplot(dfs: list[pd.DataFrame], labels: list[str]):
    """
    Figura 5: Boxplot de distribuição da latência OBD-II por cenário.
    Reproduz Tabela 6 e Figura 5 do artigo.
    """
    fig, ax = plt.subplots(figsize=(7, 4.5))

    data = []
    cores = []
    valid_labels = []
    stats_table = []

    for df, label in zip(dfs, labels):
        if "obd_latency_ms" not in df.columns:
            continue
        obd = df[df["source"] == "OBD_PID"] if "source" in df.columns else df
        lat = pd.to_numeric(obd["obd_latency_ms"], errors="coerce").dropna()
        lat = lat[lat > 0]
        if len(lat) < 5:
            continue

        data.append(lat.values)
        cores.append(COLORS.get(label, "#888888"))
        valid_labels.append(label)
        stats_table.append({
            "Cenário": label,
            "n": len(lat),
            "Média (ms)": f"{lat.mean():.2f}",
            "σ (ms)": f"{lat.std():.2f}",
            "P50 (ms)": f"{lat.median():.2f}",
            "P95 (ms)": f"{lat.quantile(0.95):.2f}",
            "Máx (ms)": f"{lat.max():.1f}",
        })

    if not data:
        print("  ⚠️  Sem dados OBD-II para gerar Figura 5.")
        plt.close()
        return

    bps = ax.boxplot(
        data,
        labels=valid_labels,
        patch_artist=True,
        medianprops=dict(color="black", linewidth=2),
        flierprops=dict(marker="o", markerfacecolor="gray", markersize=3, alpha=0.5),
        whiskerprops=dict(linewidth=1.2),
        capprops=dict(linewidth=1.5),
    )

    for patch, color in zip(bps["boxes"], cores):
        patch.set_facecolor(color)
        patch.set_alpha(0.75)

    # Linha de referência AC-02 (10 ms)
    ax.axhline(y=10.0, color="red", linestyle="--", linewidth=1.2, label="Limite AC-02 (10 ms)")
    # Linha de referência AC-03 threshold visual
    ax.axhline(y=3.0, color="orange", linestyle=":", linewidth=1.0, label="Limite σ AC-03 (3 ms)")

    ax.set_xlabel("Cenário de Condução")
    ax.set_ylabel("Latência OBD-II (ms)")
    ax.set_title("Figura 5 — Distribuição da Latência OBD-II por Cenário de Condução")
    ax.set_ylim(bottom=0.0, top=5.0)  # Limitar a 5ms para visualizar o corpo da distribuição
    ax.legend(loc="upper right", framealpha=0.9)
    
    # Nota sobre outliers
    ax.text(0.02, 0.95, "Outliers > 5ms (0.01% dos dados) não mostrados no gráfico",
            transform=ax.transAxes, fontsize=8, color="gray",
            bbox=dict(facecolor='white', alpha=0.8, edgecolor='none'))

    # Anotar mediana em cada box
    for i, (lat_arr, bp) in enumerate(zip(data, bps["medians"])):
        median_val = float(bp.get_ydata()[0])
        ax.text(i + 1, median_val + 0.15, f"{median_val:.1f}", ha="center", va="bottom",
                fontsize=8, fontweight="bold")

    fig.tight_layout()
    out = OUTPUT_DIR / "figura_05_boxplot_latencia.png"
    fig.savefig(out, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"  ✓ Figura 5 salva: {out}")

    # Imprimir tabela de estatísticas no terminal
    if stats_table:
        df_stats = pd.DataFrame(stats_table)
        print(f"\n  Tabela 6 — Latência OBD-II por Cenário:")
        print(df_stats.to_string(index=False))


def figura_06_serie_temporal(df: pd.DataFrame, label: str = "Esportivo"):
    """
    Figura 6: Série temporal dos primeiros 60 s — velocidade, RPM e throttle.
    """
    if "timestamp_ms" not in df.columns:
        print("  ⚠️  Coluna timestamp_ms ausente. Figura 6 não gerada.")
        return

    df_plot = df.copy()
    df_plot["timestamp_ms"] = pd.to_numeric(df_plot["timestamp_ms"], errors="coerce")
    df_plot = df_plot.dropna(subset=["timestamp_ms"]).sort_values("timestamp_ms")

    df_data = df_plot[df_plot["source"].isin(["CAN_DBC", "OBD_PID"])]
    t0 = df_data["timestamp_ms"].min() if len(df_data) > 0 else df_plot["timestamp_ms"].min()
    df_plot["tempo_s"] = (df_plot["timestamp_ms"] - t0) / 1000.0

    # Filtrar primeiros 60 s de coleta efetiva
    df60 = df_plot[(df_plot["tempo_s"] >= 0) & (df_plot["tempo_s"] <= 60.0)]
    if len(df60) < 10:
        print("  ⚠️  Menos de 10 pontos nos primeiros 60s. Figura 6 pode ficar vazia.")

    fig = plt.figure(figsize=(9, 6))
    gs = GridSpec(3, 1, figure=fig, hspace=0.4)

    # ─ Painel 1: Velocidade
    ax1 = fig.add_subplot(gs[0])
    speed_data = df60[pd.to_numeric(df60.get("speed_kmh", pd.Series()), errors="coerce").notna()]
    if "speed_kmh" in df60.columns and len(speed_data) > 0:
        speed_vals = pd.to_numeric(speed_data["speed_kmh"], errors="coerce")
        ax1.plot(speed_data["tempo_s"], speed_vals, color=COLORS["speed"],
                 linewidth=1.2, label="Velocidade")
        ax1.fill_between(speed_data["tempo_s"], speed_vals, alpha=0.15, color=COLORS["speed"])
    ax1.set_ylabel("Velocidade (km/h)")
    ax1.set_title(f"Figura 6 — Série Temporal Primeiros 60 s — Cenário {label}")
    ax1.legend(loc="upper right", framealpha=0.9)
    ax1.set_xlim(0, 60)

    # ─ Painel 2: RPM
    ax2 = fig.add_subplot(gs[1])
    rpm_data = df60[pd.to_numeric(df60.get("rpm", pd.Series()), errors="coerce").notna()]
    if "rpm" in df60.columns and len(rpm_data) > 0:
        rpm_vals = pd.to_numeric(rpm_data["rpm"], errors="coerce")
        ax2.plot(rpm_data["tempo_s"], rpm_vals, color=COLORS["rpm"],
                 linewidth=1.2, label="RPM")
        ax2.fill_between(rpm_data["tempo_s"], rpm_vals, alpha=0.15, color=COLORS["rpm"])
    ax2.set_ylabel("Rotação (RPM)")
    ax2.legend(loc="upper right", framealpha=0.9)
    ax2.set_xlim(0, 60)

    # ─ Painel 3: Throttle
    ax3 = fig.add_subplot(gs[2])
    thr_data = df60[pd.to_numeric(df60.get("throttle_pct", pd.Series()), errors="coerce").notna()]
    if "throttle_pct" in df60.columns and len(thr_data) > 0:
        thr_vals = pd.to_numeric(thr_data["throttle_pct"], errors="coerce")
        ax3.plot(thr_data["tempo_s"], thr_vals, color=COLORS["throttle"],
                 linewidth=1.2, label="Throttle")
        ax3.fill_between(thr_data["tempo_s"], thr_vals, alpha=0.15, color=COLORS["throttle"])
    ax3.set_xlabel("Tempo (s)")
    ax3.set_ylabel("Acelerador (%)")
    ax3.legend(loc="upper right", framealpha=0.9)
    ax3.set_xlim(0, 60)

    fig.tight_layout()
    out = OUTPUT_DIR / "figura_06_serie_temporal.png"
    fig.savefig(out, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"  ✓ Figura 6 salva: {out}")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    print(f"\n{'=' * 60}")
    print(f"  PLOT SESSION — Figuras para o Artigo eTech")
    print(f"{'=' * 60}")

    arquivos = [a for a in sys.argv[1:] if os.path.isfile(a)]
    cenarios = ["Eco", "Normal", "Esportivo"]

    dfs = []
    labels_usados = []
    for i, arq in enumerate(arquivos):
        label = cenarios[i] if i < len(cenarios) else f"C{i+1}"
        df = load_csv_safe(arq, label)
        if df is not None:
            dfs.append(df)
            labels_usados.append(label)

    if not dfs:
        print("❌ Nenhum arquivo válido carregado.")
        sys.exit(1)

    print(f"\n  Gerando Figura 5 (Boxplot de Latência OBD-II)...")
    figura_05_boxplot(dfs, labels_usados)

    print(f"\n  Gerando Figura 6 (Série Temporal — últimos dados = cenário mais intenso)...")
    # Usar o último arquivo para Figura 6 (preferencialmente Esportivo = C3)
    df_c3 = dfs[-1]
    label_c3 = labels_usados[-1]
    figura_06_serie_temporal(df_c3, label_c3)

    print(f"\n  📊 Figuras salvas em: {OUTPUT_DIR}/")
    print(f"{'=' * 60}\n")
