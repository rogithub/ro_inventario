import { expect, test } from '@playwright/test';

// La búsqueda con productos se prueba al tener el alta en pantalla (parte 7b2); aquí, lo que no
// depende de que existan.

test.beforeEach(async ({ page }) => {
  await page.goto('/productos');
});

test('se llega a productos desde el menú', async ({ page }) => {
  await page.goto('/unidades');
  await page.getByRole('link', { name: 'Productos' }).click();

  await expect(page).toHaveURL(/\/productos$/);
  await expect(page.getByRole('heading', { name: 'Productos' })).toBeVisible();
  await expect(page.getByLabel('Buscar por nombre o NID')).toBeFocused();
});

test('buscar mientras se escribe deja la búsqueda en la dirección', async ({ page }) => {
  const nombre = `no existe ${Date.now()}`;
  await page.getByLabel('Buscar por nombre o NID').fill(nombre);

  await expect(page.getByText('Ningún producto coincide con la búsqueda.')).toBeVisible();
  await expect(page).toHaveURL(/[?&]q=no(%20|\+)existe/);

  await page.reload();
  await expect(page.getByLabel('Buscar por nombre o NID')).toHaveValue(nombre);
  await expect(page.getByText('Ningún producto coincide con la búsqueda.')).toBeVisible();
});

test('captura de la pantalla para revisarla', async ({ page }, testInfo) => {
  await expect(page.getByRole('heading', { name: 'Productos' })).toBeVisible();
  await page.screenshot({ path: `capturas/${testInfo.project.name}/productos.png`, fullPage: true });
});
