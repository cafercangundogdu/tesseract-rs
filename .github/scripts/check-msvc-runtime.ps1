# Checks which C runtime the bundled Leptonica and Tesseract libraries were
# compiled against, by reading the linker directives embedded in their object
# files. Needs dumpbin on PATH (MSVC developer environment).
#
#   -Expect dynamic   /MD: objects ask for MSVCRT
#   -Expect static    /MT: objects ask for LIBCMT
#
# With -Exe, the DLL imports of a linked executable are listed too, and for
# the static runtime it must not import any C runtime DLL.
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('dynamic', 'static')]
    [string]$Expect,

    [string]$Exe
)

$ErrorActionPreference = 'Stop'

$dirSuffix = if ($Expect -eq 'static') { '-crt-static' } else { '' }
$buildRoot = Join-Path $env:APPDATA 'tesseract-rs\build'
$targetDirs = @(Get-ChildItem $buildRoot -Directory |
    ForEach-Object { Get-ChildItem $_.FullName -Directory } |
    Where-Object { $_.Name -like "*-windows-msvc$dirSuffix" })

if ($targetDirs.Count -ne 1) {
    throw "Expected one '*-windows-msvc$dirSuffix' build directory under $buildRoot, found $($targetDirs.Count)"
}

$wanted = if ($Expect -eq 'static') { 'LIBCMT' } else { 'MSVCRT' }
$forbidden = if ($Expect -eq 'static') {
    @('MSVCRT', 'MSVCRTD', 'LIBCMTD')
} else {
    @('LIBCMT', 'LIBCMTD', 'MSVCRTD')
}

foreach ($name in 'leptonica', 'tesseract') {
    $lib = Join-Path $targetDirs[0].FullName "$name\lib\$name.lib"
    if (-not (Test-Path $lib)) {
        throw "$lib does not exist"
    }

    $output = dumpbin /nologo /directives $lib
    if ($LASTEXITCODE -ne 0) {
        throw "dumpbin failed for $lib"
    }

    # One "/DEFAULTLIB:<name>" line per object file; collect the distinct names.
    $defaultLibs = @($output |
        Select-String -Pattern '/DEFAULTLIB:"?([A-Za-z0-9_.]+)"?' -AllMatches |
        ForEach-Object { $_.Matches } |
        ForEach-Object { $_.Groups[1].Value.ToUpper() -replace '\.LIB$', '' } |
        Sort-Object -Unique)

    Write-Host "$name.lib default libraries: $($defaultLibs -join ', ')"

    if ($defaultLibs -notcontains $wanted) {
        throw "$name.lib was not built against the $Expect runtime ($wanted missing)"
    }
    foreach ($bad in $forbidden) {
        if ($defaultLibs -contains $bad) {
            throw "$name.lib references $bad, expected the $Expect release runtime only"
        }
    }
}

Write-Host "Both libraries use the $Expect C runtime."

if ($Exe) {
    if (-not (Test-Path $Exe)) {
        throw "$Exe does not exist"
    }

    $output = dumpbin /nologo /dependents $Exe
    if ($LASTEXITCODE -ne 0) {
        throw "dumpbin failed for $Exe"
    }

    $dlls = @($output |
        Select-String -Pattern '^\s+(\S+\.dll)\s*$' |
        ForEach-Object { $_.Matches[0].Groups[1].Value.ToLower() } |
        Sort-Object -Unique)

    Write-Host "$Exe imports: $($dlls -join ', ')"

    $runtimeDlls = @($dlls | Where-Object { $_ -match '^(vcruntime|msvcp|ucrtbase|api-ms-win-crt-)' })
    if ($Expect -eq 'static' -and $runtimeDlls.Count -gt 0) {
        throw "$Exe imports the dynamic C runtime: $($runtimeDlls -join ', ')"
    }
}
