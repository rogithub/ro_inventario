import { defineConfig } from '@playwright/test';
import { dispositivos, evidencia } from '@ro/e2e';

// Arranca la aplicación privada compilada del código actual, contra la base que diga
// DATABASE_URL (en desarrollo, dev_ro_inventario; en CI, un Postgres desechable).
// Sin interfaz gráfica: los navegadores corren headless (kukulkan y CI no tienen pantalla).
const PORT = Number(process.env.E2E_PORT ?? 5099);
const baseURL = `http://localhost:${PORT}`;
// La sesión del usuario de pruebas (E2E_USER / E2E_PASS), guardada por auth.setup.ts.
export const SESSION_FILE = '.auth/usuario.json';

// Las pruebas viven en tests/ y, las de un caso, en historias/<área>/<historia>/<caso>/
// (kit @ro/e2e, docs/decisiones/2026-10-08-pruebas-e2e-por-historia.md). Con E2E_EVIDENCIA=1 se
// graba video de cada prueba; el reporte por paso queda en corridas/.
export default defineConfig({
  testDir: '.',
  testIgnore: ['node_modules/**'],
  forbidOnly: !!process.env.CI,
  reporter: [['list'], ['html', { open: 'never' }], ['@ro/e2e/reporter', { proyecto: 'ro_inventario' }]],
  use: {
    baseURL,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    ...evidencia(),
  },
  // Los dos puntos de venta reales (CLAUDE.md, "Entorno de uso"), definidos en @ro/e2e. El iPad
  // se emula con Chromium: WebKit en Linux ARM pide paquetes del sistema y aun así no es el
  // Safari real; lo que más importa del iPad es el ancho (744 px, entre los breakpoints sm y md).
  projects: [
    // Entra una vez con el usuario de pruebas y guarda la sesión para los demás proyectos.
    { name: 'setup', testMatch: /auth\.setup\.ts/ },
    {
      name: 'escritorio-firefox',
      use: { ...dispositivos['escritorio-firefox'], storageState: SESSION_FILE },
      dependencies: ['setup'],
    },
    {
      name: 'ipad-mini',
      use: { ...dispositivos['ipad-mini'], storageState: SESSION_FILE },
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
