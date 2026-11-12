param(
    [Parameter(Mandatory = $true)][string]$Title,
    [string]$Start,
    [string]$Deadline,
    [ValidateSet("todo", "doing", "done")][string]$Status = "todo"
)

$ErrorActionPreference = "Stop"
$dir = Join-Path $env:APPDATA "com.inzh.workcalendar"
New-Item -ItemType Directory -Force -Path $dir | Out-Null

$payload = @{ title = $Title; status = $Status }
if ($Start) { $payload.startAt = $Start }
if ($Deadline) { $payload.deadlineAt = $Deadline }
$json = $payload | ConvertTo-Json -Compress

$endpointPath = Join-Path $dir "endpoint.json"
if (Test-Path -LiteralPath $endpointPath) {
    $endpoint = Get-Content -LiteralPath $endpointPath -Raw -Encoding UTF8 | ConvertFrom-Json
    try {
        $response = Invoke-RestMethod `
            -Method Post `
            -Uri "$($endpoint.base)/tasks" `
            -Headers @{ Authorization = "Bearer $($endpoint.token)" } `
            -ContentType "application/json; charset=utf-8" `
            -Body ([System.Text.Encoding]::UTF8.GetBytes($json))
        $response | ConvertTo-Json -Depth 6
        exit 0
    } catch {
        Write-Host "Приложение не ответило, задача ляжет в очередь: $($_.Exception.Message)"
    }
}

$inbox = Join-Path $dir "inbox.jsonl"
Add-Content -LiteralPath $inbox -Value $json -Encoding UTF8
Write-Host "Записано в $inbox. Появится в календаре, когда приложение будет запущено."
