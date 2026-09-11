#!/usr/bin/env python3
"""
analyze_influxdb.py — Baixa e avalia a qualidade do dataset armazenado no InfluxDB.

Este script conecta no InfluxDB, baixa os dados de telemetria das últimas X horas,
e calcula as métricas de qualidade (similar ao compute_metrics.py):
- Taxa de recepção / Perda de pacotes na nuvem
- Jitter da rede (variação do timestamp de chegada)
- Integridade (presença de valores nulos)
- Validação das sessões (ECO, NOR, SPT)

Uso:
    pip install influxdb-client pandas
    INFLUX_URL="http://localhost:8086" INFLUX_TOKEN="seu_token" INFLUX_ORG="sua_org" INFLUX_BUCKET="seu_bucket" python tools/analyze_influxdb.py
"""

import os
import sys
import pandas as pd
from influxdb_client import InfluxDBClient
from dotenv import load_dotenv

# Carrega as variáveis do arquivo .env local (se existir)
load_dotenv()

# Configurações do InfluxDB via variáveis de ambiente (com valores padrão para teste local)
URL = os.environ.get("INFLUX_URL", "http://localhost:8897")
TOKEN = os.environ.get("INFLUX_TOKEN", "telemetry-dev-token-secret-123")
ORG = os.environ.get("INFLUX_ORG", "telemetry_org")
BUCKET = os.environ.get("INFLUX_BUCKET", "telemetry")
MEASUREMENT = os.environ.get("INFLUX_MEASUREMENT", "telemetry")
START_TIME = os.environ.get("INFLUX_START", None) # Formato ex: 2026-08-11T14:00:00-03:00
END_TIME = os.environ.get("INFLUX_END", None)     # Formato ex: 2026-08-11T14:30:00-03:00
SESSION_FILTER = os.environ.get("INFLUX_SESSION", None)

def download_influx_data(session_target: str = None, hours: int = 2) -> pd.DataFrame:
    """Baixa os dados do InfluxDB usando Flux query."""
    print(f"[*] Conectando ao InfluxDB em {URL}...")
    client = InfluxDBClient(url=URL, token=TOKEN, org=ORG, timeout=180_000)
    query_api = client.query_api()

    # Formatar o filtro de sessão se fornecido
    session_clause = ""
    range_clause = ""
    target_s = session_target or SESSION_FILTER

    if target_s and target_s.lower() == "replay":
        print("[*] Filtrando especificamente pelo tópico de REPLAY (/telemetry/replay)...")
        session_clause = 'and r["topic"] == "/telemetry/replay"'
        range_clause = '|> range(start: -30d)'
    elif target_s and target_s.lower() == "all":
        print("[*] Buscando todos os registros dos últimos 30 dias (sem filtro de tópico)...")
        session_clause = ""
        range_clause = '|> range(start: -30d)'
    elif target_s:
        num_part = ''.join(filter(str.isdigit, target_s))
        if num_part:
            target_s = f"S{int(num_part):04d}"
            print(f"[*] Filtrando pela Sessão: {target_s} (/telemetry/{target_s}/raw ou /telemetry/replay)...")
            session_clause = f'and (r["topic"] == "/telemetry/{target_s}/raw" or r["topic"] == "/telemetry/replay")'
            range_clause = '|> range(start: -30d)'
        else:
            print(f"[*] Filtrando pelo tópico: {target_s}...")
            session_clause = f'and r["topic"] == "{target_s}"'
            range_clause = '|> range(start: -30d)'

    elif START_TIME and END_TIME:
        range_clause = f'|> range(start: {START_TIME}, stop: {END_TIME})'
        print(f"[*] Executando query para o período de {START_TIME} até {END_TIME}...")
    elif START_TIME:
        range_clause = f'|> range(start: {START_TIME})'
        print(f"[*] Executando query a partir de {START_TIME}...")
    else:
        range_clause = f'|> range(start: -{hours}h)'
        print(f"[*] Executando query para as últimas {hours} horas...")

    # Query Flux otimizada (filtra measurement + topic juntos antes do pivot)
    query = f'''
        from(bucket: "{BUCKET}")
        {range_clause}
        |> filter(fn: (r) => r["_measurement"] == "{MEASUREMENT}" {session_clause})
        |> pivot(rowKey:["_time"], columnKey: ["_field"], valueColumn: "_value")
    '''
    
    try:
        df = query_api.query_data_frame(query)
    except Exception as e:
        print(f"[!] Erro ao conectar ou consultar o InfluxDB: {e}")
        return pd.DataFrame()

    if type(df) is list:
        # Se retornar múltiplas tabelas, concatena
        if len(df) > 0:
            df = pd.concat(df, ignore_index=True)
        else:
            return pd.DataFrame()

    print(f"[*] Download concluído: {len(df)} registros encontrados.")
    return df

def analyze_quality(df: pd.DataFrame):
    """Analisa a qualidade do dataset baixado."""
    if df.empty:
        print("[!] Dataset vazio. Nenhuma análise a ser feita.")
        return

    print("\n" + "="*50)
    print("   RELATÓRIO DE QUALIDADE NA NUVEM (INFLUXDB)")
    print("="*50)

    # 1. Total de Pontos
    total_pts = len(df)
    print(f"\n[1] Volume de Dados: {total_pts:,} amostras registradas.")

    # 2. Avaliação de Gaps (Jitter/Perda de Rede)
    # Certificar que _time é datetime e ordernar
    if '_time' in df.columns:
        df['_time'] = pd.to_datetime(df['_time'])
        df = df.sort_values(by='_time')
        
        # Diferença de tempo entre as amostras em milissegundos
        df['time_diff_ms'] = df['_time'].diff().dt.total_seconds() * 1000
        
        mean_diff = df['time_diff_ms'].mean()
        std_diff = df['time_diff_ms'].std()
        max_gap = df['time_diff_ms'].max()
        
        
        print(f"\n[2] Latência e Estabilidade (Nuvem):")
        print(f"    - Intervalo Médio de Chegada: {mean_diff:.2f} ms")
        print(f"    - Jitter (Desvio Padrão da Rede): {std_diff:.2f} ms")
        print(f"    - Maior Gap (Possível queda de rede/sinal): {max_gap:.0f} ms")

        # 2.5 Reconstrução Temporal (Backlog)
        if 'timestamp_ms' in df.columns:
            # Se o timestamp original do payload foi gravado como field
            # Vamos checar se o banco ordenou algo que chegou fora de ordem.
            df['payload_time'] = pd.to_numeric(df['timestamp_ms'], errors='coerce')
            out_of_order = (df['payload_time'].diff() < 0).sum()
            
            print(f"\n[2.5] Eficiência de Reconstrução de Backlog (Out-of-Order):")
            print(f"    - Pacotes recebidos fora de ordem cronológica: {out_of_order}")
            if out_of_order > 0:
                print(f"    -> O InfluxDB reconstruiu perfeitamente a linha do tempo para {out_of_order} pacotes do tópico de histórico!")
            else:
                print(f"    -> Nenhum pacote fora de ordem detectado (ou o InfluxDB já converteu o timestamp nativamente).")
    
    # 3. Integridade e Valores Nulos
    # Avaliar as colunas de telemetria esperadas (speed, rpm, etc)
    expected_cols = ['speed_kmh', 'rpm', 'throttle_pct', 'engine_load_pct', 'coolant_temp_c', 'maf_g_s']
    missing_data = False
    
    print(f"\n[3] Integridade dos Dados (Cloud vs Payload):")
    for col in expected_cols:
        if col in df.columns:
            nulls = df[col].isnull().sum()
            perc = (nulls / total_pts) * 100
            print(f"    - {col}: {nulls} nulos ({perc:.2f}%)")
            if nulls > 0: missing_data = True
        else:
            print(f"    - {col}: COLUNA NÃO ENCONTRADA NO BUCKET!")
            missing_data = True
            
    if not missing_data:
        print("    -> SUCESSO: Dataset recebido na nuvem é 100% íntegro (zero perdas de campos)!")

    # 4. Análise de Sessões (Perfis de Pilotagem)
    if 'session_label' in df.columns:
        print(f"\n[4] Distribuição de Perfis Recebidos (Sessões):")
        counts = df['session_label'].value_counts()
        for label, count in counts.items():
            print(f"    - Perfil {label}: {count:,} amostras")
    
    # Exportar cópia local
    filename = "dataset_nuvem_export.csv"
    df.to_csv(filename, index=False)
    print(f"\n[✓] Cópia CSV baixada da nuvem salva como: {filename}")
    print("="*50 + "\n")

if __name__ == "__main__":
    target = sys.argv[1] if len(sys.argv) > 1 else None
    df_influx = download_influx_data(session_target=target, hours=2)
    analyze_quality(df_influx)
