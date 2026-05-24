// ============================================================================
// TAURI IPC BRIDGE & WEB FALLBACK — PDF Reader Pro
// ============================================================================

import { invoke } from '@tauri-apps/api/core';
import { open as dialogOpen } from '@tauri-apps/plugin-dialog';
import type { OpenPdfResponse, RenderPageResponse } from './types';

// Helper to detect if running inside Tauri Native or Web Browser
export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

/**
 * Open a native file dialog to select a PDF file.
 */
export async function openFileDialog(): Promise<string | null> {
  if (!isTauri) return null; // Web uses <input type="file"> natively
  
  const selected = await dialogOpen({
    multiple: false,
    filters: [{ name: 'PDF', extensions: ['pdf'] }],
  });
  if (!selected) return null;
  if (typeof selected === 'string') return selected;
  if (Array.isArray(selected) && selected.length > 0) return selected[0];
  return null;
}

/**
 * Open a PDF file. Uses Rust in Tauri, or PDF.js in Web.
 */
export async function openPdf(path: string, file?: File): Promise<OpenPdfResponse> {
  if (!isTauri && file) {
    const { webOpenPdf } = await import('./web-pdf-engine');
    const url = URL.createObjectURL(file);
    return await webOpenPdf(url, file.name);
  }
  
  if (!isTauri) {
     throw new Error("No file provided for Web Engine fallback");
  }

  return await invoke<OpenPdfResponse>('open_pdf', { path });
}

/**
 * Render a specific page. Uses Rust in Tauri, or PDF.js in Web.
 */
export async function renderPage(
  pdfId: string,
  pageNum: number,
  zoom: number
): Promise<RenderPageResponse> {
  if (!isTauri && pdfId.startsWith('web-')) {
    const { webRenderPage } = await import('./web-pdf-engine');
    return await webRenderPage(pdfId, pageNum, zoom);
  }
  return await invoke<RenderPageResponse>('render_page', { pdfId, pageNum, zoom });
}

export async function getPageCount(pdfId: string): Promise<number> {
  if (!isTauri && pdfId.startsWith('web-')) {
    const { webGetPageCount } = await import('./web-pdf-engine');
    return await webGetPageCount(pdfId);
  }
  return await invoke<number>('get_page_count', { pdfId });
}

export async function closePdf(pdfId: string): Promise<void> {
  if (!isTauri && pdfId.startsWith('web-')) {
    const { webClosePdf } = await import('./web-pdf-engine');
    return await webClosePdf(pdfId);
  }
  return await invoke<void>('close_pdf', { pdfId });
}

export async function getPageDimensions(
  pdfId: string,
  pageNum: number
): Promise<[number, number]> {
  if (!isTauri && pdfId.startsWith('web-')) {
    return [800, 1100]; // Basic fallback for web
  }
  return await invoke<[number, number]>('get_page_dimensions', { pdfId, pageNum });
}
