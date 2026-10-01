import { expect, test } from '@playwright/test';

// La base de desarrollo se comparte entre corridas y entre los dos navegadores: cada prueba usa
// un nombre que no existe.
const newNombre = (project: string) => `E2E ${project} ${Date.now()}`;

test.beforeEach(async ({ page }) => {
  await page.goto('/categorias');
});

test('se llega a categorías desde el menú', async ({ page }) => {
  await page.goto('/unidades');
  await page.getByRole('link', { name: 'Categorías' }).click();

  await expect(page).toHaveURL(/\/categorias$/);
  await expect(page.getByRole('heading', { name: 'Categorías' })).toBeVisible();
});

test('agregar una categoría la muestra en la lista y deja el formulario listo para otra', async ({ page }, testInfo) => {
  const nombre = newNombre(testInfo.project.name);

  await page.getByLabel('Nombre').fill(nombre);
  await page.getByRole('button', { name: 'Agregar categoría' }).click();

  await expect(page.getByRole('cell', { name: nombre, exact: true })).toBeVisible();
  await expect(page.getByLabel('Nombre')).toHaveValue('');
  await expect(page.getByLabel('Nombre')).toBeFocused();
});

test('un nombre vacío no se agrega y dice por qué', async ({ page }) => {
  const response = page.waitForResponse((r) => r.url().endsWith('/categorias') && r.request().method() === 'POST');
  await page.getByLabel('Nombre').fill('   ');
  await page.getByRole('button', { name: 'Agregar categoría' }).click();

  expect((await response).status()).toBe(422);
  await expect(page.getByText('Escribe el nombre de la categoría.')).toBeVisible();
});

test('un nombre repetido no se agrega y conserva lo escrito', async ({ page }, testInfo) => {
  const nombre = newNombre(testInfo.project.name);
  await page.getByLabel('Nombre').fill(nombre);
  await page.getByRole('button', { name: 'Agregar categoría' }).click();
  await expect(page.getByRole('cell', { name: nombre, exact: true })).toBeVisible();

  await page.getByLabel('Nombre').fill(nombre.toLowerCase());
  await page.getByRole('button', { name: 'Agregar categoría' }).click();

  await expect(page.getByText(`Ya existe la categoría «${nombre.toLowerCase()}».`)).toBeVisible();
  await expect(page.getByLabel('Nombre')).toHaveValue(nombre.toLowerCase());
});

test('captura de la pantalla para revisarla', async ({ page }, testInfo) => {
  await page.getByLabel('Nombre').fill('   ');
  await page.getByRole('button', { name: 'Agregar categoría' }).click();
  await expect(page.getByText('Escribe el nombre de la categoría.')).toBeVisible();
  await page.screenshot({ path: `capturas/${testInfo.project.name}/categorias.png`, fullPage: true });
});
