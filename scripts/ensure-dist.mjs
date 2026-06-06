import { existsSync } from 'node:fs';
import { execSync } from 'node:child_process';

if (!existsSync('dist/index.html')) {
  console.log('dist/ not found — building frontend (required by Tauri compile)…');
  execSync('npm run build', { stdio: 'inherit' });
}
