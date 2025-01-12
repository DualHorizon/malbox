# Install Chocolatey
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))

# Install analysis tools
choco install -y `
    procmon `
    processhacker `
    wireshark `
    pestudio `
    x64dbg `
    python3 `
    sysinternals
