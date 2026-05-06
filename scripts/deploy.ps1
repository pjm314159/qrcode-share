#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Deploy QRcode Share Backend

.DESCRIPTION
    This script:
      1. Runs all tests
      2. Builds Docker image
      3. Pushes to registry (optional)
      4. Deploys with docker-compose

.PARAMETER Env
    Environment: production, staging (default: production)

.PARAMETER ImageName
    Docker image name (default: qrcode-share-backend)

.PARAMETER Tag
    Image tag (default: latest)

.PARAMETER NoPush
    Build only, don't push to registry

.EXAMPLE
    ./scripts/deploy.ps1
    Deploy to production

.EXAMPLE
    ./scripts/deploy.ps1 -Env staging -NoPush
    Build for staging without pushing
#>

param(
    [ValidateSet("production", "staging")]
    [string]$Env = "production",

    [string]$ImageName = "qrcode-share-backend",

    [string]$Tag = "latest",

    [switch]$NoPush
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$BackendDir = Join-Path $ProjectRoot "qrcode_share_backend"

$FullImage = "${ImageName}:${Tag}"

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

Write-Step "Deploy QRcode Share Backend"
Write-Info "Environment: $Env"
Write-Info "Image: $FullImage"

# Step 1: Run tests
Write-Step "Step 1: Run Tests"

Push-Location $BackendDir
try {
    cargo test --lib --test integration_tests --test websocket_tests 2>&1 | Select-Object -Last 5
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Tests passed"
    } else {
        Write-Fail "Tests failed! Aborting deployment."
        exit 1
    }
}
finally {
    Pop-Location
}

# Step 2: Build Docker image
Write-Step "Step 2: Build Docker Image"

Push-Location $BackendDir
try {
    docker build `
        --build-arg RUST_LOG=info `
        -t $FullImage `
        -t "${ImageName}:${Env}" `
        .

    if ($LASTEXITCODE -eq 0) {
        Write-Success "Docker image built: $FullImage"
    } else {
        Write-Fail "Docker build failed"
        exit 1
    }

    # Show image size
    $imageInfo = docker images $FullImage --format "{{.Size}}"
    Write-Info "Image size: $imageInfo"
}
finally {
    Pop-Location
}

# Step 3: Push to registry
if (-not $NoPush) {
    Write-Step "Step 3: Push to Registry"
    docker push $FullImage
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Image pushed: $FullImage"
    } else {
        Write-Fail "Push failed. Use -NoPush to skip."
        exit 1
    }
} else {
    Write-Step "Step 3: Push Skipped (-NoPush)"
}

# Step 4: Deploy
Write-Step "Step 4: Deploy with Docker Compose"

Push-Location $ProjectRoot
try {
    $composeFile = "docker-compose.yml"
    if ($Env -eq "staging") {
        $composeFile = "docker-compose.staging.yml"
    }

    if (Test-Path $composeFile) {
        # Pull latest images
        docker-compose -f $composeFile pull 2>$null

        # Start services
        docker-compose -f $composeFile up -d

        Write-Success "Services started"

        # Wait for health check
        Write-Info "Waiting for services to be healthy..."
        Start-Sleep -Seconds 5

        $maxRetries = 10
        $retry = 0
        while ($retry -lt $maxRetries) {
            try {
                $response = Invoke-WebRequest -Uri "http://localhost:3000/health" -TimeoutSec 2 -ErrorAction SilentlyContinue
                if ($response.StatusCode -eq 200) {
                    Write-Success "Backend is healthy"
                    break
                }
            }
            catch { }

            $retry++
            Start-Sleep -Seconds 2
        }

        if ($retry -eq $maxRetries) {
            Write-Fail "Backend health check failed"
            docker-compose -f $composeFile logs backend
            exit 1
        }
    } else {
        Write-Host "  No $composeFile found. Skipping docker-compose deploy." -ForegroundColor Yellow
        Write-Info "Run manually: docker-compose up -d"
    }
}
finally {
    Pop-Location
}

# Step 5: Summary
Write-Step "Deployment Summary"

Write-Success "Deployment complete!"
Write-Info "Environment: $Env"
Write-Info "Image: $FullImage"
Write-Info ""
Write-Info "Backend: http://localhost:3000"
Write-Info "Health:  http://localhost:3000/health"
Write-Info "Metrics: http://localhost:3000/metrics"
