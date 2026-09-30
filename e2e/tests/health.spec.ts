import { expect, test } from '@playwright/test';

test('la aplicación arranca con la base al día', async ({ page }) => {
  const response = await page.goto('/health');

  expect(response?.status()).toBe(200);
  expect(response?.headers()['x-request-id']).toBeTruthy();
  expect(await response?.json()).toMatchObject({ status: 'ok', db: 'ok' });
});
