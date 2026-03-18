use eframe::egui;
use crate::{
    storage::{
        storage_engine::{StorageEngine, DocumentId},
        file::DatabaseFile,
    },
    Document, Value,
};
use std::path::Path;

#[derive(PartialEq)]
enum ActiveTab {
    Insert,
    View,
}

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
    active_tab: ActiveTab,

    // Edit mode
    edit_mode: bool,
    edit_json: String,
}

impl Default for DatabaseApp {
    fn default() -> Self {
        Self {
            storage_engine: None,
            database_path: std::env::current_dir()
                .map(|cwd| cwd.join("database_ui.db").display().to_string())
                .unwrap_or_else(|_| "database_ui.db".to_string()),
            documents: Vec::new(),
            json_input: String::new(),
            status_message: "No database open. Create or open one to get started.".to_string(),
            status_color: egui::Color32::GRAY,
            selected_doc_index: None,
            active_tab: ActiveTab::Insert,
            edit_mode: false,
            edit_json: String::new(),
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
                self.set_status("Database created successfully.", egui::Color32::from_rgb(100, 220, 120));
                self.refresh_documents();
            }
            Err(e) => {
                self.set_status(&format!("Failed to create database: {}", e), egui::Color32::from_rgb(220, 80, 80));
            }
        }
    }

    fn create_database_internal(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.database_path);
        if path.exists() {
            return Err(format!(
                "A database already exists at \"{}\". Delete it or choose a different path.",
                self.database_path
            ).into());
        }
        let _db_file = DatabaseFile::create(path)?;
        drop(_db_file);
        self.storage_engine = Some(StorageEngine::new(path, 64)?);
        Ok(())
    }

    fn open_database(&mut self) {
        match self.open_database_internal() {
            Ok(_) => {
                self.set_status("Database opened.", egui::Color32::from_rgb(100, 220, 120));
                self.refresh_documents();
            }
            Err(e) => {
                self.set_status(&format!("Failed to open database: {}", e), egui::Color32::from_rgb(220, 80, 80));
            }
        }
    }

    fn open_database_internal(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.database_path);
        self.storage_engine = Some(StorageEngine::new(path, 64)?);
        Ok(())
    }

    fn set_status(&mut self, message: &str, color: egui::Color32) {
        self.status_message = message.to_string();
        self.status_color = color;
    }

    fn refresh_documents(&mut self) {
        self.set_status("Document list refreshed.", egui::Color32::from_rgb(100, 180, 220));
    }

    fn insert_document_from_json(&mut self) {
        if let Some(ref mut engine) = self.storage_engine {
            let json_input = self.json_input.clone();
            match Self::parse_json_to_document(&json_input) {
                Ok(document) => {
                    match engine.insert_document(&document) {
                        Ok(doc_id) => {
                            self.documents.push((doc_id, document));
                            self.set_status(
                                &format!("Inserted document at page {}, slot {}.", doc_id.page_id(), doc_id.slot_id()),
                                egui::Color32::from_rgb(100, 220, 120),
                            );
                            self.json_input.clear();
                        }
                        Err(e) => self.set_status(&format!("Insert failed: {}", e), egui::Color32::from_rgb(220, 80, 80)),
                    }
                }
                Err(e) => self.set_status(&format!("Invalid JSON: {}", e), egui::Color32::from_rgb(220, 80, 80)),
            }
        } else {
            self.set_status("No database open.", egui::Color32::from_rgb(220, 80, 80));
        }
    }

    fn parse_json_to_document(json_str: &str) -> Result<Document, Box<dyn std::error::Error>> {
        let json_value: serde_json::Value = serde_json::from_str(json_str)?;
        let mut document = Document::new();
        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                document.set(&key, Self::json_value_to_db_value(value));
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
            serde_json::Value::Array(_) => Value::String(value.to_string()),
            serde_json::Value::Object(_) => Value::String(value.to_string()),
        }
    }

    fn document_to_json_string(document: &Document) -> String {
        let mut json_obj = serde_json::Map::new();
        for (key, value) in document.iter() {
            json_obj.insert(key.clone(), Self::db_value_to_json_value(value));
        }
        serde_json::to_string_pretty(&json_obj).unwrap_or_else(|_| "{}".to_string())
    }

    fn db_value_to_json_value(value: &Value) -> serde_json::Value {
        match value {
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::I32(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            Value::I64(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            Value::F64(f) => serde_json::Value::Number(
                serde_json::Number::from_f64(*f).unwrap_or_else(|| serde_json::Number::from(0)),
            ),
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
            if let Some(ref mut engine) = self.storage_engine {
                let (doc_id, _) = &self.documents[index];
                match engine.delete_document(doc_id) {
                    Ok(_) => {
                        self.documents.remove(index);
                        self.selected_doc_index = None;
                        self.edit_mode = false;
                        self.active_tab = ActiveTab::Insert;
                        self.set_status("Document deleted.", egui::Color32::from_rgb(100, 220, 120));
                    }
                    Err(e) => self.set_status(&format!("Delete failed: {}", e), egui::Color32::from_rgb(220, 80, 80)),
                }
            }
        }
    }

    fn update_selected_document(&mut self) {
        if let Some(index) = self.selected_doc_index {
            if let Some(ref mut engine) = self.storage_engine {
                let edit_json = self.edit_json.clone();
                match Self::parse_json_to_document(&edit_json) {
                    Ok(new_document) => {
                        let (doc_id, _) = &self.documents[index];
                        let doc_id_copy = *doc_id;
                        match engine.update_document(&doc_id_copy, &new_document) {
                            Ok(new_doc_id) => {
                                self.documents[index] = (new_doc_id, new_document);
                                self.edit_mode = false;
                                self.set_status("Document updated.", egui::Color32::from_rgb(100, 220, 120));
                            }
                            Err(e) => self.set_status(&format!("Update failed: {}", e), egui::Color32::from_rgb(220, 80, 80)),
                        }
                    }
                    Err(e) => self.set_status(&format!("Invalid JSON: {}", e), egui::Color32::from_rgb(220, 80, 80)),
                }
            }
        }
    }

    fn doc_display_name(document: &Document) -> String {
        document.get("name")
            .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
            .or_else(|| {
                document.get("title")
                    .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
            })
            .unwrap_or_else(|| "Untitled".to_string())
    }

    fn doc_field_preview(document: &Document) -> String {
        document.iter()
            .filter(|(k, _)| *k != "name" && *k != "title" && *k != "_id")
            .take(2)
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("  |  ")
    }

    fn example_documents() -> &'static [(&'static str, &'static str)] {
        &[
            ("Person", r#"{ "name": "Alice Johnson", "age": 28, "email": "alice@example.com", "active": true }"#),
            ("Person", r#"{ "name": "Bob Smith", "age": 35, "email": "bob@example.com", "active": false, "salary": 75000 }"#),
            ("Product", r#"{ "product_name": "Laptop", "price": 999.99, "category": "Electronics", "in_stock": true, "quantity": 25 }"#),
            ("Order", r#"{ "order_id": 1042, "customer": "Carol Williams", "total": 149.99, "shipped": false }"#),
        ]
    }
}

impl eframe::App for DatabaseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let accent = egui::Color32::from_rgb(228, 110, 30); // rust orange accent

        // ── Top menu bar ────────────────────────────────────────────────
        egui::TopBottomPanel::top("menu_bar")
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(24, 26, 32)).inner_margin(egui::Margin::symmetric(8.0, 4.0)))
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("Database", |ui| {
                        if ui.button("  New database").clicked() {
                            self.create_database();
                            ui.close_menu();
                        }
                        if ui.button("  Open database").clicked() {
                            self.open_database();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("  Refresh").clicked() {
                            self.refresh_documents();
                            ui.close_menu();
                        }
                    });

                    ui.separator();

                    ui.label(egui::RichText::new(&self.database_path).color(egui::Color32::GRAY).small());

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (dot, label) = if self.storage_engine.is_some() {
                            (egui::Color32::from_rgb(100, 220, 120), "Connected")
                        } else {
                            (egui::Color32::from_rgb(220, 80, 80), "Disconnected")
                        };
                        ui.label(egui::RichText::new(label).color(egui::Color32::GRAY).small());
                        ui.colored_label(dot, "●");
                    });
                });
            });

        // ── Bottom status bar ────────────────────────────────────────────
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(20, 22, 28)).inner_margin(egui::Margin::symmetric(12.0, 6.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(self.status_color, egui::RichText::new(&self.status_message).size(14.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(format!("{} documents", self.documents.len())).color(egui::Color32::GRAY).size(14.0));
                    });
                });
            });

        // ── Left panel: document list ────────────────────────────────────
        egui::SidePanel::left("document_list")
            .resizable(true)
            .default_width(260.0)
            .min_width(180.0)
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(20, 22, 28)).inner_margin(egui::Margin::same(0.0)))
            .show(ctx, |ui| {
                // Panel header
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(24, 26, 32))
                    .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Documents").strong().size(13.0));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new(format!("{}", self.documents.len()))
                                        .color(egui::Color32::GRAY)
                                        .small(),
                                );
                            });
                        });
                    });

                ui.separator();

                // Document list
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    if self.documents.is_empty() {
                        ui.add_space(32.0);
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("No documents").color(egui::Color32::DARK_GRAY));
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new("Insert one to get started").color(egui::Color32::DARK_GRAY).small());
                        });
                        return;
                    }

                    for (index, (doc_id, document)) in self.documents.iter().enumerate() {
                        let is_selected = self.selected_doc_index == Some(index);
                        let display_name = Self::doc_display_name(document);
                        let preview = Self::doc_field_preview(document);

                        let bg = if is_selected {
                            egui::Color32::from_rgb(45, 28, 14)
                        } else {
                            egui::Color32::TRANSPARENT
                        };

                        let response = egui::Frame::none()
                            .fill(bg)
                            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        let name_color = if is_selected { accent } else { egui::Color32::WHITE };
                                        ui.label(egui::RichText::new(&display_name).color(name_color).size(13.0));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(
                                                egui::RichText::new(format!("{}:{}", doc_id.page_id(), doc_id.slot_id()))
                                                    .color(egui::Color32::DARK_GRAY)
                                                    .small(),
                                            );
                                        });
                                    });
                                    if !preview.is_empty() {
                                        ui.label(egui::RichText::new(&preview).color(egui::Color32::GRAY).small());
                                    }
                                });
                            })
                            .response;

                        let clickable = response.interact(egui::Sense::click());
                        if clickable.clicked() {
                            self.selected_doc_index = Some(index);
                            self.edit_mode = false;
                            self.active_tab = ActiveTab::View;
                        }

                        ui.separator();
                    }
                });
            });

        // ── Central panel ─────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(18, 20, 26)).inner_margin(egui::Margin::same(0.0)))
            .show(ctx, |ui| {
                // No DB open: welcome screen
                if self.storage_engine.is_none() {
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(80.0);
                            ui.label(egui::RichText::new("rustdb").size(40.0).strong().color(accent));
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("A document database engine written in Rust").color(egui::Color32::GRAY).size(15.0));
                            ui.add_space(40.0);

                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(24, 26, 32))
                                .rounding(egui::Rounding::same(8.0))
                                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 50, 60)))
                                .inner_margin(egui::Margin::same(24.0))
                                .show(ui, |ui| {
                                    ui.set_width(360.0);
                                    ui.label(egui::RichText::new("Database file").color(egui::Color32::GRAY).small());
                                    ui.add_space(4.0);
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.database_path)
                                            .desired_width(f32::INFINITY)
                                            .font(egui::TextStyle::Monospace),
                                    );
                                    ui.add_space(16.0);
                                    ui.horizontal(|ui| {
                                        if ui.add_sized(
                                            [156.0, 32.0],
                                            egui::Button::new(egui::RichText::new("Create new").color(egui::Color32::WHITE))
                                                .fill(egui::Color32::from_rgb(160, 65, 10)),
                                        ).clicked() {
                                            self.create_database();
                                        }
                                        ui.add_space(8.0);
                                        if ui.add_sized(
                                            [156.0, 32.0],
                                            egui::Button::new("Open existing")
                                                .fill(egui::Color32::from_rgb(35, 38, 48)),
                                        ).clicked() {
                                            self.open_database();
                                        }
                                    });
                                });
                        });
                    });
                    return;
                }

                // Tab bar
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(24, 26, 32))
                    .inner_margin(egui::Margin::symmetric(16.0, 0.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(0.0);
                            let insert_active = self.active_tab == ActiveTab::Insert;
                            let view_active = self.active_tab == ActiveTab::View;

                            let insert_color = if insert_active { accent } else { egui::Color32::GRAY };
                            let view_color = if view_active { accent } else { egui::Color32::GRAY };

                            let insert_btn = ui.add(
                                egui::Button::new(egui::RichText::new("Insert Document").color(insert_color).size(13.0))
                                    .fill(egui::Color32::TRANSPARENT)
                                    .stroke(egui::Stroke::NONE)
                                    .frame(false),
                            );
                            if insert_btn.clicked() {
                                self.active_tab = ActiveTab::Insert;
                            }

                            ui.add_space(8.0);

                            let view_label = if let Some(idx) = self.selected_doc_index {
                                format!("Document #{}", idx + 1)
                            } else {
                                "View Document".to_string()
                            };
                            let view_btn = ui.add(
                                egui::Button::new(egui::RichText::new(&view_label).color(view_color).size(13.0))
                                    .fill(egui::Color32::TRANSPARENT)
                                    .stroke(egui::Stroke::NONE)
                                    .frame(false),
                            );
                            if view_btn.clicked() {
                                self.active_tab = ActiveTab::View;
                            }
                        });
                    });

                ui.separator();

                // Tab content
                match self.active_tab {
                    ActiveTab::Insert => {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add_space(16.0);

                            egui::Frame::none()
                                .inner_margin(egui::Margin::symmetric(24.0, 0.0))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new("JSON Document").color(egui::Color32::GRAY).small());
                                    ui.add_space(6.0);
                                    ui.add(
                                        egui::TextEdit::multiline(&mut self.json_input)
                                            .font(egui::TextStyle::Monospace)
                                            .code_editor()
                                            .desired_rows(12)
                                            .desired_width(f32::INFINITY),
                                    );

                                    ui.add_space(10.0);
                                    ui.horizontal(|ui| {
                                        if ui.add_sized(
                                            [140.0, 30.0],
                                            egui::Button::new("Insert Document")
                                                .fill(egui::Color32::from_rgb(160, 65, 10)),
                                        ).clicked() {
                                            self.insert_document_from_json();
                                        }

                                        if ui.add_sized(
                                            [80.0, 30.0],
                                            egui::Button::new("Clear")
                                                .fill(egui::Color32::from_rgb(35, 38, 48)),
                                        ).clicked() {
                                            self.json_input.clear();
                                        }
                                    });

                                    ui.add_space(24.0);
                                    ui.separator();
                                    ui.add_space(12.0);

                                    ui.label(egui::RichText::new("Examples").color(egui::Color32::GRAY).small());
                                    ui.add_space(8.0);

                                    ui.horizontal_wrapped(|ui| {
                                        for (label, json) in Self::example_documents() {
                                            if ui.add(
                                                egui::Button::new(egui::RichText::new(*label).small())
                                                    .fill(egui::Color32::from_rgb(30, 33, 42))
                                                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 55, 68))),
                                            ).clicked() {
                                                self.json_input = json.to_string();
                                            }
                                        }
                                    });
                                });
                        });
                    }

                    ActiveTab::View => {
                        if let Some(index) = self.selected_doc_index {
                            if index < self.documents.len() {
                                let doc_id = self.documents[index].0;
                                let edit_mode = self.edit_mode;

                                egui::Frame::none()
                                    .inner_margin(egui::Margin::symmetric(24.0, 16.0))
                                    .show(ui, |ui| {
                                        // Doc header + actions
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(Self::doc_display_name(&self.documents[index].1))
                                                    .strong()
                                                    .size(18.0),
                                            );
                                            ui.label(
                                                egui::RichText::new(format!("page {} · slot {}", doc_id.page_id(), doc_id.slot_id()))
                                                    .color(egui::Color32::DARK_GRAY)
                                                    .small(),
                                            );

                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if !edit_mode {
                                                    if ui.add(
                                                        egui::Button::new(egui::RichText::new("Delete").color(egui::Color32::from_rgb(220, 80, 80)).small())
                                                            .fill(egui::Color32::from_rgb(40, 24, 24))
                                                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 30, 30))),
                                                    ).clicked() {
                                                        self.delete_selected_document();
                                                    }
                                                    ui.add_space(8.0);
                                                    if ui.add(
                                                        egui::Button::new(egui::RichText::new("Edit").small())
                                                            .fill(egui::Color32::from_rgb(35, 38, 48))
                                                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 60, 75))),
                                                    ).clicked() {
                                                        self.edit_mode = true;
                                                        if let Some(ref mut engine) = self.storage_engine {
                                                            match engine.get_document(&doc_id) {
                                                                Ok(doc) => self.edit_json = Self::document_to_json_string(&doc),
                                                                Err(_) => self.edit_json = "{}".to_string(),
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    if ui.add(
                                                        egui::Button::new(egui::RichText::new("Cancel").small())
                                                            .fill(egui::Color32::from_rgb(35, 38, 48)),
                                                    ).clicked() {
                                                        self.edit_mode = false;
                                                    }
                                                    ui.add_space(8.0);
                                                    if ui.add(
                                                        egui::Button::new(egui::RichText::new("Save").small())
                                                            .fill(egui::Color32::from_rgb(160, 65, 10)),
                                                    ).clicked() {
                                                        self.update_selected_document();
                                                    }
                                                }
                                            });
                                        });

                                        ui.add_space(12.0);
                                        ui.separator();
                                        ui.add_space(12.0);

                                        if edit_mode {
                                            ui.label(egui::RichText::new("Edit as JSON").color(egui::Color32::GRAY).small());
                                            ui.add_space(6.0);
                                            egui::ScrollArea::vertical().show(ui, |ui| {
                                                ui.add(
                                                    egui::TextEdit::multiline(&mut self.edit_json)
                                                        .font(egui::TextStyle::Monospace)
                                                        .code_editor()
                                                        .desired_rows(16)
                                                        .desired_width(f32::INFINITY),
                                                );
                                            });
                                        } else if let Some(ref mut engine) = self.storage_engine {
                                            match engine.get_document(&doc_id) {
                                                Ok(document) => {
                                                    egui::ScrollArea::vertical().show(ui, |ui| {
                                                        for (field_name, field_value) in document.iter() {
                                                            egui::Frame::none()
                                                                .fill(egui::Color32::from_rgb(22, 24, 30))
                                                                .rounding(egui::Rounding::same(4.0))
                                                                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                                                                .show(ui, |ui| {
                                                                    ui.set_width(ui.available_width());
                                                                    ui.horizontal(|ui| {
                                                                        ui.label(
                                                                            egui::RichText::new(field_name)
                                                                                .color(accent)
                                                                                .small()
                                                                                .monospace(),
                                                                        );
                                                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                            ui.label(
                                                                                egui::RichText::new(format!("{}", field_value))
                                                                                    .monospace()
                                                                                    .small(),
                                                                            );
                                                                        });
                                                                    });
                                                                });
                                                            ui.add_space(4.0);
                                                        }
                                                    });
                                                }
                                                Err(e) => {
                                                    ui.colored_label(
                                                        egui::Color32::from_rgb(220, 80, 80),
                                                        format!("Error loading document: {}", e),
                                                    );
                                                }
                                            }
                                        }
                                    });
                            }
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(80.0);
                                    ui.label(egui::RichText::new("No document selected").color(egui::Color32::DARK_GRAY).size(16.0));
                                    ui.add_space(8.0);
                                    ui.label(egui::RichText::new("Select one from the list on the left").color(egui::Color32::DARK_GRAY).small());
                                });
                            });
                        }
                    }
                }
            });
    }
}
