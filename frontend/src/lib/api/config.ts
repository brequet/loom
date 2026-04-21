import { get } from '$lib/api/client';
import type { AppConfig } from '$shared/AppConfig';

let cached: AppConfig | null = null;
let pending: Promise<AppConfig> | null = null;

export async function getAppConfig(): Promise<AppConfig> {
  if (cached) return cached;
  if (pending) return pending;
  pending = get<AppConfig>('/config').then((c) => {
    cached = c;
    pending = null;
    return c;
  });
  return pending;
}

export function invalidateAppConfig(): void {
  cached = null;
  pending = null;
}
