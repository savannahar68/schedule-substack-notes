@echo off
schtasks /delete /tn "SubstackScheduler" /f >nul 2>&1
taskkill /im substack-scheduler-windows-x86_64.exe /f >nul 2>&1
powershell -Command "Add-Type -AssemblyName PresentationFramework; [System.Windows.MessageBox]::Show('Substack Scheduler has been stopped and removed from login items.', 'Substack Scheduler')"
