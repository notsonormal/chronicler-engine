@echo off
REM Run Chronicler Engine server in background
cd /d "%~dp0.."
start "" /b cmd /c "target\debug\chronicler_engine.exe --port 3000 > server.log 2>&1"
