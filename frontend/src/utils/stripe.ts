export function mapCountryToPlatform(country: string): 'US' | 'ES' {
  return (country || '').toUpperCase() === 'US' ? 'US' : 'ES';
}
