import { expect, type Page, test } from '@playwright/test';

// La base de desarrollo se comparte entre corridas y entre los dos navegadores: cada producto
// lleva un sello que no existe, y las búsquedas lo incluyen para encontrar solo los de la prueba.
const stamp = (project: string) => `${project} ${Date.now()}`;

async function addCategoria(page: Page, nombre: string) {
  await page.goto('/categorias');
  await page.getByLabel('Nombre', { exact: true }).fill(nombre);
  await page.getByRole('button', { name: 'Agregar categoría' }).click();
  await expect(page.getByRole('cell', { name: nombre, exact: true })).toBeVisible();
}

/** Lo da de alta desde la pantalla y regresa el NID del aviso. */
async function addProducto(page: Page, nombre: string, categoria: string): Promise<string> {
  await page.goto('/productos/nuevo');
  await page.getByLabel('Nombre').fill(nombre);
  await page.getByLabel('Categoría').selectOption({ label: categoria });
  await page.getByRole('button', { name: 'Guardar producto' }).click();
  const aviso = page.getByRole('status');
  await expect(aviso).toContainText(`«${nombre}»`);
  const nid = (await aviso.textContent())?.match(/NID (\d+)/)?.[1];
  expect(nid).toBeDefined();
  return nid ?? '';
}

/** Escribe en el buscador de /productos y espera a que la dirección lleve la búsqueda. */
async function search(page: Page, text: string) {
  await page.goto('/productos');
  await page.getByLabel('Buscar por nombre o NID').fill(text);
  await expect(page).toHaveURL(/[?&]q=/);
}

const rows = (page: Page) => page.locator('#results tbody tr');

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

test('buscar sin acentos encuentra los nombres con acento', async ({ page }, testInfo) => {
  const sello = stamp(testInfo.project.name);
  const categoria = `E2E ${sello}`;
  await addCategoria(page, categoria);
  await addProducto(page, `LÁPIZ ${sello}`, categoria);

  await search(page, `lapiz ${sello}`);

  await expect(rows(page)).toHaveCount(1);
  await expect(rows(page)).toContainText(`LÁPIZ ${sello}`);
});

test('buscar con n o con ñ encuentra los nombres con ñ', async ({ page }, testInfo) => {
  const sello = stamp(testInfo.project.name);
  const categoria = `E2E ${sello}`;
  await addCategoria(page, categoria);
  await addProducto(page, `PIÑA ${sello}`, categoria);

  for (const text of [`pina ${sello}`, `piña ${sello}`]) {
    await search(page, text);
    await expect(rows(page)).toHaveCount(1);
    await expect(rows(page)).toContainText(`PIÑA ${sello}`);
  }
});

test('buscar por NID pone ese producto primero, antes de los nombres con ese número', async ({ page }, testInfo) => {
  const sello = stamp(testInfo.project.name);
  const categoria = `E2E ${sello}`;
  await addCategoria(page, categoria);
  const nid = await addProducto(page, `ENGRAPADORA ${sello}`, categoria);
  // Coincide por nombre y va antes en orden alfabético: solo la regla del NID lo pone después.
  await addProducto(page, `AAA ${nid} ${sello}`, categoria);

  await search(page, nid);

  await expect(rows(page).first().getByRole('cell').first()).toHaveText(nid);
  await expect(rows(page).first()).toContainText(`ENGRAPADORA ${sello}`);
  await expect(rows(page).filter({ hasText: `AAA ${nid} ${sello}` })).toHaveCount(1);
});

test('el filtro por categoría solo trae los suyos', async ({ page }, testInfo) => {
  const sello = stamp(testInfo.project.name);
  const plumas = `E2E plumas ${sello}`;
  const cuadernos = `E2E cuadernos ${sello}`;
  await addCategoria(page, plumas);
  await addCategoria(page, cuadernos);
  await addProducto(page, `PLUMA ${sello}`, plumas);
  await addProducto(page, `CUADERNO ${sello}`, cuadernos);

  await search(page, sello);
  await expect(rows(page)).toHaveCount(2);
  await page.getByLabel('Categoría').selectOption({ label: plumas });

  await expect(page).toHaveURL(/[?&]categoria=[^&]/);
  await expect(rows(page)).toHaveCount(1);
  await expect(rows(page)).toContainText(`PLUMA ${sello}`);
});
