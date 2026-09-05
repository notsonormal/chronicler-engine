#!/usr/bin/env bash
# Start/stop/status for the turbo-agent verified-response proxy.
# Usage: ./turbo-proxy.sh {start|stop|status|restart} [port]
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PORT="${2:-8899}"
PID_FILE="$DIR/proxy.pid"
LOG_FILE="$DIR/proxy.log"
VENV="$DIR/.venv"
AUTH_FILE="$HOME/.pi/agent/auth.json"

ensure_venv() {
    if [ ! -x "$VENV/bin/python" ]; then
        echo "Creating venv and installing turbo-agent..."
        python3 -m venv "$VENV"
        "$VENV/bin/pip" install --quiet turbo-agent
    fi
}

api_key() {
    # Read the Synthetic key from pi's auth store; never print it.
    python3 -c "import json; print(json.load(open('$AUTH_FILE'))['synthetic']['key'])"
}

is_running() {
    [ -f "$PID_FILE" ] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null
}

start() {
    if is_running; then
        echo "Proxy already running (pid $(cat "$PID_FILE")) on port $PORT."
        return 0
    fi
    ensure_venv
    echo "Starting turbo-agent proxy on 127.0.0.1:$PORT..."
    (cd "$DIR" && \
        SYNTHETIC_API_KEY="$(api_key)" \
        OPENAI_API_BASE=https://api.synthetic.new/openai/v1 \
        OPENAI_BASE_URL=https://api.synthetic.new/openai/v1 \
        nohup "$VENV/bin/python" run_proxy.py --host 127.0.0.1 --port "$PORT" \
        >> "$LOG_FILE" 2>&1 & echo $! > "$PID_FILE")
    sleep 2
    if is_running; then
        echo "Proxy running (pid $(cat "$PID_FILE")). Model: openai/hf:zai-org/GLM-5.3-Flash"
    else
        echo "Proxy failed to start; last log lines:" >&2
        tail -5 "$LOG_FILE" >&2
        return 1
    fi
}

stop() {
    if is_running; then
        kill "$(cat "$PID_FILE")"
        rm -f "$PID_FILE"
        echo "Proxy stopped."
    else
        rm -f "$PID_FILE"
        echo "Proxy not running."
    fi
}

status() {
    if is_running; then
        echo "Proxy running (pid $(cat "$PID_FILE"))."
    else
        echo "Proxy not running."
        return 1
    fi
}

case "${1:-status}" in
    start) start ;;
    stop) stop ;;
    status) status ;;
    restart) stop; start ;;
    *) echo "Usage: $0 {start|stop|status|restart} [port]"; exit 1 ;;
esac
