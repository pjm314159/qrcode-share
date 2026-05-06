#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Run integration tests for QRcode Share Backend

.DESCRIPTION
    This script runs integration tests using Docker containers.
    It supports both testcontainers (cargo test) and docker-compose modes.

.PARAMETER Mode
    Test mode: "testcontainers" (default) or "docker-compose"

.PARAMETER TestFilter
    Optional test filter to run specific tests

.PARAMETER NoCleanup
    Keep Docker containers running after tests

.PARAMETER Verbose
    Show verbose output

.EXAMPLE
    ./scripts/run-tests.ps1
    Run all tests using testcontainers

.EXAMPLE
    ./scripts/run-tests.ps1 -Mode docker-compose
    Run tests using docker-compose

.EXAMPLE
    ./scripts/run-tests.ps1 -TestFilter "test_message_flow"
    Run specific test
#>

param(
    [ValidateSet("testcontainers", "docker-compose")]
    [string]$Mode = "testcontainers",

    [string]$TestFilter = "",

    [switch]$NoCleanup,

    [switch]$Verbose
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$BackendDir = Join-Path $ProjectRoot "qrcode_share_backend"

function Write-Header {
    param([string]$Message)
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host " $Message" -ForegroundColor Cyan
    Write-Host "========================================`n" -ForegroundColor Cyan
}

function Test-DockerAvailable {
    try {
        docker info | Out-Null
        return $true
    }
    catch {
        Write-Error "Docker is not running. Please start Docker Desktop."
        return $false
    }
}

function Start-DockerCompose {
    Write-Header "Starting Docker Compose Test Environment"

    Push-Location $ProjectRoot
    try {
        docker-compose -f docker-compose.test.yml down -v 2>$null
        docker-compose -f docker-compose.test.yml up -d

        Write-Host "Waiting for services to be healthy..." -ForegroundColor Yellow
        Start-Sleep -Seconds 10

        # Check health
        $maxRetries = 30
        $retry = 0
        while ($retry -lt $maxRetries) {
            try {
                $response = Invoke-WebRequest -Uri "http://localhost:3001/health" -TimeoutSec 2 -ErrorAction SilentlyContinue
                if ($response.StatusCode -eq 200) {
                    Write-Host "Backend is healthy!" -ForegroundColor Green
                    break
                }
            }
            catch {
                # Ignore errors
            }
            $retry++
            Write-Host "Waiting for backend... ($retry/$maxRetries)" -ForegroundColor Yellow
            Start-Sleep -Seconds 2
        }

        if ($retry -eq $maxRetries) {
            Write-Error "Backend failed to start within timeout"
            return $false
        }

        return $true
    }
    finally {
        Pop-Location
    }
}

function Stop-DockerCompose {
    Write-Header "Stopping Docker Compose Test Environment"

    Push-Location $ProjectRoot
    try {
        docker-compose -f docker-compose.test.yml down -v --remove-orphans
        Write-Host "Test environment stopped and cleaned up" -ForegroundColor Green
    }
    finally {
        Pop-Location
    }
}

function Run-TestcontainersTests {
    Write-Header "Running Tests with Testcontainers"

    Push-Location $BackendDir
    try {
        $env:RUST_LOG = if ($Verbose) { "debug" } else { "info" }
        $env:RUST_BACKTRACE = "1"

        $testArgs = @("test", "--test", "docker_tests")
        if ($TestFilter) {
            $testArgs += $TestFilter
        }
        if ($Verbose) {
            $testArgs += "-- --nocapture"
        }

        & cargo $testArgs
        return $LASTEXITCODE -eq 0
    }
    finally {
        Pop-Location
    }
}

function Run-DockerComposeTests {
    Write-Header "Running Tests against Docker Compose Environment"

    Push-Location $BackendDir
    try {
        # Set environment for docker-compose tests
        $env:DATABASE_URL = "postgres://test_user:test_password@localhost:5433/qrcode_share_test"
        $env:RUST_LOG = if ($Verbose) { "debug" } else { "info" }
        $env:RUST_BACKTRACE = "1"

        $testArgs = @("test", "--test", "integration_tests", "--test", "websocket_tests")
        if ($TestFilter) {
            $testArgs += $TestFilter
        }
        if ($Verbose) {
            $testArgs += "-- --nocapture"
        }

        & cargo $testArgs
        return $LASTEXITCODE -eq 0
    }
    finally {
        Pop-Location
    }
}

# Main execution
Write-Header "QRcode Share Integration Tests"
Write-Host "Mode: $Mode" -ForegroundColor White
Write-Host "Project Root: $ProjectRoot" -ForegroundColor White

if (-not (Test-DockerAvailable)) {
    exit 1
}

$success = $false

try {
    if ($Mode -eq "docker-compose") {
        if (-not (Start-DockerCompose)) {
            throw "Failed to start Docker Compose environment"
        }

        $success = Run-DockerComposeTests
    }
    else {
        $success = Run-TestcontainersTests
    }

    if ($success) {
        Write-Header "All Tests Passed!"
        Write-Host "Status: SUCCESS" -ForegroundColor Green
    }
    else {
        Write-Header "Some Tests Failed"
        Write-Host "Status: FAILURE" -ForegroundColor Red
    }
}
catch {
    Write-Error $_.Exception.Message
    $success = $false
}
finally {
    if ($Mode -eq "docker-compose" -and -not $NoCleanup) {
        Stop-DockerCompose
    }
}

exit if ($success) { 0 } else { 1 }
