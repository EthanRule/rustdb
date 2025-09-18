use eframe::egui;
use crate::{
    storage::{
        storage_engine::{StorageEngine, DocumentId},
        file::DatabaseFile,
    },
    Document, Value,
};
use std::path::Path;

pub struct DatabaseApp {
    storage_engine: Option<StorageEngine>,
    
    // Database state
    database_path: String,
    documents: Vec<(DocumentId, Document)>,
    
    // UI state
    json_input: String,
    status_message: String,
    status_color: egui::Color32,
    selected_doc_index: Option<usize>,
    
    // Edit mode
    edit_mode: bool,
    edit_json: String,
    
    // Document creation helper
    show_example_documents: bool,
}

impl Default for DatabaseApp {
    fn default() -> Self {
        Self {
            storage_engine: None,
            database_path: "database_ui.db".to_string(),
            documents: Vec::new(),
            json_input: String::new(),
            status_message: "Ready - Create or Open a database to start".to_string(),
            status_color: egui::Color32::from_rgb(100, 200, 100),
            selected_doc_index: None,
            edit_mode: false,
            edit_json: String::new(),
            show_example_documents: false,
        }
    }
}

impl DatabaseApp {
    pub fn new() -> Self {
        Self::default()
    }
    
    fn create_database(&mut self) {
        match self.create_database_internal() {
            Ok(_) => {
                self.set_status("✅ Database created successfully!", egui::Color32::GREEN);
                self.refresh_documents();
            }
            Err(e) => {
                self.set_status(&format!("❌ Failed to create database: {}", e), egui::Color32::RED);
            }
        }
    }
    
    fn create_database_internal(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.database_path);
        
        // Create the database file
        let _db_file = DatabaseFile::create(path)?;
        drop(_db_file); // Close it so StorageEngine can open it
        
        // Create storage engine
        let storage_engine = StorageEngine::new(path, 64)?;
        self.storage_engine = Some(storage_engine);
        
        Ok(())
    }
    
    fn open_database(&mut self) {
        match self.open_database_internal() {
            Ok(_) => {
                self.set_status("✅ Database opened successfully!", egui::Color32::GREEN);
                self.refresh_documents();
            }
            Err(e) => {
                self.set_status(&format!("❌ Failed to open database: {}", e), egui::Color32::RED);
            }
        }
    }
    
    fn open_database_internal(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.database_path);
        let storage_engine = StorageEngine::new(path, 64)?;
        self.storage_engine = Some(storage_engine);
        Ok(())
    }
    
    fn set_status(&mut self, message: &str, color: egui::Color32) {
        self.status_message = message.to_string();
        self.status_color = color;
    }
    
    fn refresh_documents(&mut self) {
        // Note: In a real implementation, you'd need to add a method to StorageEngine
        // to iterate over all documents. For now, we'll keep the current list.
        self.set_status("📋 Document list refreshed", egui::Color32::LIGHT_BLUE);
    }
    
    fn insert_document_from_json(&mut self) {
        if let Some(ref mut storage_engine) = self.storage_engine {
            let json_input = self.json_input.clone(); // Clone to avoid borrowing issues
            match Self::parse_json_to_document(&json_input) {
                Ok(document) => {
                    match storage_engine.insert_document(&document) {
                        Ok(doc_id) => {
                            self.documents.push((doc_id, document));
                            self.set_status(&format!("✅ Document inserted with ID: {:?}", doc_id), egui::Color32::GREEN);
                            self.json_input.clear();
                        }
                        Err(e) => {
                            self.set_status(&format!("❌ Insert failed: {}", e), egui::Color32::RED);
                        }
                    }
                }
                Err(e) => {
                    self.set_status(&format!("❌ Invalid JSON: {}", e), egui::Color32::RED);
                }
            }
        } else {
            self.set_status("❌ No database open", egui::Color32::RED);
        }
    }
    
    fn parse_json_to_document(json_str: &str) -> Result<Document, Box<dyn std::error::Error>> {
        let json_value: serde_json::Value = serde_json::from_str(json_str)?;
        let mut document = Document::new();
        
        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                let db_value = Self::json_value_to_db_value(value);
                document.set(&key, db_value);
            }
        }
        
        Ok(document)
    }
    
    fn json_value_to_db_value(value: serde_json::Value) -> Value {
        match value {
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                        Value::I32(i as i32)
                    } else {
                        Value::I64(i)
                    }
                } else if let Some(f) = n.as_f64() {
                    Value::F64(f)
                } else {
                    Value::String(n.to_string())
                }
            }
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Array(_) => {
                // For now, convert arrays to strings
                Value::String(value.to_string())
            }
            serde_json::Value::Object(_) => {
                // For now, convert objects to strings
                Value::String(value.to_string())
            }
        }
    }
    
    fn document_to_json_string(document: &Document) -> String {
        let mut json_obj = serde_json::Map::new();
        
        // Add all document fields
        for (key, value) in document.iter() {
            let json_value = Self::db_value_to_json_value(value);
            json_obj.insert(key.clone(), json_value);
        }
        
        // Pretty print the JSON
        serde_json::to_string_pretty(&json_obj).unwrap_or_else(|_| "{}".to_string())
    }
    
    fn db_value_to_json_value(value: &Value) -> serde_json::Value {
        match value {
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::I32(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            Value::I64(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            Value::F64(f) => serde_json::Value::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| serde_json::Number::from(0))),
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::Null => serde_json::Value::Null,
            Value::ObjectId(oid) => serde_json::Value::String(oid.to_string()),
            Value::Array(_) => serde_json::Value::String(format!("{}", value)),
            Value::Object(_) => serde_json::Value::String(format!("{}", value)),
            Value::DateTime(dt) => serde_json::Value::String(dt.to_rfc3339()),
            Value::Binary(_) => serde_json::Value::String(format!("{}", value)),
        }
    }
    
    fn delete_selected_document(&mut self) {
        if let Some(index) = self.selected_doc_index {
            if let Some(ref mut storage_engine) = self.storage_engine {
                let (doc_id, _) = &self.documents[index];
                match storage_engine.delete_document(doc_id) {
                    Ok(_) => {
                        self.documents.remove(index);
                        self.selected_doc_index = None;
                        self.edit_mode = false;
                        self.set_status("✅ Document deleted successfully!", egui::Color32::GREEN);
                    }
                    Err(e) => {
                        self.set_status(&format!("❌ Delete failed: {}", e), egui::Color32::RED);
                    }
                }
            }
        }
    }
    
    fn update_selected_document(&mut self) {
        if let Some(index) = self.selected_doc_index {
            if let Some(ref mut storage_engine) = self.storage_engine {
                let edit_json = self.edit_json.clone(); // Clone to avoid borrowing issues
                match Self::parse_json_to_document(&edit_json) {
                    Ok(new_document) => {
                        let (doc_id, _) = &self.documents[index];
                        let doc_id_copy = *doc_id; // Copy the DocumentId
                        match storage_engine.update_document(&doc_id_copy, &new_document) {
                            Ok(new_doc_id) => {
                                self.documents[index] = (new_doc_id, new_document);
                                self.edit_mode = false;
                                self.set_status("✅ Document updated successfully!", egui::Color32::GREEN);
                            }
                            Err(e) => {
                                self.set_status(&format!("❌ Update failed: {}", e), egui::Color32::RED);
                            }
                        }
                    }
                    Err(e) => {
                        self.set_status(&format!("❌ Invalid JSON: {}", e), egui::Color32::RED);
                    }
                }
            }
        }
    }
    
    fn get_example_documents(&self) -> Vec<&'static str> {
        vec![
            r#"{
  "name": "Alice Johnson",
  "age": 28,
  "email": "alice@example.com",
  "active": true,
  "department": "Engineering"
}"#,
            r#"{
  "name": "Bob Smith", 
  "age": 35,
  "email": "bob@example.com",
  "active": false,
  "department": "Marketing",
  "salary": 75000
}"#,
            r#"{
  "name": "Carol Williams",
  "age": 42,
  "email": "carol@example.com", 
  "active": true,
  "department": "Sales",
  "commission_rate": 0.15
}"#,
            r#"{
  "product_name": "Laptop",
  "price": 999.99,
  "category": "Electronics",
  "in_stock": true,
  "quantity": 25
}"#,
        ]
    }
}

impl eframe::App for DatabaseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("📁 Database", |ui| {
                    if ui.button("🆕 Create New").clicked() {
                        self.create_database();
                        ui.close_menu();
                    }
                    if ui.button("📂 Open Existing").clicked() {
                        self.open_database();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("🔄 Refresh Documents").clicked() {
                        self.refresh_documents();
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("📄 Examples", |ui| {
                    if ui.button("Show Example Documents").clicked() {
                        self.show_example_documents = !self.show_example_documents;
                        ui.close_menu();
                    }
                });
                
                ui.separator();
                ui.label(format!("📂 {}", self.database_path));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let status = if self.storage_engine.is_some() { "🟢 Connected" } else { "🔴 Disconnected" };
                    ui.label(status);
                });
            });
        });
        
        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.colored_label(self.status_color, &self.status_message);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("📊 {} documents", self.documents.len()));
                });
            });
        });
        
        // Left panel - Document list
        egui::SidePanel::left("document_list").resizable(true).show(ctx, |ui| {
            ui.heading("📚 Documents");
            
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.database_path);
                if ui.button("🔄").clicked() {
                    self.refresh_documents();
                }
            });
            
            ui.separator();
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, (doc_id, document)) in self.documents.iter().enumerate() {
                    let is_selected = self.selected_doc_index == Some(index);
                    
                    // Try to get a display name from the document
                    let display_name = document.get("name")
                        .and_then(|v| match v {
                            Value::String(s) => Some(s.as_str()),
                            _ => None,
                        })
                        .unwrap_or("Unnamed");
                    
                    let label_text = format!("📄 {} ({}:{})", display_name, doc_id.page_id(), doc_id.slot_id());
                    
                    if ui.selectable_label(is_selected, label_text).clicked() {
                        self.selected_doc_index = Some(index);
                        self.edit_mode = false;
                    }
                }
                
                if self.documents.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label("No documents yet");
                        ui.label("Insert some documents to see them here");
                    });
                }
            });
        });
        
        // Central panel - Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🗄️ Rust Database Explorer");
            
            if self.storage_engine.is_none() {
                ui.separator();
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("Welcome to Rust Database Engine!");
                    ui.add_space(20.0);
                    ui.label("Create a new database or open an existing one to get started.");
                    ui.add_space(20.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("🆕 Create New Database").clicked() {
                            self.create_database();
                        }
                        if ui.button("📂 Open Existing Database").clicked() {
                            self.open_database();
                        }
                    });
                });
                return;
            }
            
            ui.separator();
            
            // Document insertion section
            ui.group(|ui| {
                ui.heading("➕ Insert New Document");
                
                ui.horizontal(|ui| {
                    ui.label("Enter JSON document:");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("📋 Examples").clicked() {
                            self.show_example_documents = !self.show_example_documents;
                        }
                    });
                });
                
                // Show example documents if requested
                if self.show_example_documents {
                    ui.collapsing("📄 Example Documents", |ui| {
                        for (i, example) in self.get_example_documents().iter().enumerate() {
                            if ui.button(format!("Use Example {}", i + 1)).clicked() {
                                self.json_input = example.to_string();
                                self.show_example_documents = false;
                            }
                            ui.label(example.lines().next().unwrap_or(""));
                            ui.separator();
                        }
                    });
                }
                
                ui.add(
                    egui::TextEdit::multiline(&mut self.json_input)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_rows(8)
                        .desired_width(f32::INFINITY),
                );
                
                ui.horizontal(|ui| {
                    if ui.button("📤 Insert Document").clicked() {
                        self.insert_document_from_json();
                    }
                    
                    if ui.button("🧹 Clear").clicked() {
                        self.json_input.clear();
                    }
                });
            });
            
            ui.separator();
            
            // Document viewer/editor section
            if let Some(index) = self.selected_doc_index {
                if index < self.documents.len() {
                    // Extract document info to avoid borrowing conflicts
                    let doc_id = self.documents[index].0;
                    let edit_mode = self.edit_mode;
                    
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.heading(format!("📖 Document {}", index + 1));
                            ui.label(format!("({}:{})", doc_id.page_id(), doc_id.slot_id()));
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if !edit_mode {
                                    if ui.button("✏️ Edit").clicked() {
                                        self.edit_mode = true;
                                        // Convert document back to JSON for editing
                                        if let Some(ref mut storage_engine) = self.storage_engine {
                                            match storage_engine.get_document(&doc_id) {
                                                Ok(document) => {
                                                    self.edit_json = Self::document_to_json_string(&document);
                                                }
                                                Err(_) => {
                                                    self.edit_json = format!("{{\n  \"error\": \"Could not load document for editing\"\n}}");
                                                }
                                            }
                                        } else {
                                            self.edit_json = format!("{{\n  \"error\": \"No database connection\"\n}}");
                                        }
                                    }
                                    if ui.button("🗑️ Delete").clicked() {
                                        self.delete_selected_document();
                                    }
                                } else {
                                    if ui.button("💾 Save").clicked() {
                                        self.update_selected_document();
                                    }
                                    if ui.button("❌ Cancel").clicked() {
                                        self.edit_mode = false;
                                    }
                                }
                            });
                        });
                        
                        ui.separator();
                        
                        if edit_mode {
                            ui.label("Edit document (JSON format):");
                            ui.add(
                                egui::TextEdit::multiline(&mut self.edit_json)
                                    .font(egui::TextStyle::Monospace)
                                    .code_editor()
                                    .desired_rows(10)
                                    .desired_width(f32::INFINITY),
                            );
                        } else {
                            ui.label("Document fields:");
                            
                            // Actually fetch and display the document from storage
                            if let Some(ref mut storage_engine) = self.storage_engine {
                                match storage_engine.get_document(&doc_id) {
                                    Ok(document) => {
                                        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                                            egui::Grid::new("document_fields")
                                                .num_columns(2)
                                                .striped(true)
                                                .show(ui, |ui| {
                                                    // Header row
                                                    ui.strong("🔑 Field");
                                                    ui.strong("💎 Value");
                                                    ui.end_row();
                                                    
                                                    // Document ID row
                                                    ui.label("_id");
                                                    if let Some(object_id) = document.get_id() {
                                                        ui.label(format!("{}", object_id));
                                                    } else {
                                                        ui.label("No ID");
                                                    }
                                                    ui.end_row();
                                                    
                                                    // Iterate through all document fields
                                                    for (field_name, field_value) in document.iter() {
                                                        ui.label(field_name);
                                                        ui.label(format!("{}", field_value));
                                                        ui.end_row();
                                                    }
                                                    
                                                    if document.is_empty() {
                                                        ui.label("(empty document)");
                                                        ui.label("-");
                                                        ui.end_row();
                                                    }
                                                });
                                        });
                                    }
                                    Err(e) => {
                                        ui.colored_label(egui::Color32::RED, format!("❌ Error loading document: {}", e));
                                    }
                                }
                            } else {
                                ui.label("❌ No database connection");
                            }
                        }
                    });
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("👈 Select a document from the list");
                    ui.label("Or insert a new document above");
                });
            }
        });
    }
}