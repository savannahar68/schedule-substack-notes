@echo off
cd /d "%~dp0"

set BIN=%~dp0substack-scheduler-windows-x86_64.exe

if not exist "%BIN%" (
    powershell -Command "Add-Type -AssemblyName PresentationFramework; [System.Windows.MessageBox]::Show('Server binary not found. Make sure all downloaded files are in the same folder.', 'Substack Scheduler')"
    exit /b 1
)

:: Register to run at login via Task Scheduler
schtasks /query /tn "SubstackScheduler" >nul 2>&1
if errorlevel 1 (
    schtasks /create /tn "SubstackScheduler" /tr "\"%BIN%\"" /sc onlogon /rl limited /f >nul
)

:: Start the server now if not already running
tasklist /fi "imagename eq substack-scheduler-windows-x86_64.exe" 2>nul | find /i "substack-scheduler" >nul
if errorlevel 1 (
    start "" /b "%BIN%"
)

powershell -Command "Add-Type -AssemblyName PresentationFramework; [System.Windows.MessageBox]::Show('Substack Scheduler is running and will start automatically at login.', 'Substack Scheduler')"
