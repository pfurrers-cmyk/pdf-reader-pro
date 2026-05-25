use once_cell::sync::Lazy;
use parking_lot::Mutex;
use pdfium_render::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

use tantivy::schema::{Schema, STORED, TEXT, Value};
use tantivy::{doc, Index, IndexReader, IndexWriter, TantivyDocument};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use walkdir::WalkDir;

// ============================================================================
// TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfRectDto {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultDto {
    pub page_index: u32,
    pub match_text: String,
    pub rects: Vec<PdfRectDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSearchResultDto {
    pub title: String,
    pub path: String,
    pub snippet: String,
    pub score: f32,
}

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
    path: PathBuf,
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
    #[error("Erro no motor de busca: {0}")]
    SearchEngineError(String),
    #[error("Erro ao manipular anotações: {0}")]
    AnnotationError(String),
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

/// Search Engine Globals
struct SearchEngine {
    index: Index,
    reader: IndexReader,
    schema: Schema,
}

static TANTIVY_ENGINE: Lazy<Mutex<Option<SearchEngine>>> = Lazy::new(|| Mutex::new(None));

/// Initialize the Pdfium engine
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
// TAURI COMMANDS: SEARCH ENGINE
// ============================================================================

fn init_search_engine() -> Result<(), PdfError> {
    let mut engine = TANTIVY_ENGINE.lock();
    if engine.is_none() {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("body", TEXT | STORED);
        schema_builder.add_text_field("path", TEXT | STORED);
        let schema = schema_builder.build();

        let index = Index::create_in_ram(schema.clone());
        let reader = index
            .reader_builder()
            .try_into()
            .map_err(|e| PdfError::SearchEngineError(e.to_string()))?;

        *engine = Some(SearchEngine {
            index,
            reader,
            schema,
        });
        eprintln!("[PDF Reader Pro] Tantivy In-Memory Engine initialized");
    }
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
async fn index_folder(dir_path: String) -> Result<usize, PdfError> {
    init_search_engine()?;
    
    let result = tokio::task::spawn_blocking(move || {
        let engine_guard = TANTIVY_ENGINE.lock();
        let engine = engine_guard.as_ref().unwrap();
        
        let mut index_writer: IndexWriter = engine
            .index
            .writer(50_000_000)
            .map_err(|e| PdfError::SearchEngineError(e.to_string()))?;

        let title_field = engine.schema.get_field("title").unwrap();
        let body_field = engine.schema.get_field("body").unwrap();
        let path_field = engine.schema.get_field("path").unwrap();

        let pdfium = get_pdfium();
        let mut count = 0;

        for entry in WalkDir::new(&dir_path).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("pdf") {
                let file_path = entry.path().to_string_lossy().into_owned();
                let file_name = entry.file_name().to_string_lossy().into_owned();
                
                if let Ok(bytes) = std::fs::read(entry.path()) {
                    if let Ok(document) = pdfium.load_pdf_from_byte_slice(&bytes, None) {
                        let mut full_text = String::new();
                        for page in document.pages().iter() {
                            if let Ok(text_page) = page.text() {
                                full_text.push_str(&text_page.all());
                                full_text.push(' ');
                            }
                        }

                        if !full_text.trim().is_empty() {
                            index_writer.add_document(doc!(
                                title_field => file_name,
                                body_field => full_text,
                                path_field => file_path
                            )).map_err(|e| PdfError::SearchEngineError(e.to_string()))?;
                            count += 1;
                        }
                    }
                }
            }
        }
        
        index_writer.commit().map_err(|e| PdfError::SearchEngineError(e.to_string()))?;
        Ok::<usize, PdfError>(count)
    }).await.map_err(|e| PdfError::SearchEngineError(e.to_string()))??;

    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
async fn global_search(query: String) -> Result<Vec<GlobalSearchResultDto>, PdfError> {
    init_search_engine()?;

    let result = tokio::task::spawn_blocking(move || {
        let engine_guard = TANTIVY_ENGINE.lock();
        let engine = engine_guard.as_ref().unwrap();

        let searcher = engine.reader.searcher();
        let title_field = engine.schema.get_field("title").unwrap();
        let body_field = engine.schema.get_field("body").unwrap();
        let path_field = engine.schema.get_field("path").unwrap();

        let query_parser = QueryParser::for_index(&engine.index, vec![title_field, body_field]);
        
        let parsed_query = match query_parser.parse_query(&query) {
            Ok(q) => q,
            Err(_) => return Ok(Vec::new()),
        };

        let top_docs = searcher
            .search(&parsed_query, &TopDocs::with_limit(20).order_by_score())
            .map_err(|e| PdfError::SearchEngineError(e.to_string()))?;

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            if let Ok(retrieved_doc) = searcher.doc::<TantivyDocument>(doc_address) {
                let title = retrieved_doc.get_first(title_field)
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let path = retrieved_doc.get_first(path_field)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let body = retrieved_doc.get_first(body_field)
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let snippet = if body.len() > 150 {
                    let mut s = String::from(&body[..150]);
                    s.push_str("...");
                    s
                } else {
                    body.to_string()
                };

                results.push(GlobalSearchResultDto {
                    title,
                    path,
                    snippet,
                    score,
                });
            }
        }

        Ok::<Vec<GlobalSearchResultDto>, PdfError>(results)
    }).await.map_err(|e| PdfError::SearchEngineError(e.to_string()))??;

    Ok(result)
}

// ============================================================================
// TAURI COMMANDS: BASIC PDF
// ============================================================================

#[tauri::command]
async fn open_pdf(path: String) -> Result<OpenPdfResponse, PdfError> {
    eprintln!("[PDF Reader Pro] open_pdf called with path: {}", path);
    let file_path = PathBuf::from(&path);

    if !file_path.exists() {
        return Err(PdfError::IoError(format!("Arquivo não existe: {}", path)));
    }

    let file_name = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "documento.pdf".to_string());

    let path_clone = path.clone();
    let bytes = tokio::task::spawn_blocking(move || {
        std::fs::read(&path_clone).map_err(|e| PdfError::IoError(e.to_string()))
    })
    .await
    .map_err(|e| PdfError::IoError(e.to_string()))??;

    let bytes_for_render = bytes.clone();
    let render_result = tokio::task::spawn_blocking(move || {
        let pdfium = get_pdfium();
        let document = pdfium
            .load_pdf_from_byte_slice(&bytes_for_render, None)
            .map_err(|e| PdfError::LoadError(e.to_string()))?;

        let page_count = document.pages().len() as u32;
        let first_page = document
            .pages()
            .get(0u16)
            .map_err(|e| PdfError::InvalidPage(e.to_string()))?;

        let page_width = first_page.width().value as f32;
        let page_height = first_page.height().value as f32;

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

    let pdf_id = PdfId::new();
    let id_str = pdf_id.0.clone();

    {
        let mut store = PDF_STORE.lock();
        store.insert(
            id_str.clone(),
            PdfDocument {
                path: file_path,
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

#[tauri::command(rename_all = "snake_case")]
async fn render_page(
    pdf_id: String,
    page_num: u32,
    zoom: f32,
) -> Result<RenderPageResponse, PdfError> {
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

        let page_count = document.pages().len() as u32;
        if page_num >= page_count {
            return Err(PdfError::InvalidPage(format!(
                "Página {} não existe.",
                page_num + 1
            )));
        }

        let page = document
            .pages()
            .get(page_num as u16)
            .map_err(|e| PdfError::InvalidPage(e.to_string()))?;

        let base_width: i32 = 800;
        let target_width = (base_width as f32 * zoom) as i32;
        let max_height = (target_width as f32 * 2.0) as i32;

        // Render including annotations (they are rendered by default in pdfium-render 0.8+)
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

#[tauri::command(rename_all = "snake_case")]
async fn get_page_count(pdf_id: String) -> Result<u32, PdfError> {
    let store = PDF_STORE.lock();
    match store.get(&pdf_id) {
        Some(doc) => Ok(doc.page_count),
        None => Err(PdfError::DocumentNotFound(pdf_id)),
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn close_pdf(pdf_id: String) -> Result<(), PdfError> {
    let mut store = PDF_STORE.lock();
    store.remove(&pdf_id).ok_or_else(|| PdfError::DocumentNotFound(pdf_id))?;
    Ok(())
}

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

#[tauri::command(rename_all = "snake_case")]
async fn search_in_doc(
    pdf_id: String,
    query: String,
    match_case: bool,
) -> Result<Vec<SearchResultDto>, PdfError> {
    if query.is_empty() {
        return Ok(Vec::new());
    }

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

        let mut all_results = Vec::new();

        for (page_index, page) in document.pages().iter().enumerate() {
            if let Ok(text_page) = page.text() {
                let options = PdfSearchOptions::new().match_case(match_case);
                if let Ok(search) = text_page.search(&query, &options) {
                    for segments in search.iter(PdfSearchDirection::SearchForward) {
                        let mut match_text = String::new();
                        let mut rects = Vec::new();
                        
                        for i in 0..segments.len() {
                            if let Ok(segment) = segments.get((i as u32).try_into().unwrap()) {
                                match_text.push_str(&segment.text());
                                let bounds = segment.bounds();
                                rects.push(PdfRectDto {
                                    left: bounds.left().value as f32,
                                    top: bounds.top().value as f32,
                                    right: bounds.right().value as f32,
                                    bottom: bounds.bottom().value as f32,
                                });
                            }
                        }
                        
                        all_results.push(SearchResultDto {
                            page_index: page_index as u32,
                            match_text,
                            rects,
                        });
                    }
                }
            }
        }
        Ok::<Vec<SearchResultDto>, PdfError>(all_results)
    })
    .await
    .map_err(|e| PdfError::RenderError(e.to_string()))??;

    Ok(result)
}

// ============================================================================
// TAURI COMMANDS: ANNOTATIONS & SAVE (PHASE 3)
// ============================================================================

/// Adds a sticky note (Text Annotation) at a specific coordinate
#[tauri::command(rename_all = "snake_case")]
async fn add_text_annotation(
    pdf_id: String,
    page_num: u32,
    x: f32,
    y: f32,
    content: String,
    r: u8,
    g: u8,
    b: u8,
) -> Result<(), PdfError> {
    let mut bytes = {
        let store = PDF_STORE.lock();
        match store.get(&pdf_id) {
            Some(doc) => doc.bytes.clone(),
            None => return Err(PdfError::DocumentNotFound(pdf_id.clone())),
        }
    };

    let new_bytes = tokio::task::spawn_blocking(move || {
        let pdfium = get_pdfium();
        let mut document = pdfium
            .load_pdf_from_byte_slice(&bytes, None)
            .map_err(|e| PdfError::LoadError(e.to_string()))?;

        {
            let mut page = document
                .pages_mut().get(page_num as u16)
                .map_err(|e| PdfError::InvalidPage(e.to_string()))?;
                
            let mut annotation = page.annotations_mut()
                .create_text_annotation("")
                .map_err(|e| PdfError::AnnotationError(e.to_string()))?;

            let width = 24.0;
            let height = 24.0;
            
            let points = PdfPoints::new(x);
            let y_points = PdfPoints::new(y);
            
            annotation.set_position(points, y_points)
                .map_err(|e| PdfError::AnnotationError(e.to_string()))?;
                
            annotation.set_width(PdfPoints::new(width))
                .map_err(|e| PdfError::AnnotationError(e.to_string()))?;
                
            annotation.set_height(PdfPoints::new(height))
                .map_err(|e| PdfError::AnnotationError(e.to_string()))?;

            annotation.set_contents(&content)
                .map_err(|e| PdfError::AnnotationError(e.to_string()))?;
                
            let color = PdfColor::new(r, g, b, 255);
            annotation.set_fill_color(color)
                .map_err(|e| PdfError::AnnotationError(e.to_string()))?;
        }

        // Save the mutated document back to bytes
        document.save_to_bytes().map_err(|e| PdfError::AnnotationError(e.to_string()))
    })
    .await
    .map_err(|e| PdfError::AnnotationError(e.to_string()))??;

    // Update the byte store with the new version containing the annotation
    let mut store = PDF_STORE.lock();
    if let Some(doc) = store.get_mut(&pdf_id) {
        doc.bytes = new_bytes;
    }
    
    Ok(())
}

/// Adds a highlight over the specified coordinates (useful after selecting a rect area)
#[tauri::command(rename_all = "snake_case")]
async fn add_highlight_annotation(
    pdf_id: String,
    page_num: u32,
    left: f32,
    bottom: f32,
    right: f32,
    top: f32,
    r: u8,
    g: u8,
    b: u8,
) -> Result<(), PdfError> {
    let mut bytes = {
        let store = PDF_STORE.lock();
        match store.get(&pdf_id) {
            Some(doc) => doc.bytes.clone(),
            None => return Err(PdfError::DocumentNotFound(pdf_id.clone())),
        }
    };

    let new_bytes = tokio::task::spawn_blocking(move || {
        let pdfium = get_pdfium();
        let mut document = pdfium
            .load_pdf_from_byte_slice(&bytes, None)
            .map_err(|e| PdfError::LoadError(e.to_string()))?;

        {
            let mut page = document
                .pages_mut().get(page_num as u16)
                .map_err(|e| PdfError::InvalidPage(e.to_string()))?;
                
            let mut annotation = page.annotations_mut()
                .create_highlight_annotation()
                .map_err(|e| PdfError::AnnotationError(e.to_string()))?;

            annotation.set_position(PdfPoints::new(left), PdfPoints::new(bottom))
                .map_err(|e| PdfError::AnnotationError(e.to_string()))?;
                
            annotation.set_width(PdfPoints::new(right - left))
                .map_err(|e| PdfError::AnnotationError(e.to_string()))?;
                
            annotation.set_height(PdfPoints::new(top - bottom))
                .map_err(|e| PdfError::AnnotationError(e.to_string()))?;

            // Generate attachment points bounding box using PdfQuadPoints (4 corners of a quad)
            let quad = PdfQuadPoints::new(
                PdfPoints::new(left), PdfPoints::new(top),      // top-left (x1, y1)
                PdfPoints::new(right), PdfPoints::new(top),     // top-right (x2, y2)
                PdfPoints::new(left), PdfPoints::new(bottom),   // bottom-left (x3, y3)
                PdfPoints::new(right), PdfPoints::new(bottom),  // bottom-right (x4, y4)
            );
            annotation.attachment_points_mut()
                .create_attachment_point_at_end(quad)
                .map_err(|e| PdfError::AnnotationError(e.to_string()))?;

            let color = PdfColor::new(r, g, b, 128); // 50% opacity
            annotation.set_fill_color(color)
                .map_err(|e| PdfError::AnnotationError(e.to_string()))?;
        }

        // Save the mutated document back to bytes
        document.save_to_bytes().map_err(|e| PdfError::AnnotationError(e.to_string()))
    })
    .await
    .map_err(|e| PdfError::AnnotationError(e.to_string()))??;

    // Update the byte store
    let mut store = PDF_STORE.lock();
    if let Some(doc) = store.get_mut(&pdf_id) {
        doc.bytes = new_bytes;
    }
    
    Ok(())
}

/// Persists the currently mutated PDF Document bytes to the filesystem
#[tauri::command(rename_all = "snake_case")]
async fn save_pdf_to_disk(pdf_id: String) -> Result<(), PdfError> {
    let (path, bytes) = {
        let store = PDF_STORE.lock();
        match store.get(&pdf_id) {
            Some(doc) => (doc.path.clone(), doc.bytes.clone()),
            None => return Err(PdfError::DocumentNotFound(pdf_id.clone())),
        }
    };

    tokio::task::spawn_blocking(move || {
        std::fs::write(&path, bytes).map_err(|e| PdfError::IoError(e.to_string()))
    })
    .await
    .map_err(|e| PdfError::IoError(e.to_string()))??;

    Ok(())
}

// ============================================================================
// APPLICATION ENTRY POINT
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            open_pdf,
            render_page,
            get_page_count,
            close_pdf,
            get_page_dimensions,
            search_in_doc,
            index_folder,
            global_search,
            add_text_annotation,
            add_highlight_annotation,
            save_pdf_to_disk,
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar PDF Reader Pro");
}

