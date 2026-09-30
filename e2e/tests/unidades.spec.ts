import { expect, test } from '@playwright/test';

// La base de desarrollo se comparte entre corridas y entre los dos navegadores: cada prueba usa
// un nombre que no existe.
const newNombre = (project: string) => `E2E ${project} ${Date.now()}`;

test.beforeEach(async ({ page }) => {
  await page.goto('/unidades');
});

test('agregar una unidad la muestra en la lista y deja el formulario listo para otra', async ({ page }, testInfo) => {
  const nombre = newNombre(testInfo.project.name);

  await page.getByLabel('Nombre').fill(nombre);
  await page.getByLabel('Se vende en fracciones').check();
  await page.getByRole('button', { name: 'Agregar unidad' }).click();

  await expect(page.getByRole('row', { name: `${nombre} Sí` })).toBeVisible();
  await expect(page.getByLabel('Nombre')).toHaveValue('');
  await expect(page.getByLabel('Nombre')).toBeFocused();
  await expect(page.getByLabel('Se vende en fracciones')).not.toBeChecked();
});

test('un nombre vacío no se agrega y dice por qué', async ({ page }) => {
  // No se cuentan filas: el otro navegador puede estar agregando unidades al mismo tiempo.
  const response = page.waitForResponse((r) => r.url().endsWith('/unidades') && r.request().method() === 'POST');
  await page.getByLabel('Nombre').fill('   ');
  await page.getByRole('button', { name: 'Agregar unidad' }).click();

  expect((await response).status()).toBe(422);
  await expect(page.getByText('Escribe el nombre de la unidad.')).toBeVisible();
  for (const nombre of await page.locator('tbody td:first-child').allTextContents()) {
    expect(nombre.trim()).not.toBe('');
  }
});

test('un nombre repetido no se agrega y conserva lo escrito', async ({ page }) => {
  await page.getByLabel('Nombre').fill('pieza');
  await page.getByRole('button', { name: 'Agregar unidad' }).click();

  await expect(page.getByText('Ya existe la unidad «pieza».')).toBeVisible();
  await expect(page.getByLabel('Nombre')).toHaveValue('pieza');
});

test('captura de la pantalla para revisarla', async ({ page }, testInfo) => {
  await page.getByLabel('Nombre').fill('pieza');
  await page.getByRole('button', { name: 'Agregar unidad' }).click();
  await expect(page.getByText('Ya existe la unidad')).toBeVisible();
  await page.screenshot({ path: `capturas/${testInfo.project.name}/unidades.png`, fullPage: true });
});
