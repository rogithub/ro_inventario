import { expect, type Page, test } from '@playwright/test';
import { addCategoria, addProducto, stamp } from './support';

/** Busca en /productos y espera a que la dirección lleve la búsqueda. */
async function search(page: Page, text: string) {
  await page.goto('/productos');
  await page.getByLabel('Buscar por nombre o NID').fill(text);
  await expect(page).toHaveURL(/[?&]q=/);
}

const rows = (page: Page) => page.locator('#results tbody tr');

test('al descontinuar deja de salir en la lista y vuelve con ver descontinuados, en gris y con su insignia', async ({ page }, testInfo) => {
  const sello = stamp(testInfo.project.name);
  const categoria = `E2E ${sello}`;
  const nombre = `CARPETA ${sello}`;
  await addCategoria(page, categoria);
  const nid = await addProducto(page, nombre, categoria);

  await search(page, sello);
  await expect(rows(page)).toHaveCount(1);
  await rows(page).getByRole('link', { name: nombre }).click();
  await page.getByLabel('Motivo para descontinuarlo').fill('el proveedor ya no lo vende');
  await page.getByRole('button', { name: 'Descontinuar' }).click();
  await expect(page).toHaveURL(new RegExp(`/productos/${nid}\\?q=`));
  await expect(page.getByText(/^Descontinuado el \d{1,2} [a-z]{3} \d{4}: el proveedor ya no lo vende$/)).toBeVisible();
  await page.screenshot({ path: `capturas/${testInfo.project.name}/productos-descontinuado.png`, fullPage: true });

  await page.getByRole('link', { name: 'Volver a productos' }).click();
  await expect(page.getByLabel('Buscar por nombre o NID')).toHaveValue(sello);
  await expect(page.getByText('Ningún producto coincide con la búsqueda.')).toBeVisible();

  await page.getByLabel('Ver descontinuados').check();
  await expect(rows(page)).toHaveCount(1);
  await expect(rows(page).getByText('Descontinuado', { exact: true })).toBeVisible();
  await expect(rows(page).locator('td').first()).toHaveClass(/text-body-secondary/);
  await page.screenshot({ path: `capturas/${testInfo.project.name}/productos-con-descontinuados.png`, fullPage: true });

  await rows(page).getByRole('link', { name: nombre }).click();
  await page.getByRole('button', { name: 'Volver a manejar' }).click();
  await expect(page.getByText(/^Descontinuado el/)).toHaveCount(0);
  await expect(page.getByLabel('Motivo para descontinuarlo')).toBeVisible();

  await page.getByRole('link', { name: 'Volver a productos' }).click();
  await page.getByLabel('Ver descontinuados').uncheck();
  await expect(rows(page)).toHaveCount(1);
  await expect(rows(page).getByText('Descontinuado', { exact: true })).toHaveCount(0);
});

test('captura del motivo para descontinuar', async ({ page }, testInfo) => {
  const categoria = `E2E ${stamp(testInfo.project.name)}`;
  await addCategoria(page, categoria);
  const nid = await addProducto(page, `CARPETA ${stamp(testInfo.project.name)}`, categoria);
  await page.goto(`/productos/${nid}`);
  await expect(page.getByLabel('Motivo para descontinuarlo')).toHaveAttribute('placeholder', 'Ej.: el proveedor ya no lo vende');
  await page.screenshot({ path: `capturas/${testInfo.project.name}/productos-editar-descontinuar.png`, fullPage: true });
});
