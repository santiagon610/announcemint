param(
  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$Command
)

$ErrorActionPreference = "Stop"

function Find-VsWhere {
  $candidates = @(
    (Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\\Installer\\vswhere.exe"),
    (Join-Path ${env:ProgramFiles} "Microsoft Visual Studio\\Installer\\vswhere.exe")
  )

  foreach ($p in $candidates) {
    if (Test-Path $p) { return $p }
  }

  return $null
}

$vswhere = Find-VsWhere
if (-not $vswhere) {
  Write-Error @"
Could not find vswhere.exe.

Install Visual Studio Build Tools 2022 with the C++ workload, then retry:
  winget install -e --id Microsoft.VisualStudio.2022.BuildTools --accept-package-agreements --accept-source-agreements --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive --wait --norestart"
"@
}

$installPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $installPath) {
  Write-Error "Visual Studio installation with C++ tools not found. Re-run the Build Tools install with the VC++ workload."
}

$vsDevCmd = Join-Path $installPath "Common7\\Tools\\VsDevCmd.bat"
if (-not (Test-Path $vsDevCmd)) {
  Write-Error "VsDevCmd.bat not found at: $vsDevCmd"
}

if (-not $Command -or $Command.Count -eq 0) {
  Write-Host "Launching Developer Command Prompt..."
  cmd.exe /k "`"$vsDevCmd`" -arch=x64 -host_arch=x64"
  exit $LASTEXITCODE
}

# Run the provided command under the VS Developer environment
$cmdLine = ($Command -join " ")
cmd.exe /c "`"$vsDevCmd`" -arch=x64 -host_arch=x64 && $cmdLine"
exit $LASTEXITCODE

