import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { ask, message } from '@tauri-apps/plugin-dialog';
import { isTauri } from './api';

export async function checkForAppUpdates() {
  if (!isTauri) return;

  try {
    const update = await check();
    if (update) {
      const yes = await ask(
        `Uma nova versão (${update.version}) está disponível.\n\nNotas de lançamento:\n${update.body || 'Correções de bugs e melhorias.'}\n\nDeseja atualizar agora?`,
        { title: 'Atualização Disponível', kind: 'info' }
      );

      if (yes) {
        await update.downloadAndInstall();
        await relaunch();
      }
    }
  } catch (error) {
    console.error('Falha ao checar por atualizações:', error);
  }
}
