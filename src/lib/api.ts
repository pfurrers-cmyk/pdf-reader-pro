// ============================================================================
// TAURI IPC BRIDGE — PDF Reader Pro
// Uses @tauri-apps npm packages which work with withGlobalTauri: true
// ============================================================================

import { invoke } from '@tauri-apps/api/core';
import { open as dialogOpen } from '@tauri-apps/plugin-dialog';
import type { OpenPdfResponse, RenderPageResponse } from './types';

/**
 * Open a native file dialog to select a PDF file.
 * Returns the file path as a string, or null if cancelled.
 */
export async function openFileDialog(): Promise<string | null> {
  const selected = await dialogOpen({
    multiple: false,
    filters: [{ name: 'PDF', extensions: ['pdf'] }],
  });
  // dialog returns string | null (single) or string[] | null (multiple)
  if (!selected) return null;
  if (typeof selected === 'string') return selected;
  if (Array.isArray(selected) && selected.length > 0) return selected[0];
  return null;
}

/**
 * Open a PDF file at the given path.
 * Returns the first page rendered as a base64 PNG image + metadata.
 */
export async function openPdf(path: string): Promise<OpenPdfResponse> {
  return await invoke<OpenPdfResponse>('open_pdf', { path });
}

/**
 * Render a specific page of an open PDF.
 */
export async function renderPage(
  pdfId: string,
  pageNum: number,
  zoom: number
): Promise<RenderPageResponse> {
  return await invoke<RenderPageResponse>('render_page', { pdfId, pageNum, zoom });
}

/**
 * Get the total page count for an open PDF.
 */
export async function getPageCount(pdfId: string): Promise<number> {
  return await invoke<number>('get_page_count', { pdfId });
}

/**
 * Close a PDF and free its memory.
 */
export async function closePdf(pdfId: string): Promise<void> {
  return await invoke<void>('close_pdf', { pdfId });
}

/**
 * Get page dimensions (width, height) for a specific page.
 */
export async function getPageDimensions(
  pdfId: string,
  pageNum: number
): Promise<[number, number]> {
  return await invoke<[number, number]>('get_page_dimensions', { pdfId, pageNum });
}
