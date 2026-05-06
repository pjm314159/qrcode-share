#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Full integration test pipeline for QRcode Share Backend

.DESCRIPTION
    This script:
      1. Builds the project
      2. Starts PostgreSQL in Docker
      3. Runs database migrations
      4. Runs all tests (unit + integration + migration + docker)
      5. Cleans up Docker containers

.PARAMETER Verbose
    Show verbose output

.PARAMETER SkipBuild
    Skip build step

.PARAMETER KeepContainers
    Keep Docker containers after tests

.EXAMPLE
    ./scripts/test-full.ps1
    Run full test pipeline

.EXAMPLE
    ./scripts/test-full.ps1 -KeepContainers
    Keep containers running after tests
#>

param(
    [switch]$Verbose,
    [switch]$SkipBuild,
    [switch]$KeepContainers
)

$ErrorActionPreference = "Stop"

$ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$BackendDir = Join-Path $ProjectRoot "qrcode_share_backend"

$PostgresContainer = "qrcode_share_test_postgres"
$PostgresPort = 5433
$PostgresUser = "test_user"
$PostgresPassword = "test_password"
$PostgresDb = "qrcode_share_test"

function Write-Step {
    param([string]$Message)
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host "  $Message" -ForegroundColor Cyan
    Write-Host "========================================`n" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "  OK $Message" -ForegroundColor Green
}

function Write-Fail {
    param([string]$Message)
    Write-Host "  FAIL $Message" -ForegroundColor Red
}

function Write-Info {
    param([string]$Message)
    Write-Host "  $Message" -ForegroundColor White
}

# Check Docker
if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Fail "Docker is not installed or not in PATH"
    exit 1
}

try {
    docker info | Out-Null
}
catch {
    Write-Fail "Docker is not running. Please start Docker Desktop."
    exit 1
}

$testResult = 0

try {
    # Step 1: Build
    if (-not $SkipBuild) {
        Write-Step "Step 1: Build Project"

        Push-Location $BackendDir
        try {
            if ($Verbose) {
                cargo build 2>&1
            } else {
                cargo build 2>&1 | Select-Object -Last 5
            }

            if ($LASTEXITCODE -eq 0) {
                Write-Success "Build succeeded"
            } else {
                Write-Fail "Build failed"
                exit 1
            }
        }
        finally {
            Pop-Location
        }
    }

    # Step 2: Start PostgreSQL
    Write-Step "Step 2: Start PostgreSQL Container"

    # Remove existing container
    docker rm -f $PostgresContainer 2>$null | Out-Null

    # Start PostgreSQL
    docker run -d `
        --name $PostgresContainer `
        -e "POSTGRES_USER=$PostgresUser" `
        -e "POSTGRES_PASSWORD=$PostgresPassword" `
        -e "POSTGRES_DB=$PostgresDb" `
        -p "${PostgresPort}:5432" `
        postgres:16-alpine | Out-Null

    # Wait for PostgreSQL
    Write-Info "Waiting for PostgreSQL to be ready..."
    $maxRetries = 30
    $retry = 0
    while ($retry -lt $maxRetries) {
        try {
            $result = docker exec $PostgresContainer pg_isready -U $PostgresUser -d $PostgresDb 2>$null
            if ($LASTEXITCODE -eq 0) {
                Write-Success "PostgreSQL is ready"
                break
            }
        }
        catch { }

        $retry++
        Write-Info "Waiting... ($retry/$maxRetries)"
        Start-Sleep -Seconds 1
    }

    if ($retry -eq $maxRetries) {
        Write-Fail "PostgreSQL failed to start"
        docker logs $PostgresContainer
        exit 1
    }

    # Step 3: Run Migrations
    Write-Step "Step 3: Run Database Migrations"

    Push-Location $BackendDir
    try {
        $env:DATABASE_URL = "postgres://${PostgresUser}:${PostgresPassword}@localhost:${PostgresPort}/${PostgresDb}"

        # Check if sqlx-cli is installed
        $sqlxInstalled = Get-Command sqlx -ErrorAction SilentlyContinue
        if (-not $sqlxInstalled) {
            Write-Info "Installing sqlx-cli..."
            cargo install sqlx-cli --no-default-features --features postgres 2>&1 | Select-Object -Last 3
        }

        # Run migrations
        $migrationSource = Join-Path $ProjectRoot "migrations"
        sqlx migrate run --source $migrationSource

        if ($LASTEXITCODE -eq 0) {
            Write-Success "Migrations applied successfully"
        } else {
            Write-Fail "Migration failed"
            exit 1
        }

        # Verify migrations
        Write-Info "Verifying migration status..."
        sqlx migrate info --source $migrationSource
    }
    finally {
        Pop-Location
    }

    # Step 4: Run Tests
    Write-Step "Step 4: Run Tests"

    Push-Location $BackendDir
    try {
        $env:DATABASE_URL = "postgres://${PostgresUser}:${PostgresPassword}@localhost:${PostgresPort}/${PostgresDb}"
        $env:RUST_BACKTRACE = "1"
        $env:RUST_LOG = if ($Verbose) { "debug" } else { "info" }

        # 4a: Unit tests
        Write-Info "Running unit tests..."
        cargo test --lib 2>&1 | Select-Object -Last 5
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Unit tests passed"
        } else {
            Write-Fail "Unit tests failed"
            $testResult = 1
        }

        # 4b: Integration tests
        Write-Info "Running integration tests..."
        cargo test --test integration_tests --test websocket_tests 2>&1 | Select-Object -Last 10
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Integration tests passed"
        } else {
            Write-Fail "Integration tests failed"
            $testResult = 1
        }

        # 4c: Migration tests
        Write-Info "Running migration tests..."
        cargo test --test migration_tests 2>&1 | Select-Object -Last 10
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Migration tests passed"
        } else {
            Write-Fail "Migration tests failed"
            $testResult = 1
        }

        # 4d: Docker integration tests
        Write-Info "Running Docker integration tests..."
        cargo test --test docker_tests 2>&1 | Select-Object -Last 10
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Docker integration tests passed"
        } else {
            Write-Fail "Docker integration tests failed"
            $testResult = 1
        }
    }
    finally {
        Pop-Location
    }

    # Step 5: Summary
    Write-Step "Step 5: Test Summary"

    if ($testResult -eq 0) {
        Write-Success "All tests passed!"
        Write-Host ""
        Write-Host "  ALL TESTS PASSED" -ForegroundColor Green
    } else {
        Write-Fail "Some tests failed!"
        Write-Host ""
        Write-Host "  SOME TESTS FAILED" -ForegroundColor Red
    }
}
finally {
    # Cleanup
    if (-not $KeepContainers) {
        Write-Step "Step 6: Cleanup"
        docker rm -f $PostgresContainer 2>$null | Out-Null
        Write-Success "Docker containers removed"
    } else {
        Write-Host "  Keeping containers (PostgreSQL on localhost:$PostgresPort)" -ForegroundColor Yellow
    }
}

exit $testResult
