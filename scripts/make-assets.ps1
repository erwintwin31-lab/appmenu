# Erzeugt App-Icon und Installer-Bilder (laeuft nur im Build auf GitHub)
Add-Type -AssemblyName System.Drawing
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot

function New-RoundPath([double]$x, [double]$y, [double]$w, [double]$h, [double]$r) {
  $p = New-Object System.Drawing.Drawing2D.GraphicsPath
  $d = $r * 2
  $p.AddArc($x, $y, $d, $d, 180, 90)
  $p.AddArc($x + $w - $d, $y, $d, $d, 270, 90)
  $p.AddArc($x + $w - $d, $y + $h - $d, $d, $d, 0, 90)
  $p.AddArc($x, $y + $h - $d, $d, $d, 90, 90)
  $p.CloseFigure()
  return ,$p
}

function New-Logo([int]$s) {
  $bmp = New-Object System.Drawing.Bitmap($s, $s, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.SmoothingMode = 'AntiAlias'
  $g.Clear([System.Drawing.Color]::Transparent)
  $u = $s / 28.0
  $bg = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 27, 34, 44))
  $g.FillPath($bg, (New-RoundPath 0 0 ($s - 1) ($s - 1) ($s * 0.22)))
  $acc = [System.Drawing.Color]::FromArgb(255, 96, 205, 255)
  $fill = New-Object System.Drawing.SolidBrush($acc)
  $pen = New-Object System.Drawing.Pen($acc, [single](1.5 * $u))
  $pen.StartCap = 'Round'
  $pen.EndCap = 'Round'
  $g.FillPath($fill, (New-RoundPath (5 * $u) (5 * $u) (8 * $u) (8 * $u) (2.2 * $u)))
  $g.DrawPath($pen, (New-RoundPath (15 * $u) (5 * $u) (8 * $u) (8 * $u) (2.2 * $u)))
  $g.DrawPath($pen, (New-RoundPath (5 * $u) (15 * $u) (8 * $u) (8 * $u) (2.2 * $u)))
  $g.DrawLine($pen, [single](15 * $u), [single](19 * $u), [single](23 * $u), [single](19 * $u))
  $g.DrawLine($pen, [single](19 * $u), [single](15 * $u), [single](19 * $u), [single](23 * $u))
  $g.Dispose()
  return ,$bmp
}

function Save-Grad([int]$w, [int]$h, $logo, [int]$lx, [int]$ly, [int]$ls, [string]$path) {
  $bmp = New-Object System.Drawing.Bitmap($w, $h, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $rect = New-Object System.Drawing.Rectangle(0, 0, $w, $h)
  $br = New-Object System.Drawing.Drawing2D.LinearGradientBrush($rect, [System.Drawing.Color]::FromArgb(22, 28, 37), [System.Drawing.Color]::FromArgb(32, 42, 67), 90.0)
  $g.FillRectangle($br, $rect)
  $g.InterpolationMode = 'HighQualityBicubic'
  $g.DrawImage($logo, $lx, $ly, $ls, $ls)
  $g.Dispose()
  $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Bmp)
  $bmp.Dispose()
}

$logo = New-Logo 512
$logo.Save((Join-Path $root 'src-tauri\app-icon.png'), [System.Drawing.Imaging.ImageFormat]::Png)
$inst = Join-Path $root 'src-tauri\installer'
New-Item -ItemType Directory -Force -Path $inst | Out-Null
Save-Grad 164 314 $logo 34 30 96 (Join-Path $inst 'sidebar.bmp')
Save-Grad 150 57 $logo 100 8 40 (Join-Path $inst 'header.bmp')
$logo.Dispose()
Write-Host 'Icons und Installer-Bilder erzeugt.'
