#!/usr/bin/env python3
"""
send_command.py — Utilitário para envio de comandos remotos ao ESP32 via MQTT.
Uso:
  python tools/send_command.py REPLAY,1
  python tools/send_command.py STOP
  python tools/send_command.py LIST_SESSIONS
  python tools/send_command.py RESET,ECO,10
"""

import os
import sys
import time
import paho.mqtt.client as mqtt

BROKER_IP   = os.environ.get("MQTT_BROKER_IP", "127.0.0.1")
BROKER_PORT = int(os.environ.get("MQTT_BROKER_PORT", 1883))
USER        = os.environ.get("MQTT_USER", None)
PASS        = os.environ.get("MQTT_PASS", None)
TOPIC_CMD   = "/coach/command"
TOPIC_STATUS = "/system/status"
TOPIC_REPLAY = "/telemetry/replay"

received_bytes = 0
replay_done = False

def on_message(client, userdata, msg):
    global received_bytes, replay_done
    if msg.topic == TOPIC_REPLAY:
        chunk_len = len(msg.payload)
        received_bytes += chunk_len
        if received_bytes % 51200 < chunk_len:
            print(f"📦 [REPLAY STREAMING] {received_bytes:,} bytes recebidos do broker...")
    else:
        text = msg.payload.decode('utf-8', errors='ignore')
        print(f"📡 [STATUS RECEBIDO] {msg.topic}: {text}")
        if "REPLAY_COMPLETE" in text:
            replay_done = True

def main():
    global replay_done, received_bytes
    if len(sys.argv) < 2:
        print("Uso: python tools/send_command.py <COMANDO>")
        print("Exemplos:")
        print("  python tools/send_command.py REPLAY,1")
        print("  python tools/send_command.py STOP")
        print("  python tools/send_command.py LIST_SESSIONS")
        print("  python tools/send_command.py RESET,ECO,10")
        sys.exit(1)

    command = " ".join(sys.argv[1:])

    try:
        client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2)
    except AttributeError:
        client = mqtt.Client()
    if USER:
        client.username_pw_set(USER, PASS)
    client.on_message = on_message

    print(f"[*] Conectando ao Broker MQTT em {BROKER_IP}:{BROKER_PORT}...")
    try:
        client.connect(BROKER_IP, BROKER_PORT, 60)
    except Exception as e:
        print(f"❌ Erro ao conectar ao Broker MQTT: {e}")
        sys.exit(1)

    client.subscribe(TOPIC_STATUS)
    client.subscribe(TOPIC_REPLAY)
    client.loop_start()

    time.sleep(0.5)
    print(f"🚀 Enviando comando: '{command}' para {TOPIC_CMD}...")
    res = client.publish(TOPIC_CMD, command, qos=0)
    res.wait_for_publish()

    if "REPLAY" in command:
        print("✓ Comando de REPLAY enviado! Transmitindo dados do SD Card (aguardando conclusão)...")
        timeout = 90
        start = time.time()
        while not replay_done and (time.time() - start) < timeout:
            time.sleep(0.5)
        print(f"\n🏁 Replay finalizado! Total recebido no broker: {received_bytes:,} bytes.")
    else:
        print("✓ Comando enviado! Aguardando resposta (4s)...")
        time.sleep(4)

    client.loop_stop()
    client.disconnect()
    print("[*] Concluído.")

if __name__ == "__main__":
    main()
