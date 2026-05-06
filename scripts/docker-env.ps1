#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Manage Docker test environment for QRcode Share

.DESCRIPTION
    Start, stop, or check status of the Docker test environment.

.PARAMETER Action
    Action to perform: start, stop, status, logs, or restart

.EXAMPLE
    ./scripts/docker-env.ps1 start
    Start the test environment

.EXAMPLE
    ./scripts/docker-env.ps1 logs
    View logs from all services
#>

param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("start", "stop", "status", "logs", "restart")]
    [string]$Action
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot

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

switch ($Action) {
    "start" {
        Write-Header "Starting Docker Test Environment"

        if (-not (Test-DockerAvailable)) { exit 1 }

        Push-Location $ProjectRoot
        try {
            # Start services
            docker-compose -f docker-compose.test.yml up -d

            Write-Host "Waiting for services to be healthy..." -ForegroundColor Yellow
            Start-Sleep -Seconds 5

            # Wait for PostgreSQL
            $maxRetries = 30
            $retry = 0
            while ($retry -lt $maxRetries) {
                try {
                    docker exec qrcode_share_test_db pg_isready -U test_user -d qrcode_share_test 2>$null
                    if ($LASTEXITCODE -eq 0) {
                        Write-Host "PostgreSQL is ready!" -ForegroundColor Green
                        break
                    }
                }
                catch { }

                $retry++
                Write-Host "Waiting for PostgreSQL... ($retry/$maxRetries)" -ForegroundColor Yellow
                Start-Sleep -Seconds 2
            }

            # Wait for backend
            $retry = 0
            while ($retry -lt $maxRetries) {
                try {
                    $response = Invoke-WebRequest -Uri "http://localhost:3001/health" -TimeoutSec 2 -ErrorAction SilentlyContinue
                    if ($response.StatusCode -eq 200) {
                        Write-Host "Backend is healthy!" -ForegroundColor Green
                        break
                    }
                }
                catch { }

                $retry++
                Write-Host "Waiting for backend... ($retry/$maxRetries)" -ForegroundColor Yellow
                Start-Sleep -Seconds 2
            }

            Write-Host "`nTest environment is ready!" -ForegroundColor Green
            Write-Host "PostgreSQL: localhost:5433" -ForegroundColor White
            Write-Host "Backend:    localhost:3001" -ForegroundColor White
            Write-Host "Redis:      localhost:6380" -ForegroundColor White
        }
        finally {
            Pop-Location
        }
    }

    "stop" {
        Write-Header "Stopping Docker Test Environment"

        Push-Location $ProjectRoot
        try {
            docker-compose -f docker-compose.test.yml down -v --remove-orphans
            Write-Host "Test environment stopped and cleaned up" -ForegroundColor Green
        }
        finally {
            Pop-Location
        }
    }

    "status" {
        Write-Header "Docker Test Environment Status"

        Push-Location $ProjectRoot
        try {
            docker-compose -f docker-compose.test.yml ps

            Write-Host "`nService Health:" -ForegroundColor Yellow

            # Check PostgreSQL
            try {
                docker exec qrcode_share_test_db pg_isready -U test_user -d qrcode_share_test 2>$null
                Write-Host "  PostgreSQL: Healthy" -ForegroundColor Green
            }
            catch {
                Write-Host "  PostgreSQL: Not running" -ForegroundColor Red
            }

            # Check Backend
            try {
                $response = Invoke-WebRequest -Uri "http://localhost:3001/health" -TimeoutSec 2
                if ($response.StatusCode -eq 200) {
                    Write-Host "  Backend:    Healthy" -ForegroundColor Green
                }
            }
            catch {
                Write-Host "  Backend:    Not running" -ForegroundColor Red
            }

            # Check Redis
            try {
                docker exec qrcode_share_test_redis redis-cli ping 2>$null | Out-Null
                Write-Host "  Redis:      Healthy" -ForegroundColor Green
            }
            catch {
                Write-Host "  Redis:      Not running" -ForegroundColor Red
            }
        }
        finally {
            Pop-Location
        }
    }

    "logs" {
        Push-Location $ProjectRoot
        try {
            docker-compose -f docker-compose.test.yml logs -f
        }
        finally {
            Pop-Location
        }
    }

    "restart" {
        & $PSCommandPath -Action stop
        Start-Sleep -Seconds 2
        & $PSCommandPath -Action start
    }
}
