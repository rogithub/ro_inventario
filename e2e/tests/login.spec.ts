import { expect, test, type Page } from '@playwright/test';

// Sin la sesión guardada: estas pruebas empiezan sin haber entrado.
test.use({ storageState: { cookies: [], origins: [] } });

async function entrar(page: Page, email: string, contrasena: string) {
  await page.getByLabel('Email').fill(email);
  await page.getByLabel('Contraseña').fill(contrasena);
  await page.getByRole('button', { name: 'Entrar' }).click();
}

// Email que no existe, distinto en cada corrida: los fallos no bloquean al usuario de pruebas.
const emailInventado = (proyecto: string, para: string) => `${para}-${proyecto}-${Date.now()}@e2e.local`;

test('sin sesión, una pantalla lleva a entrar y después regresa a ella', async ({ page }) => {
  await page.goto('/unidades');
  await expect(page).toHaveURL(/\/login\?siguiente=%2Funidades$/);

  await entrar(page, process.env.E2E_USER!, process.env.E2E_PASS!);

  await expect(page).toHaveURL(/\/unidades$/);
  await expect(page.getByRole('heading', { name: 'Unidades de medida' })).toBeVisible();
});

test('una contraseña equivocada dice por qué y no entra', async ({ page }, testInfo) => {
  await page.goto('/login');
  await entrar(page, emailInventado(testInfo.project.name, 'mala'), 'no-es-la-contrasena');

  await expect(page.getByRole('alert')).toHaveText('Email o contraseña incorrectos.');
  await expect(page).toHaveURL(/\/login$/);
});

test('después de 5 intentos equivocados pide esperar', async ({ page }, testInfo) => {
  const email = emailInventado(testInfo.project.name, 'bloqueo');
  await page.goto('/login');
  for (let intento = 0; intento < 6; intento++) {
    await entrar(page, email, 'no-es-la-contrasena');
  }
  await expect(page.getByRole('alert')).toContainText('Demasiados intentos.');
});

test('salir cierra la sesión', async ({ page }) => {
  await page.goto('/login');
  await entrar(page, process.env.E2E_USER!, process.env.E2E_PASS!);
  await expect(page).toHaveURL(/\/unidades$/);

  await page.getByRole('button', { name: 'Salir' }).click();
  await expect(page).toHaveURL(/\/login$/);

  await page.goto('/unidades');
  await expect(page).toHaveURL(/\/login\?siguiente=/);
});

test('captura de la pantalla de entrar para revisarla', async ({ page }, testInfo) => {
  await page.goto('/login');
  await entrar(page, emailInventado(testInfo.project.name, 'captura'), 'no-es-la-contrasena');
  await expect(page.getByRole('alert')).toBeVisible();
  await page.screenshot({ path: `capturas/${testInfo.project.name}/login.png`, fullPage: true });
});
