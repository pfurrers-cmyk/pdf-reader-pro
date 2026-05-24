// ============================================================================
// TYPESCRIPT TYPES — PDF Reader Pro
// ============================================================================

/** Unique identifier for an open PDF document */
export type PdfId = string;

/** Response from opening a PDF (matches Rust serde camelCase output) */
export interface OpenPdfResponse {
  pdfId: PdfId;
  fileName: string;
  pageCount: number;
  firstPageImage: string; // base64 PNG
  pageWidth: number;
  pageHeight: number;
}

/** Response from rendering a page (matches Rust serde camelCase output) */
export interface RenderPageResponse {
  image: string; // base64 PNG
  width: number;
  height: number;
  pageIndex: number;
}

export interface PdfRectDto {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

export interface SearchResultDto {
  pageIndex: number;
  matchText: string;
  rects: PdfRectDto[];
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
