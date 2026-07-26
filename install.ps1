$ErrorActionPreference = 'Stop'

# Force TLS 1.2 for GitHub downloads on Windows PowerShell 5.1
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

# Disable UI progress bar to speed up Invoke-WebRequest downloads
$ProgressPreference = 'SilentlyContinue'

$Repo = "Ivan23BG/latex_handler"
$ArchiveUrl = "https://github.com/$Repo/releases/latest/download/latex_handler-windows.zip"

$InstallDir = "$env:LOCALAPPDATA\latex_handler\bin"
$ExePath = "$InstallDir\latex_handler.exe"

Write-Host "Installing latex_handler..." -ForegroundColor Cyan

# Create install directory if it doesn't exist
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$TempZip = "$env:TEMP\latex_handler-windows.zip"

Write-Host "Downloading latest release..." -ForegroundColor Yellow
Invoke-WebRequest -Uri $ArchiveUrl -OutFile $TempZip

Write-Host "Extracting binary..." -ForegroundColor Yellow
Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
Remove-Item $TempZip -Force

Write-Host "Binary installed to $ExePath" -ForegroundColor Green

# Add to User PATH if not already present
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "Adding $InstallDir to User PATH..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$InstallDir", "User")
    $env:PATH += ";$InstallDir"
}

Write-Host "Running initial setup to fetch LaTeX templates..." -ForegroundColor Cyan
& "$ExePath" update

Write-Host "Installation complete! Please restart your terminal to use 'latex_handler' globally." -ForegroundColor Green