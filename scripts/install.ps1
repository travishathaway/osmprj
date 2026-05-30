<#
.SYNOPSIS
    osmprj install script.
.DESCRIPTION
    This script is used to install osmprj on Windows from the command line.
.PARAMETER OsmprjVersion
    Specifies the version of osmprj to install.
    The default value is 'latest'. You can also specify it by setting the
    environment variable 'OSMPRJ_VERSION'.
.PARAMETER OsmprjHome
    Specifies osmprj's home directory.
    The default value is '$Env:USERPROFILE\.local\osmprj'. You can also specify
    it by setting the environment variable 'OSMPRJ_INSTALL_DIR'.
.PARAMETER NoPathUpdate
    If specified, the script will not update the PATH environment variable.
    You can also set the environment variable 'OSMPRJ_NO_PATH_UPDATE'.
.PARAMETER OsmprjRepourl
    Specifies osmprj's repo url.
    The default value is 'https://github.com/travishathaway/osmprj'. You can
    also specify it by setting the environment variable 'OSMPRJ_REPOURL'.
.LINK
    https://github.com/travishathaway/osmprj
#>
param (
    [string] $OsmprjVersion = 'latest',
    [string] $OsmprjHome    = "$Env:USERPROFILE\.local\osmprj",
    [switch] $NoPathUpdate,
    [string] $OsmprjRepourl = 'https://github.com/travishathaway/osmprj'
)

Set-StrictMode -Version Latest

function Mask-Credentials {
    param(
        [string] $Url
    )
    return $Url -replace '://[^:@/]+:[^@/]+@', '://***:***@'
}

function Publish-Env {
    if (-not ("Win32.NativeMethods" -as [Type])) {
        Add-Type -Namespace Win32 -Name NativeMethods -MemberDefinition @"
[DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
public static extern IntPtr SendMessageTimeout(
    IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
    uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
"@
    }

    $HWND_BROADCAST = [IntPtr] 0xffff
    $WM_SETTINGCHANGE = 0x1a
    $result = [UIntPtr]::Zero

    [Win32.Nativemethods]::SendMessageTimeout($HWND_BROADCAST,
        $WM_SETTINGCHANGE,
        [UIntPtr]::Zero,
        "Environment",
        2,
        5000,
        [ref] $result
    ) | Out-Null
}

function Write-Env {
    param(
        [String] $name,
        [String] $val,
        [Switch] $global
    )

    $RegisterKey = if ($global) {
        Get-Item -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager'
    } else {
        Get-Item -Path 'HKCU:'
    }

    $EnvRegisterKey = $RegisterKey.OpenSubKey('Environment', $true)
    if ($null -eq $val) {
        $EnvRegisterKey.DeleteValue($name)
    } else {
        $RegistryValueKind = if ($val.Contains('%')) {
            [Microsoft.Win32.RegistryValueKind]::ExpandString
        } elseif ($EnvRegisterKey.GetValue($name)) {
            $EnvRegisterKey.GetValueKind($name)
        } else {
            [Microsoft.Win32.RegistryValueKind]::String
        }
        $EnvRegisterKey.SetValue($name, $val, $RegistryValueKind)
    }
    Publish-Env
}

function Get-Env {
    param(
        [String] $name,
        [Switch] $global
    )

    $RegisterKey = if ($global) {
        Get-Item -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager'
    } else {
        Get-Item -Path 'HKCU:'
    }

    $EnvRegisterKey = $RegisterKey.OpenSubKey('Environment')
    $RegistryValueOption = [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames
    $EnvRegisterKey.GetValue($name, $null, $RegistryValueOption)
}

# Apply environment variable overrides
if ($Env:OSMPRJ_VERSION) {
    $OsmprjVersion = $Env:OSMPRJ_VERSION
}

if ($Env:OSMPRJ_INSTALL_DIR) {
    $OsmprjHome = $Env:OSMPRJ_INSTALL_DIR
}

if ($Env:OSMPRJ_NO_PATH_UPDATE) {
    $NoPathUpdate = $true
}

if ($Env:OSMPRJ_REPOURL) {
    $OsmprjRepourl = $Env:OSMPRJ_REPOURL -replace '/$', ''
}

$InstallerName = "osmprj-win-64-installer.ps1"

if ($Env:OSMPRJ_DOWNLOAD_URL) {
    $DownloadUrl = $Env:OSMPRJ_DOWNLOAD_URL
} elseif ($OsmprjVersion -eq 'latest') {
    $DownloadUrl = "$OsmprjRepourl/releases/latest/download/$InstallerName"
} else {
    $OsmprjVersion = "v" + ($OsmprjVersion -replace '^v', '')
    $DownloadUrl = "$OsmprjRepourl/releases/download/$OsmprjVersion/$InstallerName"
}

Write-Host "This script will automatically download and install osmprj ($OsmprjVersion) for you."
Write-Host "Downloading from: $(Mask-Credentials $DownloadUrl)"
Write-Host "osmprj will be installed to '$OsmprjHome'"

$TempFile = [System.IO.Path]::GetTempFileName() + ".ps1"
$webClient = New-Object System.Net.WebClient

try {
    $webClient.DownloadFile($DownloadUrl, $TempFile)
} catch {
    Write-Host "Error: '$(Mask-Credentials $DownloadUrl)' is not available or failed to download."
    exit 1
} finally {
    $webClient.Dispose()
}

if ((Get-Item $TempFile).Length -eq 0) {
    Write-Host "Error: downloaded file is empty. Check that the version exists and the URL is reachable."
    Remove-Item -Path $TempFile -ErrorAction SilentlyContinue
    exit 1
}

Write-Host "Installing osmprj to '$OsmprjHome'..."
& powershell -ExecutionPolicy Bypass -File $TempFile --output-directory $OsmprjHome
Remove-Item -Path $TempFile -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "osmprj has been installed to '$OsmprjHome'."
Write-Host ""

$BinDir    = Join-Path $OsmprjHome 'env\bin'
$ThemePath = Join-Path $OsmprjHome 'env\share\osmprj\themes\'

if (!$NoPathUpdate) {
    $PATH = Get-Env 'PATH'
    if ($PATH -notlike "*$BinDir*") {
        Write-Output "Adding $BinDir to PATH"
        Write-Env -name 'PATH' -val "$BinDir;$PATH"
        $Env:PATH = "$BinDir;$Env:PATH"
        Write-Output "You may need to restart your shell."
    } else {
        Write-Output "$BinDir is already in PATH"
    }
    Write-Env -name 'OSMPRJ_THEME_PATH' -val $ThemePath
    $Env:OSMPRJ_THEME_PATH = $ThemePath
} else {
    Write-Output "Skipping PATH update (OSMPRJ_NO_PATH_UPDATE is set)."
    Write-Output "Add '$BinDir' to your PATH manually."
}
