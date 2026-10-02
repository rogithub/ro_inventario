import { expect, type Page, test } from '@playwright/test';

// La base de desarrollo se comparte entre corridas y entre los dos navegadores: cada prueba usa
// una categoría y un nombre que no existen.
const stamp = (project: string) => `${project} ${Date.now()}`;

async function addCategoria(page: Page, nombre: string) {
  await page.goto('/categorias');
  await page.getByLabel('Nombre', { exact: true }).fill(nombre);
  await page.getByRole('button', { name: 'Agregar categoría' }).click();
  await expect(page.getByRole('cell', { name: nombre, exact: true })).toBeVisible();
}

test('se llega al alta desde la lista de productos', async ({ page }) => {
  await page.goto('/productos');
  await page.getByRole('link', { name: 'Nuevo producto' }).click();

  await expect(page).toHaveURL(/\/productos\/nuevo$/);
  await expect(page.getByRole('heading', { name: 'Nuevo producto' })).toBeVisible();
  await expect(page.getByLabel('Nombre')).toBeFocused();
  await expect(page.getByLabel('Unidad')).toHaveText(/Pieza/);
});

test('agregar un producto deja el formulario vacío con el aviso del NID y la misma categoría', async ({ page }, testInfo) => {
  const categoria = `E2E ${stamp(testInfo.project.name)}`;
  const nombre = `PLUMA ${stamp(testInfo.project.name)}`;
  await addCategoria(page, categoria);
  await page.goto('/productos/nuevo');

  await page.getByLabel('Nombre').fill(nombre);
  await page.getByLabel('Categoría').selectOption({ label: categoria });
  await page.getByLabel('Precio de venta').fill('12.5');
  await page.getByRole('button', { name: 'Guardar producto' }).click();

  await expect(page.getByRole('status')).toHaveText(new RegExp(`Se agregó el NID \\d+ «${nombre}»\\.`));
  await expect(page.getByLabel('Nombre')).toHaveValue('');
  await expect(page.getByLabel('Nombre')).toBeFocused();
  await expect(page.getByLabel('Precio de venta')).toHaveValue('');
  await expect(page.getByLabel('Categoría').locator('option:checked')).toHaveText(categoria);
});

test('captura del formulario con más datos abierto y un error', async ({ page }, testInfo) => {
  await page.goto('/productos/nuevo');
  await page.getByLabel('Nombre').fill('PLUMA AZUL PUNTO MEDIANO');
  await page.getByLabel('Precio de venta').fill('12.345');
  await page.getByText('Más datos').click();
  await page.getByLabel('Marca').fill('Bic');
  await page.getByRole('button', { name: 'Guardar producto' }).click();

  await expect(page.getByText('Elige la categoría.')).toBeVisible();
  await expect(page.getByLabel('Marca')).toHaveValue('Bic');
  await page.screenshot({ path: `capturas/${testInfo.project.name}/productos-nuevo.png`, fullPage: true });
});
