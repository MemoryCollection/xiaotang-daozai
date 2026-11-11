param(
    [Parameter(Mandatory = $true)][string]$Version,
    [Parameter(Mandatory = $true)][string]$Output,
    [string]$Changelog = 'CHANGELOG.md'
)

# Extract the section for the current tag from CHANGELOG.md and turn it into a
# plain-text release body. The in-app updater renders the GitHub Release body
# line by line (no Markdown), so strip the inline markup that would show up as
# literal asterisks or backticks there.
$ErrorActionPreference = 'Stop'

$tag = $Version
if (-not $tag.StartsWith('v')) { $tag = "v$tag" }

if (-not (Test-Path -LiteralPath $Changelog)) { throw "Changelog not found: $Changelog" }
$raw = Get-Content -LiteralPath $Changelog -Raw -Encoding UTF8
$lines = @($raw -split "\r?\n")

$start = -1
$end = $lines.Count
for ($i = 0; $i -lt $lines.Count; $i++) {
    if ($lines[$i] -match '^##\s+(v?\d+\.\d+\.\d+)\s*$') {
        $found = $matches[1]
        if ($start -ge 0) { $end = $i; break }
        if ($found -eq $tag -or $found -eq $tag.TrimStart('v')) { $start = $i }
    }
}
if ($start -lt 0) { throw "Missing CHANGELOG.md section for $tag" }

$body = New-Object System.Collections.Generic.List[string]
for ($i = $start; $i -lt $end; $i++) {
    $line = $lines[$i].TrimEnd()
    if ($i -eq $start) { $line = $line -replace '^##\s+', '# ' }
    $line = $line -replace '\*\*', ''
    $line = $line -replace '`', ''
    $body.Add($line)
}
while ($body.Count -gt 0 -and [string]::IsNullOrWhiteSpace($body[$body.Count - 1])) {
    $body.RemoveAt($body.Count - 1)
}
if ($body.Count -le 1) { throw "CHANGELOG.md section for $tag is empty" }

$text = ($body -join "`n") + "`n"
$full = $Output
if (-not [System.IO.Path]::IsPathRooted($full)) { $full = Join-Path (Get-Location).Path $full }
$dir = Split-Path -Parent $full
if ($dir -and -not (Test-Path -LiteralPath $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
$utf8 = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText($full, $text, $utf8)
Write-Host "Release notes for $tag written to $full ($($text.Length) chars)"
