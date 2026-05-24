// ============================================================================
// TYPESCRIPT TYPES — PDF Reader Pro
// ============================================================================

/** Unique identifier for an open PDF document */
export type PdfId = string;

/** Response from opening a PDF */
export interface OpenPdfResponse {
  pdf_id: PdfId;
  file_name: string;
  page_count: number;
  first_page_image: string; // base64 PNG
  page_width: number;
  page_height: number;
}

/** Response from rendering a page */
export interface RenderPageResponse {
  image: string; // base64 PNG
  width: number;
  height: number;
  page_index: number;
}

/** A single open tab in the UI */
export interface PdfTab {
  id: PdfId;
  fileName: string;
  pageCount: number;
  pageWidth: number;
  pageHeight: number;
  currentPage: number;
  zoom: number;
  scrollPosition: number;
  renderedPages: Map<number, string>; // page_num -> base64 image
  isLoading: boolean;
}

/** View mode for the PDF reader */
export type ViewMode = 'single' | 'continuous' | 'two-page' | 'reading';

/** Search result within a document */
export interface SearchResult {
  pageIndex: number;
  text: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

/** Global search result from tantivy index */
export interface GlobalSearchResult {
  filePath: string;
  fileName: string;
  pageIndex: number;
  snippet: string;
  score: number;
}
