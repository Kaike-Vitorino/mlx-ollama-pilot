#!/usr/bin/env zsh
set -euo pipefail

SERVICE_LABEL="com.kaike.mlx-ollama-daemon"
USER_ID="$(id -u)"

if launchctl print "gui/${USER_ID}/${SERVICE_LABEL}" >/dev/null 2>&1; then
  launchctl bootout "gui/${USER_ID}/${SERVICE_LABEL}" >/dev/null 2>&1 || true
  echo "LaunchAgent finalizado (${SERVICE_LABEL})."
fi

RUNNING_PIDS=$(lsof -t -nP -iTCP:11435 -sTCP:LISTEN 2>/dev/null || true)
if [ -n "$RUNNING_PIDS" ]; then
  kill $RUNNING_PIDS >/dev/null 2>&1 || true
  echo "Processo(s) na porta 11435 finalizado(s): $RUNNING_PIDS"
fi

if [ -f /tmp/mlx-ollama-daemon.pid ]; then
  PID=$(cat /tmp/mlx-ollama-daemon.pid)
  if ps -p "$PID" >/dev/null 2>&1; then
    kill "$PID"
    echo "Daemon finalizado (PID $PID)."
  else
    echo "PID registrado nao esta ativo."
  fi
  rm -f /tmp/mlx-ollama-daemon.pid
else
  echo "Nenhum PID de daemon encontrado."
fi
