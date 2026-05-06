#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Quick test runner for development (no Docker required)

.DESCRIPTION
    Runs unit tests and integration tests that don't require Docker.
    Use this for quick feedback during development.

.PARAMETER TestFilter
    Optional test filter to run specific tests

.PARAMETER Verbose
    Show verbose output
#>

param(
    [string]$TestFilter = "",
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$BackendDir = Join-Path $ProjectRoot "qrcode_share_backend"

Write-Host "`n=== Quick Test Run ===`n" -ForegroundColor Cyan

Push-Location $BackendDir
try {
    $env:RUST_LOG = if ($Verbose) { "debug" } else { "info" }

    $testArgs = @("test")
    if ($TestFilter) {
        $testArgs += $TestFilter
    }
    if ($Verbose) {
        $testArgs += "-- --nocapture"
    }

    & cargo $testArgs

    if ($LASTEXITCODE -eq 0) {
        Write-Host "`n=== All Tests Passed ===`n" -ForegroundColor Green
    }
    else {
        Write-Host "`n=== Some Tests Failed ===`n" -ForegroundColor Red
    }

    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
