import subprocess, time, requests, os, json, sys

port = 9997
settings_dir = os.path.join(os.environ["TEMP"], f"chronicler_test_settings_0_{port}")
os.makedirs(settings_dir, exist_ok=True)
settings_path = os.path.join(settings_dir, "settings.json")
with open(settings_path, "w") as f:
    json.dump({
        "connections": [{"id": "openrouter-gpt-4o-mini", "name": "openrouter-gpt-4o-mini", "provider": "Mock", "model": "mock-model", "api_key": None, "base_url": None}],
        "narration_connection_id": "openrouter-gpt-4o-mini",
        "quantifier_connection_id": "openrouter-gpt-4o-mini"
    }, f)

db_path = os.path.join("target", "debug", "data", "chronicler.db")
if os.path.exists(db_path):
    os.remove(db_path)

env = {**os.environ, "CHRONICLER_SETTINGS_PATH": settings_path}
proc = subprocess.Popen(["target/debug/chronicler_engine.exe", "--world", "test", "--port", str(port)], env=env)
time.sleep(3)

try:
    r = requests.post(f"http://127.0.0.1:{port}/action/check", data={"command": "hello test"}, timeout=10)
    print("POST status:", r.status_code)
    print("POST body:", r.text[:500])
    time.sleep(4)
    r2 = requests.get(f"http://127.0.0.1:{port}/fragment/story-log", timeout=10)
    print("GET status:", r2.status_code)
    print("Story log:", r2.text[:2000])
    r3 = requests.get(f"http://127.0.0.1:{port}/fragment/action-area", timeout=10)
    print("Action area status:", r3.status_code)
    print("Action area:", r3.text[:1000])
finally:
    proc.terminate()
    proc.wait(timeout=5)
