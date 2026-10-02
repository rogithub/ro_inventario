import { expect, test } from '@playwright/test';
import { addCategoria, newNombre } from './support';

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

test('renombrar una categoría cambia su nombre en la lista', async ({ page }, testInfo) => {
  const nombre = newNombre(testInfo.project.name);
  const renamed = `${nombre} renombrada`;
  await addCategoria(page, nombre);

  await page.getByRole('link', { name: `Renombrar «${nombre}»` }).click();
  await expect(page.getByLabel(`Nuevo nombre de «${nombre}»`)).toBeFocused();
  await page.getByLabel(`Nuevo nombre de «${nombre}»`).fill(renamed);
  await page.getByRole('button', { name: 'Guardar' }).click();

  await expect(page.getByRole('cell', { name: renamed, exact: true })).toBeVisible();
  await expect(page.getByRole('cell', { name: nombre, exact: true })).toHaveCount(0);
});

test('cancelar el renombre deja el nombre como estaba', async ({ page }, testInfo) => {
  const nombre = newNombre(testInfo.project.name);
  await addCategoria(page, nombre);

  await page.getByRole('link', { name: `Renombrar «${nombre}»` }).click();
  await page.getByLabel(`Nuevo nombre de «${nombre}»`).fill('otro nombre');
  await page.getByRole('link', { name: 'Cancelar' }).click();

  await expect(page.getByRole('cell', { name: nombre, exact: true })).toBeVisible();
  await expect(page.getByLabel(`Nuevo nombre de «${nombre}»`)).toHaveCount(0);
});

test('renombrar a un nombre que ya existe avisa en el renglón', async ({ page }, testInfo) => {
  const nombre = newNombre(testInfo.project.name);
  const otro = `${nombre} otra`;
  await addCategoria(page, nombre);
  await addCategoria(page, otro);

  await page.getByRole('link', { name: `Renombrar «${otro}»` }).click();
  await page.getByLabel(`Nuevo nombre de «${otro}»`).fill(nombre.toUpperCase());
  await page.getByRole('button', { name: 'Guardar' }).click();

  await expect(page.getByText(`Ya existe la categoría «${nombre.toUpperCase()}».`)).toBeVisible();
  await expect(page.getByLabel(`Nuevo nombre de «${otro}»`)).toHaveValue(nombre.toUpperCase());
});

test('captura del renglón en modo renombrar', async ({ page }, testInfo) => {
  const nombre = newNombre(testInfo.project.name);
  await addCategoria(page, nombre);
  await page.getByRole('link', { name: `Renombrar «${nombre}»` }).click();
  await page.getByLabel(`Nuevo nombre de «${nombre}»`).fill('');
  await page.getByRole('button', { name: 'Guardar' }).click();
  await expect(page.getByText('Escribe el nombre de la categoría.')).toBeVisible();
  await page.screenshot({ path: `capturas/${testInfo.project.name}/categorias-renombrar.png`, fullPage: true });
});

test('captura de la pantalla para revisarla', async ({ page }, testInfo) => {
  await page.getByLabel('Nombre').fill('   ');
  await page.getByRole('button', { name: 'Agregar categoría' }).click();
  await expect(page.getByText('Escribe el nombre de la categoría.')).toBeVisible();
  await page.screenshot({ path: `capturas/${testInfo.project.name}/categorias.png`, fullPage: true });
});
