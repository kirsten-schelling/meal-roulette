# Standalone build + run for the C solver. Compiles solver.c together with the
# host harness (csrc/main.c) and runs it, so you can see printf output without
# starting the API. Prefer `just solver`; this script is the engine behind it.
#
#   just solver                      # uses solver/input.json
#   just solver path\to\input.json   # any input file
#
# cl.exe (MSVC) is not on PATH globally — it lives inside the VS Developer
# shell — so we locate the install with vswhere and enter that shell once.

[CmdletBinding()]
param(
    [string]$InputPath = "$PSScriptRoot\input.txt"
)

$ErrorActionPreference = 'Stop'

$csrc  = Join-Path $PSScriptRoot 'csrc'
$build = Join-Path $PSScriptRoot 'build'
$exe   = Join-Path $build 'solver.exe'
New-Item -ItemType Directory -Force -Path $build | Out-Null

if (-not (Get-Command cl -ErrorAction SilentlyContinue)) {
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $vswhere)) {
        throw "vswhere not found. Install Visual Studio (or Build Tools) with the C++ workload."
    }
    $vsPath = & $vswhere -latest -products * `
        -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
        -property installationPath
    if (-not $vsPath) { throw "No Visual Studio install with the C++ tools was found." }
    # Enter-VsDevShell shells out to a bare `vswhere`; put its dir on PATH so it resolves quietly.
    $env:PATH = "$(Split-Path $vswhere);$env:PATH"
    Import-Module (Join-Path $vsPath 'Common7\Tools\Microsoft.VisualStudio.DevShell.dll')
    Enter-VsDevShell -VsInstallPath $vsPath -SkipAutomaticLocation `
        -DevCmdArguments '-arch=x64 -no_logo' | Out-Null
}

Write-Host "[solver] compiling..." -ForegroundColor Cyan
& cl /nologo /W4 /D_CRT_SECURE_NO_WARNINGS /Fe:$exe /Fo:"$build\" (Join-Path $csrc 'solver.c') (Join-Path $csrc 'main.c')
if ($LASTEXITCODE -ne 0) { throw "compile failed ($LASTEXITCODE)" }

Write-Host "[solver] running: $exe $InputPath" -ForegroundColor Cyan
Write-Host ('-' * 60)
& $exe $InputPath
$code = $LASTEXITCODE
Write-Host ('-' * 60)
Write-Host "[solver] exit code: $code" -ForegroundColor Cyan
exit $code
