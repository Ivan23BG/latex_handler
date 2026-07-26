#STREAMING_CHUNK:Configuring fail-safes and variables...

#Stop script execution upon encountering an error

$ErrorActionPreference = "Stop"

#TODO: Replace with your actual GitHub username and repository name

$Repo = "Ivan23BG/latex_handler"

#We expect the GitHub Release to have a file named 'latex_handler-windows.exe'

$BinaryUrl = "https://github.com/$Repo/releases/latest/download/latex_handler-windows.exe"

#STREAMING_CHUNK:Setting up local directories...

#Using the standard Local AppData folder for the binary

$InstallDir = "$env:LOCALAPPDATA\latex_handler\bin"
$ExePath = "$InstallDir\latex_handler.exe"

Write-Host "🚀 Installing latex_handler..." -ForegroundColor Cyan

#Create the installation directory if it doesn't exist

if (-not (Test-Path -Path $InstallDir)) {
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

#STREAMING_CHUNK:Downloading and installing binary...

Write-Host "📥 Downloading the latest release from GitHub..."
try {
Invoke-WebRequest -Uri $BinaryUrl -OutFile$ExePath
} catch {
Write-Host "❌ Failed to download the binary. Please check the URL and ensure a release exists." -ForegroundColor Red
exit
}

Write-Host "✅ Binary installed to $ExePath" -ForegroundColor Green

#STREAMING_CHUNK:Verifying and updating system PATH...

#Check if the installation directory is in the User's PATH environment variable

$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notmatch [regex]::Escape($InstallDir)) {
Write-Host "🔧 Adding $InstallDir to your PATH..." -ForegroundColor Yellow
$NewPath = "$UserPath;$InstallDir"
[Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
# Also update the current session's path so we can run it immediately
$env:PATH = "$env:PATH;$InstallDir"
}

#STREAMING_CHUNK:Running initial setup...

Write-Host "🔄 Running initial setup to fetch LaTeX templates..." -ForegroundColor Cyan
& $ExePath update

Write-Host "🎉 Installation complete! You can now use 'latex_handler' in your terminal." -ForegroundColor Green
Write-Host "⚠️  Note: You may need to restart your current terminal window for the PATH changes to take full effect." -ForegroundColor Yellow

# one liner: irm [https://raw.githubusercontent.com/YourUsername/YourRepo/main/install.ps1](https://raw.githubusercontent.com/YourUsername/YourRepo/main/install.ps1) | iex