@echo off
REM Windows entrypoint for protoc-windows-wrapper.sh (std::process can launch .cmd).
REM Requires Git Bash on PATH (true on GitHub windows-*- runners).
setlocal
set "SCRIPT_DIR=%~dp0"
REM Prefer bash from Git for Windows; fall back to bash on PATH.
set "BASH="
if exist "C:\Program Files\Git\bin\bash.exe" set "BASH=C:\Program Files\Git\bin\bash.exe"
if not defined BASH if exist "C:\Program Files (x86)\Git\bin\bash.exe" set "BASH=C:\Program Files (x86)\Git\bin\bash.exe"
if not defined BASH set "BASH=bash"
"%BASH%" "%SCRIPT_DIR%protoc-windows-wrapper.sh" %*
exit /b %ERRORLEVEL%
