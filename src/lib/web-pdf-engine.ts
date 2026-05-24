import * as pdfjsLib from 'pdfjs-dist';

// Use CDN for the worker to avoid complex bundler configurations for the MVP
pdfjsLib.GlobalWorkerOptions.workerSrc = `https://unpkg.com/pdfjs-dist@${pdfjsLib.version}/build/pdf.worker.min.mjs`;

const webDocs = new Map<string, pdfjsLib.PDFDocumentProxy>();

export async function webOpenPdf(url: string, fileName: string) {
  const loadingTask = pdfjsLib.getDocument(url);
  const doc = await loadingTask.promise;
  const pdfId = 'web-' + Math.random().toString(36).substring(2);
  webDocs.set(pdfId, doc);

  const page = await doc.getPage(1);
  const viewport = page.getViewport({ scale: 1.5 }); // Base scale matching approx rust dimensions
  
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error("Could not get canvas context");
  
  canvas.width = viewport.width;
  canvas.height = viewport.height;
  
  await page.render({ canvasContext: ctx, viewport }).promise;
  const base64 = canvas.toDataURL('image/png').split(',')[1];

  return {
    pdfId,
    fileName,
    pageCount: doc.numPages,
    firstPageImage: base64,
    pageWidth: viewport.width,
    pageHeight: viewport.height,
  };
}

export async function webRenderPage(pdfId: string, pageNum: number, zoom: number) {
  const doc = webDocs.get(pdfId);
  if (!doc) throw new Error('PDF not found in Web Engine');

  const page = await doc.getPage(pageNum + 1); // 1-based index
  const viewport = page.getViewport({ scale: zoom * 1.5 });
  
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error("Could not get canvas context");
  
  canvas.width = viewport.width;
  canvas.height = viewport.height;
  
  await page.render({ canvasContext: ctx, viewport }).promise;
  const base64 = canvas.toDataURL('image/png').split(',')[1];

  return {
    image: base64,
    width: canvas.width,
    height: canvas.height,
    pageIndex: pageNum,
  };
}

export async function webGetPageCount(pdfId: string) {
  const doc = webDocs.get(pdfId);
  return doc ? doc.numPages : 0;
}

export async function webClosePdf(pdfId: string) {
  const doc = webDocs.get(pdfId);
  if (doc) {
    await doc.destroy();
    webDocs.delete(pdfId);
  }
}
