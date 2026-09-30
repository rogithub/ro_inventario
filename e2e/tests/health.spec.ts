import { expect, test } from '@playwright/test';

test('la aplicación arranca con la base al día', async ({ page }) => {
  const respuesta = await page.goto('/health');

  expect(respuesta?.status()).toBe(200);
  expect(respuesta?.headers()['x-request-id']).toBeTruthy();
  expect(await respuesta?.json()).toMatchObject({ status: 'ok', db: 'ok' });
});
