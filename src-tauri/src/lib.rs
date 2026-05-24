use once_cell::sync::Lazy;
use parking_lot::Mutex;
use pdfium_render::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

// ============================================================================
// TYPES
// ============================================================================

/// Unique identifier for an open PDF document
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct PdfId(String);

impl PdfId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

/// Represents an open PDF document stored in memory
struct PdfDocument {
    _path: PathBuf,
    bytes: Vec<u8>,
    page_count: u32,
}

/// Response returned when opening a PDF
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPdfResponse {
    pdf_id: String,
    file_name: String,
    page_count: u32,
    first_page_image: String, // base64 encoded PNG
    page_width: f32,
    page_height: f32,
}

/// Response returned when rendering a page
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPageResponse {
    image: String, // base64 encoded PNG
    width: u32,
    height: u32,
    page_index: u32,
}

/// Error type for PDF operations
#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    #[error("PDF não encontrado: {0}")]
    DocumentNotFound(String),
    #[error("Erro ao carregar PDF: {0}")]
    LoadError(String),
    #[error("Erro ao renderizar página: {0}")]
    RenderError(String),
    #[error("Página inválida: {0}")]
    InvalidPage(String),
    #[error("Erro de IO: {0}")]
    IoError(String),
}

impl Serialize for PdfError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// ============================================================================
// GLOBAL STATE
// ============================================================================

/// Global store of open PDF documents (thread-safe)
static PDF_STORE: Lazy<Mutex<HashMap<String, PdfDocument>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Initialize the Pdfium engine
/// Tries to bind to pdfium.dll in the app directory first, then system library
fn get_pdfium() -> Pdfium {
    match Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")) {
        Ok(bindings) => Pdfium::new(bindings),
        Err(_) => {
            eprintln!("[PDF Reader Pro] WARNING: pdfium.dll not found in current dir, trying system library...");
            match Pdfium::bind_to_system_library() {
                Ok(bindings) => Pdfium::new(bindings),
                Err(e) => {
                    eprintln!("[PDF Reader Pro] FATAL: Could not bind to any PDFium library: {:?}", e);
                    panic!("Could not bind to PDFium library. Ensure pdfium.dll is next to the executable.");
                }
            }
        }
    }
}

/// Helper: render a PdfBitmap to base64 PNG
fn bitmap_to_base64_png(bitmap: &PdfBitmap) -> Result<String, PdfError> {
    let image = bitmap.as_image(); // pdfium-render 0.8 returns DynamicImage directly
    let rgb_image = image.into_rgb8();
    let mut png_bytes: Vec<u8> = Vec::new();
    rgb_image
        .write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| PdfError::RenderError(e.to_string()))?;
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &png_bytes,
    ))
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

/// Open a PDF file, load it into memory, and render the first page
/// JS calls: invoke('open_pdf', { path: '/path/to/file.pdf' })
/// Tauri v2: camelCase JS args auto-convert to snake_case Rust params
#[tauri::command]
async fn open_pdf(path: String) -> Result<OpenPdfResponse, PdfError> {
    eprintln!("[PDF Reader Pro] open_pdf called with path: {}", path);
    let file_path = PathBuf::from(&path);

    if !file_path.exists() {
        eprintln!("[PDF Reader Pro] File does NOT exist: {}", path);
        return Err(PdfError::IoError(format!(
            "Arquivo não existe: {}",
            path
        )));
    }
    eprintln!("[PDF Reader Pro] File exists, reading bytes...");

    let file_name = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "documento.pdf".to_string());

    // Read file bytes in a blocking thread to not freeze the main thread
    let path_clone = path.clone();
    let bytes = tokio::task::spawn_blocking(move || {
        std::fs::read(&path_clone).map_err(|e| PdfError::IoError(e.to_string()))
    })
    .await
    .map_err(|e| PdfError::IoError(e.to_string()))??;

    // Clone bytes for the blocking render task
    let bytes_for_render = bytes.clone();

    // Render first page in a blocking thread (heavy CPU work)
    let render_result = tokio::task::spawn_blocking(move || {
        let pdfium = get_pdfium();
        let document = pdfium
            .load_pdf_from_byte_slice(&bytes_for_render, None)
            .map_err(|e| PdfError::LoadError(e.to_string()))?;

        let page_count = document.pages().len() as u32;

        // Get first page
        let first_page = document
            .pages()
            .get(0u16)
            .map_err(|e| PdfError::InvalidPage(e.to_string()))?;

        let page_width = first_page.width().value as f32;
        let page_height = first_page.height().value as f32;

        // Render first page with a reasonable default width
        let render_config = PdfRenderConfig::new()
            .set_target_width(1200)
            .set_maximum_height(2000);

        let bitmap = first_page
            .render_with_config(&render_config)
            .map_err(|e| PdfError::RenderError(e.to_string()))?;

        let base64_image = bitmap_to_base64_png(&bitmap)?;

        Ok::<(u32, f32, f32, String), PdfError>((page_count, page_width, page_height, base64_image))
    })
    .await
    .map_err(|e| PdfError::RenderError(e.to_string()))??;

    let (page_count, page_width, page_height, first_page_image) = render_result;
    eprintln!("[PDF Reader Pro] Rendered first page: {}x{}, {} pages, image {} bytes", 
        page_width, page_height, page_count, first_page_image.len());

    // Generate unique ID and store document
    let pdf_id = PdfId::new();
    let id_str = pdf_id.0.clone();

    {
        let mut store = PDF_STORE.lock();
        store.insert(
            id_str.clone(),
            PdfDocument {
                _path: file_path,
                bytes,
                page_count,
            },
        );
    }

    Ok(OpenPdfResponse {
        pdf_id: id_str,
        file_name,
        page_count,
        first_page_image,
        page_width,
        page_height,
    })
}

/// Render a specific page of an open PDF at the given zoom level
/// JS calls: invoke('render_page', { pdfId, pageNum, zoom })
/// Tauri v2: camelCase JS "pdfId" → snake_case Rust "pdf_id"
#[tauri::command(rename_all = "snake_case")]
async fn render_page(
    pdf_id: String,
    page_num: u32,
    zoom: f32,
) -> Result<RenderPageResponse, PdfError> {
    eprintln!("[PDF Reader Pro] render_page called: pdf_id={}, page={}, zoom={}", pdf_id, page_num, zoom);
    // Get document bytes from store
    let bytes = {
        let store = PDF_STORE.lock();
        match store.get(&pdf_id) {
            Some(doc) => doc.bytes.clone(),
            None => return Err(PdfError::DocumentNotFound(pdf_id)),
        }
    };

    // Render in blocking thread
    let result = tokio::task::spawn_blocking(move || {
        let pdfium = get_pdfium();
        let document = pdfium
            .load_pdf_from_byte_slice(&bytes, None)
            .map_err(|e| PdfError::LoadError(e.to_string()))?;

        let page_count = document.pages().len() as u32;
        if page_num >= page_count {
            return Err(PdfError::InvalidPage(format!(
                "Página {} não existe. Total: {}",
                page_num + 1,
                page_count
            )));
        }

        let page = document
            .pages()
            .get(page_num as u16)
            .map_err(|e| PdfError::InvalidPage(e.to_string()))?;

        // Calculate render dimensions based on zoom
        let base_width: i32 = 800;
        let target_width = (base_width as f32 * zoom) as i32;
        let max_height = (target_width as f32 * 2.0) as i32;

        let render_config = PdfRenderConfig::new()
            .set_target_width(target_width)
            .set_maximum_height(max_height);

        let bitmap = page
            .render_with_config(&render_config)
            .map_err(|e| PdfError::RenderError(e.to_string()))?;

        let width = bitmap.width() as u32;
        let height = bitmap.height() as u32;

        let base64_image = bitmap_to_base64_png(&bitmap)?;

        Ok::<RenderPageResponse, PdfError>(RenderPageResponse {
            image: base64_image,
            width,
            height,
            page_index: page_num,
        })
    })
    .await
    .map_err(|e| PdfError::RenderError(e.to_string()))??;

    Ok(result)
}

/// Get the total number of pages for an open PDF
#[tauri::command(rename_all = "snake_case")]
async fn get_page_count(pdf_id: String) -> Result<u32, PdfError> {
    let store = PDF_STORE.lock();
    match store.get(&pdf_id) {
        Some(doc) => Ok(doc.page_count),
        None => Err(PdfError::DocumentNotFound(pdf_id)),
    }
}

/// Close a PDF document and free its memory
#[tauri::command(rename_all = "snake_case")]
async fn close_pdf(pdf_id: String) -> Result<(), PdfError> {
    let mut store = PDF_STORE.lock();
    store
        .remove(&pdf_id)
        .ok_or_else(|| PdfError::DocumentNotFound(pdf_id.clone()))?;
    Ok(())
}

/// Get page dimensions for a specific page
#[tauri::command(rename_all = "snake_case")]
async fn get_page_dimensions(pdf_id: String, page_num: u32) -> Result<(f32, f32), PdfError> {
    let bytes = {
        let store = PDF_STORE.lock();
        match store.get(&pdf_id) {
            Some(doc) => doc.bytes.clone(),
            None => return Err(PdfError::DocumentNotFound(pdf_id)),
        }
    };

    let result = tokio::task::spawn_blocking(move || {
        let pdfium = get_pdfium();
        let document = pdfium
            .load_pdf_from_byte_slice(&bytes, None)
            .map_err(|e| PdfError::LoadError(e.to_string()))?;

        let page = document
            .pages()
            .get(page_num as u16)
            .map_err(|e| PdfError::InvalidPage(e.to_string()))?;

        Ok::<(f32, f32), PdfError>((page.width().value as f32, page.height().value as f32))
    })
    .await
    .map_err(|e| PdfError::RenderError(e.to_string()))??;

    Ok(result)
}

// ============================================================================
// APPLICATION ENTRY POINT
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            open_pdf,
            render_page,
            get_page_count,
            close_pdf,
            get_page_dimensions,
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar PDF Reader Pro");
}
