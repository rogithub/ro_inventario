import { expect, test as setup } from '@playwright/test';
import { SESSION_FILE } from '../playwright.config';

// Credenciales del usuario de pruebas: del perfil del dueño en kukulkan, aleatorias en CI.
// Nunca se imprimen.
setup('entrar con el usuario de pruebas', async ({ page }) => {
  const email = process.env.E2E_USER;
  const password = process.env.E2E_PASS;
  if (!email || !password) throw new Error('Faltan E2E_USER y E2E_PASS');

  await page.goto('/login');
  await page.getByLabel('Email').fill(email);
  await page.getByLabel('Contraseña').fill(password);
  await page.getByRole('button', { name: 'Entrar' }).click();

  await expect(page).toHaveURL(/\/unidades$/);
  await page.context().storageState({ path: SESSION_FILE });
});
