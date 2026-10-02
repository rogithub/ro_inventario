import { expect, type Page } from '@playwright/test';

// Ayudantes compartidos por los E2E. La base de desarrollo se comparte entre corridas y entre los
// dos navegadores: cada dato lleva un sello que no existe (el proyecto y la hora en milisegundos).

export const stamp = (project: string) => `${project} ${Date.now()}`;

/** Un nombre de catálogo que no existe: «E2E <proyecto> <milisegundos>». */
export const newNombre = (project: string) => `E2E ${stamp(project)}`;

/** La agrega desde /categorias y espera a verla en la lista. */
export async function addCategoria(page: Page, nombre: string) {
  await page.goto('/categorias');
  await page.getByLabel('Nombre', { exact: true }).fill(nombre);
  await page.getByRole('button', { name: 'Agregar categoría' }).click();
  await expect(page.getByRole('cell', { name: nombre, exact: true })).toBeVisible();
}

/** Lo da de alta desde /productos/nuevo y regresa el NID del aviso. */
export async function addProducto(page: Page, nombre: string, categoria: string): Promise<string> {
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
