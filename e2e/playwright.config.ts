import { defineConfig, devices } from '@playwright/test';

// Arranca la aplicación privada compilada del código actual, contra la base que diga
// DATABASE_URL (en desarrollo, dev_ro_inventario; en CI, un Postgres desechable).
// Sin interfaz gráfica: los navegadores corren headless (kukulkan y CI no tienen pantalla).
const PORT = Number(process.env.E2E_PORT ?? 5099);
const baseURL = `http://localhost:${PORT}`;
// La sesión del usuario de pruebas (E2E_USER / E2E_PASS), guardada por auth.setup.ts.
export const SESSION_FILE = '.auth/usuario.json';

export default defineConfig({
  testDir: './tests',
  forbidOnly: !!process.env.CI,
  reporter: [['list'], ['html', { open: 'never' }]],
  use: {
    baseURL,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  // Los dos puntos de venta reales (CLAUDE.md, "Entorno de uso"). El iPad se emula con
  // Chromium: WebKit en Linux ARM pide paquetes del sistema y aun así no es el Safari real;
  // lo que más importa del iPad es el ancho (744 px, entre los breakpoints sm y md).
  projects: [
    // Entra una vez con el usuario de pruebas y guarda la sesión para los demás proyectos.
    { name: 'setup', testMatch: /auth\.setup\.ts/ },
    {
      name: 'escritorio-firefox',
      use: {
        ...devices['Desktop Firefox'],
        viewport: { width: 1920, height: 1080 },
        storageState: SESSION_FILE,
      },
      dependencies: ['setup'],
    },
    {
      name: 'ipad-mini',
      use: {
        ...devices['Desktop Chrome'],
        viewport: { width: 744, height: 1133 },
        deviceScaleFactor: 2,
        hasTouch: true,
        storageState: SESSION_FILE,
      },
      dependencies: ['setup'],
    },
  ],
  webServer: {
    command: 'cargo run -q -p privada',
    cwd: '..',
    url: `${baseURL}/health`,
    // Compilar en la Raspberry Pi puede tardar.
    timeout: 300_000,
    // Siempre el código actual, nunca un servidor que se quedó corriendo.
    reuseExistingServer: false,
    env: {
      PORT: String(PORT),
      NEGOCIO_CONFIG: 'negocio.ejemplo.toml',
    },
  },
});
