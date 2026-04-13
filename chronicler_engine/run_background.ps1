# Run Chronicler Engine server in background
#
# PURPOSE:
# - For local manual testing and development
# - Starts server with test world on port 3000
# - Server runs in hidden window, survives terminal closure
#
# USAGE:
#   powershell -ExecutionPolicy Bypass -File chronicler_engine/run_background.ps1
#
# To stop: taskkill /F /IM chronicler_engine.exe
#
# For CI/Testing: Use TestServer in tests/ui_tests.rs which manages its own server

$exePath = "D:\John\DevContainer\mrn-general\chronicler_engine\target\debug\chronicler_engine.exe"
$workingDir = "D:\John\DevContainer\mrn-general\chronicler_engine"

$proc = Start-Process -FilePath $exePath -WorkingDirectory $workingDir -ArgumentList "--world test --port 3000" -WindowStyle Hidden -PassThru

Write-Host "Server started with PID: $($proc.Id)"
Write-Host "Server running at http://127.0.0.1:3000 (test world)"
