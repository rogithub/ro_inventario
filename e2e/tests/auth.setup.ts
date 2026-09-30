import { expect, test as setup } from '@playwright/test';
import { SESION } from '../playwright.config';

// Credenciales del usuario de pruebas: del perfil del dueño en kukulkan, aleatorias en CI.
// Nunca se imprimen.
setup('entrar con el usuario de pruebas', async ({ page }) => {
  const email = process.env.E2E_USER;
  const contrasena = process.env.E2E_PASS;
  if (!email || !contrasena) throw new Error('Faltan E2E_USER y E2E_PASS');

  await page.goto('/login');
  await page.getByLabel('Email').fill(email);
  await page.getByLabel('Contraseña').fill(contrasena);
  await page.getByRole('button', { name: 'Entrar' }).click();

  await expect(page).toHaveURL(/\/unidades$/);
  await page.context().storageState({ path: SESION });
});
