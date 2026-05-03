@echo off

REM Build Rust binary
cargo auditable install --locked --no-track --bins --root "%PREFIX%" --path .
if errorlevel 1 exit /b 1

cargo-bundle-licenses --format yaml --output THIRDPARTY.yml
if errorlevel 1 exit /b 1

REM Move default themes to themes directory
if not exist "%PREFIX%\share\osmprj\" mkdir "%PREFIX%\share\osmprj\"
xcopy /E /I /Y themes "%PREFIX%\share\osmprj\themes\"
if errorlevel 1 exit /b 1

REM Setup environment variables
if not exist "%PREFIX%\etc\conda\env_vars.d\" mkdir "%PREFIX%\etc\conda\env_vars.d\"

REM Escape backslashes for JSON
set "THEME_PATH=%PREFIX%\share\osmprj\themes\"
set "THEME_PATH_JSON=%THEME_PATH:\=\\%"

(
  echo {
  echo   "OSMPRJ_THEME_PATH": "%THEME_PATH_JSON%"
  echo }
) > "%PREFIX%\etc\conda\env_vars.d\osmprj.json"
if errorlevel 1 exit /b 1
