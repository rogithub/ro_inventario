import { expect, test } from '@playwright/test';
import { addCategoria, addProducto, newNombre, stamp } from './support';

test('desde la lista se cambia el precio y al volver la lista muestra el nuevo', async ({ page }, testInfo) => {
  const categoria = newNombre(testInfo.project.name);
  const nombre = `PLUMA ${stamp(testInfo.project.name)}`;
  await addCategoria(page, categoria);
  const nid = await addProducto(page, nombre, categoria);
  await page.goto('/productos');
  await page.getByLabel('Buscar por nombre o NID').fill(nombre);
  await expect(page).toHaveURL(/[?&]q=/);

  await page.getByRole('link', { name: nombre }).click();
  await expect(page).toHaveURL(new RegExp(`/productos/${nid}\\?q=`));
  await expect(page.getByRole('heading', { name: `Editar NID ${nid}` })).toBeVisible();
  await expect(page.getByLabel('Nombre')).toHaveValue(nombre);
  await page.getByLabel('Precio de venta').fill('15');
  await page.getByRole('button', { name: 'Guardar producto' }).click();
  await expect(page.getByRole('status')).toHaveText(`Se guardaron los cambios del NID ${nid}.`);
  await expect(page.getByLabel('Precio de venta')).toHaveValue('15.00');

  await page.getByRole('link', { name: 'Volver a productos' }).click();
  await expect(page.getByLabel('Buscar por nombre o NID')).toHaveValue(nombre);
  await expect(page.locator('#results tbody tr')).toHaveCount(1);
  await expect(page.locator('#results tbody tr')).toContainText('$15.00');
});

test('captura de la edición con un error en el precio', async ({ page }, testInfo) => {
  const categoria = newNombre(testInfo.project.name);
  await addCategoria(page, categoria);
  const nid = await addProducto(page, `PLUMA AZUL ${stamp(testInfo.project.name)}`, categoria);
  await page.goto(`/productos/${nid}`);
  await expect(page.getByLabel('Nombre')).not.toBeFocused();
  await page.getByLabel('Precio de venta').fill('12');
  await page.getByRole('button', { name: 'Guardar producto' }).click();
  await page.getByLabel('Precio de venta').fill('');
  await page.getByRole('button', { name: 'Guardar producto' }).click();

  await expect(page.getByText('Un producto con precio no puede quedar sin precio')).toBeVisible();
  await page.screenshot({ path: `capturas/${testInfo.project.name}/productos-editar.png`, fullPage: true });
});
