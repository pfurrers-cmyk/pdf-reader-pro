// ============================================================================
// TAURI IPC BRIDGE & WEB FALLBACK — PDF Reader Pro
// ============================================================================

import { invoke } from '@tauri-apps/api/core';
import { open as dialogOpen } from '@tauri-apps/plugin-dialog';
import type { OpenPdfResponse, RenderPageResponse, SearchResultDto, GlobalSearchResultDto } from './types';

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

export async function searchInDoc(
  pdfId: string,
  query: string,
  matchCase: boolean = false
): Promise<SearchResultDto[]> {
  if (!isTauri && pdfId.startsWith('web-')) {
    // Basic web fallback for search (returns empty for now until PDF.js text layer is implemented)
    console.warn("Search not fully implemented in web fallback yet.");
    return [];
  }
  return await invoke<SearchResultDto[]>('search_in_doc', { pdfId, query, matchCase });
}

export async function indexFolder(dirPath: string): Promise<number> {
  if (!isTauri) {
    console.warn("Indexação local não suportada via Web. Baixe o aplicativo desktop.");
    return 0;
  }
  return await invoke<number>('index_folder', { dirPath });
}

export async function addHighlightAnnotation(
  pdfId: string,
  pageNum: number,
  left: number,
  bottom: number,
  right: number,
  top: number,
  color: { r: number, g: number, b: number }
): Promise<void> {
  if (!isTauri) return;
  return await invoke<void>('add_highlight_annotation', { 
    pdfId, pageNum, left, bottom, right, top, ...color 
  });
}

export async function addTextAnnotation(
  pdfId: string,
  pageNum: number,
  x: number,
  y: number,
  content: string,
  color: { r: number, g: number, b: number }
): Promise<void> {
  if (!isTauri) return;
  return await invoke<void>('add_text_annotation', { 
    pdfId, pageNum, x, y, content, ...color 
  });
}

export async function globalSearch(query: string): Promise<GlobalSearchResultDto[]> {
  if (!isTauri) {
    console.warn("Busca global não suportada via Web. Baixe o aplicativo desktop.");
    return [];
  }
  return await invoke<GlobalSearchResultDto[]>('global_search', { query });
}

export async function savePdfToDisk(pdfId: string): Promise<void> {
  if (!isTauri) return;
  return await invoke<void>('save_pdf_to_disk', { pdfId });
}
