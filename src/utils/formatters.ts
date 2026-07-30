export function formatGps(lat: number, lon: number): string {
  const latDir = lat >= 0 ? 'N' : 'S'
  const lonDir = lon >= 0 ? 'E' : 'W'
  return `${Math.abs(lat).toFixed(6)}° ${latDir}, ${Math.abs(lon).toFixed(6)}° ${lonDir}`
}

/**
 * Format a datetime string to consistent YYYY-MM-DD HH:MM:SS format.
 * Handles EXIF datetime format (2024:07:26 14:30:00) and ISO strings.
 */
export function formatDateTime(value: string | null): string {
  if (!value) return ''
  // EXIF datetime uses colons in date part: "2024:07:26 14:30:00"
  // Normalize to "2024-07-26 14:30:00"
  const normalized = value.replace(/^(\d{4}):(\d{2}):(\d{2})/, '$1-$2-$3')
  return normalized
}

/**
 * Format shutter speed value to "1/125" style string.
 * - If already contains "/", return as-is (e.g. "1/500")
 * - If numeric (e.g. "0.008"), convert to "1/125" (1/value)
 * - If integer >= 1 (e.g. "2"), return as-is (seconds)
 */
export function formatShutterSpeed(value: string | number | null): string {
  if (value == null) return ''
  const str = String(value).trim()
  if (!str) return ''
  // Already in fraction format
  if (str.includes('/')) return str
  const num = parseFloat(str)
  if (isNaN(num)) return str
  // Integer seconds (e.g. "2", "30")
  if (num >= 1) return str
  // Decimal seconds -> convert to "1/N" fraction
  const denom = Math.round(1 / num)
  return `1/${denom}`
}

