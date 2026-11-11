//! `wikicraft-app`: Native eframe/egui desktop studio for WikiCraft.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use craft_biz::bus::Bus;
use craft_biz_ui::shell::{self, settings_path, UiSettings};
use craft_biz_ui::theme::{self, Tone};
use craft_biz_ui::widgets;
use eframe::egui::{self, Color32, CornerRadius, RichText, ScrollArea, Stroke};
use std::path::{Path, PathBuf};
use wikicraft_core::{dispatch, extract_wiki_links, Command, Confinement, Ctx};
use wikicraft_store::{Adr, Page, PageSummary, SearchResult, Store, WikiStats};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Pages,
    Adrs,
    Search,
    Graph,
}

struct App {
    ctx: Ctx,
    _db_path: PathBuf,
    settings: UiSettings,
    settings_file: PathBuf,
    view: View,
    stats: WikiStats,
    pages_list: Vec<PageSummary>,
    page_filter: String,
    selected_page: Option<Page>,
    selected_page_backlinks: Vec<String>,
    adrs_list: Vec<Adr>,
    search_query: String,
    search_results: Vec<SearchResult>,
    status_msg: String,
    // Page editing state
    is_editing: bool,
    edit_title: String,
    edit_content: String,
    // Import drawer state
    show_import_bar: bool,
    custom_import_path: String,
}

impl App {
    fn new(ctx: Ctx, db_path: PathBuf) -> Self {
        let s_path = settings_path(&db_path);
        let settings = UiSettings::load(&s_path);
        let mut app = Self {
            ctx,
            _db_path: db_path,
            settings,
            settings_file: s_path,
            view: View::Pages,
            stats: WikiStats {
                page_count: 0,
                adr_count: 0,
                link_count: 0,
            },
            pages_list: Vec::new(),
            page_filter: String::new(),
            selected_page: None,
            selected_page_backlinks: Vec::new(),
            adrs_list: Vec::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            status_msg: "Ready".to_string(),
            is_editing: false,
            edit_title: String::new(),
            edit_content: String::new(),
            show_import_bar: false,
            custom_import_path: String::new(),
        };
        app.reload();
        app
    }

    fn reload(&mut self) {
        if let Ok(st) = self.ctx.store.stats() {
            self.stats = st;
        }
        if let Ok(pages) = self.ctx.store.list_pages() {
            self.pages_list = pages;
            if self.selected_page.is_none() && !self.pages_list.is_empty() {
                let first_id = self.pages_list[0].id.clone();
                self.load_page(&first_id);
            }
        }
        if let Ok(adrs) = self.ctx.store.list_adrs() {
            self.adrs_list = adrs;
        }
    }

    fn load_page(&mut self, id: &str) {
        if let Ok(p) = self.ctx.store.get_page(id) {
            self.edit_title = p.title.clone();
            self.edit_content = p.content.clone();
            self.is_editing = false;
            self.selected_page = Some(p);
            self.selected_page_backlinks = self.ctx.store.get_backlinks(id).unwrap_or_default();
        }
    }

    fn run_search(&mut self) {
        if self.search_query.trim().is_empty() {
            self.search_results.clear();
            return;
        }
        if let Ok(res) = self.ctx.store.search_pages(&self.search_query) {
            self.search_results = res;
        }
    }

    fn seed_demo(&mut self) {
        match dispatch(&Command::SeedDemo, &mut self.ctx) {
            Ok(res) => {
                let pages = res.get("pages_created").and_then(|v| v.as_u64()).unwrap_or(0);
                let adrs = res.get("adrs_created").and_then(|v| v.as_u64()).unwrap_or(0);
                self.reload();
                self.status_msg = format!("🌱 Seeded {pages} pages and {adrs} ADRs successfully");
            }
            Err(e) => {
                self.status_msg = format!("Seed error: {e}");
            }
        }
    }

    fn import_path(&mut self, path: &Path, label: &str) {
        if !path.exists() {
            self.status_msg = format!("Directory not found: {}", path.display());
            return;
        }
        let cmd = Command::TreeImport {
            path: path.to_path_buf(),
        };
        match dispatch(&cmd, &mut self.ctx) {
            Ok(res) => {
                let count = res.get("imported_pages").and_then(|v| v.as_u64()).unwrap_or(0);
                let scanned = res.get("scanned_files").and_then(|v| v.as_u64()).unwrap_or(0);
                self.reload();
                self.status_msg = format!("📚 Imported {count} pages ({scanned} scanned) from {label}");
            }
            Err(e) => {
                self.status_msg = format!("Import error: {e}");
            }
        }
    }

    fn save_current_page(&mut self) {
        if let Some(ref p) = self.selected_page {
            let id = p.id.clone();
            let title = self.edit_title.trim().to_string();
            let content = self.edit_content.clone();

            let cmd = Command::PageSave {
                id: id.clone(),
                title: if title.is_empty() { id.clone() } else { title },
                content,
            };

            match dispatch(&cmd, &mut self.ctx) {
                Ok(_) => {
                    self.load_page(&id);
                    self.reload();
                    self.is_editing = false;
                    self.status_msg = format!("Saved page {id}");
                }
                Err(e) => {
                    self.status_msg = format!("Failed to save page: {e}");
                }
            }
        }
    }
}

fn locate_mcd_root() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from(r"d:\Dev\repos\mcp-central-docs"),
        PathBuf::from("../mcp-central-docs"),
        PathBuf::from("../../mcp-central-docs"),
    ];
    for c in &candidates {
        if c.is_dir() {
            return Some(c.clone());
        }
    }
    None
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        shell::handle_zoom(ui.ctx(), &mut self.settings);
        let t = theme::current(ui.ctx());

        // Header bar
        ui.horizontal(|ui| {
                ui.heading(
                    RichText::new("📖 WikiCraft Studio")
                        .size(19.0)
                        .strong()
                        .color(t.accent),
                );
                ui.separator();

                if ui
                    .selectable_label(self.view == View::Pages, "📄 Pages")
                    .clicked()
                {
                    self.view = View::Pages;
                }
                if ui
                    .selectable_label(self.view == View::Adrs, "🏛️ ADR Matrix")
                    .clicked()
                {
                    self.view = View::Adrs;
                }
                if ui
                    .selectable_label(self.view == View::Search, "🔍 Search")
                    .clicked()
                {
                    self.view = View::Search;
                }
                if ui
                    .selectable_label(self.view == View::Graph, "🕸️ Wiki Graph")
                    .clicked()
                {
                    self.view = View::Graph;
                }

                ui.separator();

                if ui.button("🔄 Refresh").clicked() {
                    self.reload();
                    self.status_msg = "Reloaded".to_string();
                }

                if ui
                    .button("🌱 Seed Demo")
                    .on_hover_text("Insert sample documentation pages, cross-links, and ADRs")
                    .clicked()
                {
                    self.seed_demo();
                }

                let import_text = if self.show_import_bar {
                    "📂 Ingest Docs ▲"
                } else {
                    "📂 Ingest Docs ▼"
                };
                if ui
                    .button(import_text)
                    .on_hover_text("Open MCD and directory import tools")
                    .clicked()
                {
                    self.show_import_bar = !self.show_import_bar;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mode_icon = if self.settings.dark { "☀️ Light" } else { "🌙 Dark" };
                    if ui.button(mode_icon).on_hover_text("Toggle High-Contrast Light/Dark mode").clicked() {
                        let new_dark = !self.settings.dark;
                        shell::set_dark(ui.ctx(), &mut self.settings, new_dark);
                        let _ = self.settings.save(&self.settings_file);
                    }
                    ui.label(
                        RichText::new(format!("Actor: {}", self.ctx.actor))
                            .color(t.text_weak),
                    );
                });
            });

            // Expandable Import Bar
            if self.show_import_bar {
                ui.add_space(4.0);
                egui::Frame::group(ui.style())
                    .fill(t.sunken)
                    .stroke(Stroke::new(1.0, t.border))
                    .corner_radius(CornerRadius::same(6))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("Ingest MCD Sections:").strong().color(t.text));

                            let mcd_opt = locate_mcd_root();
                            if let Some(ref mcd) = mcd_opt {
                                if ui.button("📘 Standards").on_hover_text(format!("Import {}", mcd.join("standards").display())).clicked() {
                                    let path = mcd.join("standards");
                                    self.import_path(&path, "MCD Standards");
                                }
                                if ui.button("📗 Patterns").on_hover_text(format!("Import {}", mcd.join("patterns").display())).clicked() {
                                    let path = mcd.join("patterns");
                                    self.import_path(&path, "MCD Patterns");
                                }
                                if ui.button("📙 Operations").on_hover_text(format!("Import {}", mcd.join("operations").display())).clicked() {
                                    let path = mcd.join("operations");
                                    self.import_path(&path, "MCD Operations");
                                }
                                if ui.button("📕 Troubleshooting").on_hover_text(format!("Import {}", mcd.join("troubleshooting").display())).clicked() {
                                    let path = mcd.join("troubleshooting");
                                    self.import_path(&path, "MCD Troubleshooting");
                                }
                                if ui.button("🏛️ Architecture").on_hover_text(format!("Import {}", mcd.join("architecture").display())).clicked() {
                                    let path = mcd.join("architecture");
                                    self.import_path(&path, "MCD Architecture");
                                }
                            } else {
                                ui.colored_label(Color32::from_rgb(220, 100, 100), "MCD repository not found automatically.");
                            }

                            ui.separator();
                            ui.label(RichText::new("Custom Folder:").color(t.text));
                            let edit = ui.add(
                                egui::TextEdit::singleline(&mut self.custom_import_path)
                                    .hint_text("d:/path/to/markdown/folder")
                                    .desired_width(220.0),
                            );
                            if (ui.button("Import Path").clicked() || (edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                                && !self.custom_import_path.trim().is_empty()
                            {
                                let p = PathBuf::from(self.custom_import_path.trim());
                                self.import_path(&p, "Custom Folder");
                            }
                        });
                    });
            }

            ui.separator();

            let mut nav_to_page = None;

            // Main view switcher
            match self.view {
                View::Pages => {
                    ui.horizontal(|ui| {
                        // Left sidebar: Documentation Tree
                        ui.vertical(|ui| {
                            ui.set_width(280.0);
                            ui.horizontal(|ui| {
                                ui.heading(RichText::new("Documentation Tree").size(15.0).color(t.text));
                                ui.label(RichText::new(format!("({})", self.pages_list.len())).color(t.text_weak));
                            });

                            ui.add(
                                egui::TextEdit::singleline(&mut self.page_filter)
                                    .hint_text("🔍 Filter pages...")
                                    .desired_width(f32::INFINITY),
                            );
                            ui.separator();

                            let filter_lower = self.page_filter.trim().to_lowercase();
                            let mut to_load = None;

                            ScrollArea::vertical()
                                .id_salt("page_list_scroll")
                                .show(ui, |ui| {
                                    for p in &self.pages_list {
                                        if !filter_lower.is_empty()
                                            && !p.title.to_lowercase().contains(&filter_lower)
                                            && !p.id.to_lowercase().contains(&filter_lower)
                                        {
                                            continue;
                                        }

                                        let is_sel = self
                                            .selected_page
                                            .as_ref()
                                            .map(|sp| sp.id == p.id)
                                            .unwrap_or(false);

                                        let label = format!("{} (v{})", p.title, p.version);
                                        if ui.selectable_label(is_sel, label).clicked() {
                                            to_load = Some(p.id.clone());
                                        }
                                    }
                                });

                            if let Some(id) = to_load {
                                self.load_page(&id);
                            }
                        });

                        ui.separator();

                        // Right pane: Detail / Viewer / Editor
                        ui.vertical(|ui| {
                            let page_info = self.selected_page.as_ref().map(|p| {
                                (p.id.clone(), p.title.clone(), p.content.clone(), p.version, p.updated_at.clone())
                            });

                            if let Some((page_id, page_title, page_content, page_version, page_updated_at)) = page_info {
                                ui.horizontal(|ui| {
                                    ui.heading(
                                        RichText::new(&page_title)
                                            .size(21.0)
                                            .strong()
                                            .color(t.text),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if self.is_editing {
                                                if ui.button("💾 Save").clicked() {
                                                    self.save_current_page();
                                                }
                                                if ui.button("Cancel").clicked() {
                                                    self.is_editing = false;
                                                    self.edit_title = page_title.clone();
                                                    self.edit_content = page_content.clone();
                                                }
                                            } else if ui.button("✏️ Edit Page").clicked() {
                                                self.is_editing = true;
                                            }
                                        },
                                    );
                                });

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("ID: {page_id}")).color(t.text_weak));
                                    ui.separator();
                                    ui.label(RichText::new(format!("Version: {page_version}")).color(t.text_weak));
                                    ui.separator();
                                    ui.label(RichText::new(format!("Updated: {page_updated_at}")).color(t.text_weak));
                                });

                                ui.separator();

                                if self.is_editing {
                                    ui.label(RichText::new("Title:").strong().color(t.text));
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.edit_title)
                                            .desired_width(f32::INFINITY),
                                    );
                                    ui.add_space(4.0);
                                    ui.label(RichText::new("Markdown Content:").strong().color(t.text));
                                    ScrollArea::vertical()
                                        .id_salt("page_edit_scroll")
                                        .show(ui, |ui| {
                                            ui.add(
                                                egui::TextEdit::multiline(&mut self.edit_content)
                                                    .font(egui::TextStyle::Monospace)
                                                    .desired_width(f32::INFINITY)
                                                    .desired_rows(22),
                                            );
                                        });
                                } else {
                                    ScrollArea::vertical()
                                        .id_salt("page_view_scroll")
                                        .show(ui, |ui| {
                                            let mut in_code_block = false;
                                            let mut code_lines = Vec::new();

                                            for line in page_content.lines() {
                                                let trimmed = line.trim();

                                                if trimmed.starts_with("```") {
                                                    if in_code_block {
                                                        // End of code block
                                                        let block_text = code_lines.join("\n");
                                                        egui::Frame::group(ui.style())
                                                            .fill(t.sunken)
                                                            .stroke(Stroke::new(1.0, t.border))
                                                            .corner_radius(CornerRadius::same(4))
                                                            .show(ui, |ui| {
                                                                ui.label(
                                                                    RichText::new(block_text)
                                                                        .monospace()
                                                                        .color(t.text),
                                                                );
                                                            });
                                                        code_lines.clear();
                                                        in_code_block = false;
                                                    } else {
                                                        in_code_block = true;
                                                    }
                                                    continue;
                                                }

                                                if in_code_block {
                                                    code_lines.push(line);
                                                    continue;
                                                }

                                                if trimmed.starts_with("# ") {
                                                    let h = trimmed.trim_start_matches("# ").trim();
                                                    ui.heading(
                                                        RichText::new(h)
                                                            .size(22.0)
                                                            .strong()
                                                            .color(t.text),
                                                    );
                                                } else if trimmed.starts_with("## ") {
                                                    let h = trimmed.trim_start_matches("## ").trim();
                                                    ui.heading(
                                                        RichText::new(h)
                                                            .size(18.0)
                                                            .strong()
                                                            .color(t.accent),
                                                    );
                                                } else if trimmed.starts_with("### ") {
                                                    let h = trimmed.trim_start_matches("### ").trim();
                                                    ui.label(
                                                        RichText::new(h)
                                                            .size(15.0)
                                                            .strong()
                                                            .color(t.text),
                                                    );
                                                } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                                                    let item = &trimmed[2..];
                                                    ui.horizontal(|ui| {
                                                        ui.label(RichText::new("•").color(t.accent));
                                                        render_markdown_line(ui, item, &t, &mut nav_to_page);
                                                    });
                                                } else if trimmed.is_empty() {
                                                    ui.add_space(6.0);
                                                } else {
                                                    ui.horizontal_wrapped(|ui| {
                                                        render_markdown_line(ui, trimmed, &t, &mut nav_to_page);
                                                    });
                                                }
                                            }

                                            if in_code_block && !code_lines.is_empty() {
                                                let block_text = code_lines.join("\n");
                                                egui::Frame::group(ui.style())
                                                    .fill(t.sunken)
                                                    .stroke(Stroke::new(1.0, t.border))
                                                    .corner_radius(CornerRadius::same(4))
                                                    .show(ui, |ui| {
                                                        ui.label(
                                                            RichText::new(block_text)
                                                                .monospace()
                                                                .color(t.text),
                                                        );
                                                    });
                                            }
                                        });

                                    if !self.selected_page_backlinks.is_empty() {
                                        ui.separator();
                                        ui.horizontal_wrapped(|ui| {
                                            ui.label(RichText::new("Backlinks:").strong().color(t.text));
                                            for bl in &self.selected_page_backlinks {
                                                if ui.button(RichText::new(bl).color(t.accent)).clicked() {
                                                    nav_to_page = Some(bl.clone());
                                                }
                                            }
                                        });
                                    }
                                }
                            } else {
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        RichText::new("Select a page from the left tree or click '🌱 Seed Demo' to load initial content.")
                                            .color(t.text_weak),
                                    );
                                });
                            }
                        });
                    });
                }

                View::Adrs => {
                    ui.heading(
                        RichText::new("Architecture Decision Records (ADRs)")
                            .size(19.0)
                            .strong()
                            .color(t.text),
                    );
                    ui.label(
                        RichText::new("Formal lifecycle records: proposed → accepted/rejected → superseded")
                            .color(t.text_weak),
                    );
                    ui.separator();

                    ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("adr_grid")
                            .striped(true)
                            .spacing([24.0, 10.0])
                            .show(ui, |ui| {
                                ui.label(RichText::new("ID").strong().color(t.text));
                                ui.label(RichText::new("Title").strong().color(t.text));
                                ui.label(RichText::new("Status").strong().color(t.text));
                                ui.label(RichText::new("Decision").strong().color(t.text));
                                ui.label(RichText::new("Updated").strong().color(t.text));
                                ui.end_row();

                                for adr in &self.adrs_list {
                                    ui.label(RichText::new(&adr.id).strong().color(t.text));
                                    ui.label(RichText::new(&adr.title).color(t.text));

                                    let tone = match adr.status.as_str() {
                                        "accepted" => Tone::Ok,
                                        "proposed" => Tone::Warn,
                                        "rejected" => Tone::Danger,
                                        "superseded" => Tone::Neutral,
                                        _ => Tone::Info,
                                    };
                                    widgets::pill(ui, &adr.status, tone);

                                    ui.label(RichText::new(&adr.decision).color(t.text));
                                    ui.label(RichText::new(&adr.updated_at).color(t.text_weak));
                                    ui.end_row();
                                }
                            });
                    });
                }

                View::Search => {
                    ui.heading(
                        RichText::new("Full-Text Documentation Search (SQLite FTS5)")
                            .size(19.0)
                            .strong()
                            .color(t.text),
                    );
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Query:").strong().color(t.text));
                        let edit = ui.add(
                            egui::TextEdit::singleline(&mut self.search_query)
                                .hint_text("Enter keywords (e.g. SQLite, standard, ADR)...")
                                .desired_width(320.0),
                        );
                        if edit.changed() || ui.button("Search").clicked() {
                            self.run_search();
                        }
                    });

                    ui.add_space(8.0);
                    ScrollArea::vertical()
                        .id_salt("search_scroll")
                        .show(ui, |ui| {
                            if self.search_results.is_empty() && !self.search_query.trim().is_empty() {
                                ui.label(RichText::new("No matching documents found.").color(t.text_weak));
                            }

                            for r in &self.search_results {
                                egui::Frame::group(ui.style())
                                    .fill(t.surface)
                                    .stroke(Stroke::new(1.0, t.border))
                                    .corner_radius(CornerRadius::same(6))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(&r.title)
                                                    .strong()
                                                    .size(16.0)
                                                    .color(t.text),
                                            );
                                            ui.label(RichText::new(format!("({})", r.id)).color(t.text_weak));

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui.button("Open Page →").clicked() {
                                                        nav_to_page = Some(r.id.clone());
                                                    }
                                                },
                                            );
                                        });
                                        ui.add_space(4.0);
                                        ui.label(RichText::new(&r.snippet).italics().color(t.text));
                                    });
                                ui.add_space(6.0);
                            }
                        });
                }

                View::Graph => {
                    ui.heading(
                        RichText::new("Wiki-Link Topology & Backlink Graph")
                            .size(19.0)
                            .strong()
                            .color(t.text),
                    );
                    ui.separator();
                    ui.label(
                        RichText::new(format!(
                            "Total tracked bidirectional links: {}",
                            self.stats.link_count
                        ))
                        .color(t.text_weak),
                    );
                    ui.add_space(8.0);

                    ScrollArea::vertical()
                        .id_salt("graph_scroll")
                        .show(ui, |ui| {
                            for p in &self.pages_list {
                                let outgoing = self.ctx.store.get_outgoing_links(&p.id).unwrap_or_default();
                                let backlinks = self.ctx.store.get_backlinks(&p.id).unwrap_or_default();

                                if !outgoing.is_empty() || !backlinks.is_empty() {
                                    egui::Frame::group(ui.style())
                                        .fill(t.surface)
                                        .stroke(Stroke::new(1.0, t.border))
                                        .corner_radius(CornerRadius::same(6))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new(&p.title).strong().size(15.0).color(t.text));
                                                if ui.button("Jump →").clicked() {
                                                    nav_to_page = Some(p.id.clone());
                                                }
                                            });

                                            if !outgoing.is_empty() {
                                                ui.horizontal_wrapped(|ui| {
                                                    ui.label(RichText::new("Outgoing:").color(t.text_weak));
                                                    for out in &outgoing {
                                                        if ui.button(RichText::new(out).color(t.accent)).clicked() {
                                                            nav_to_page = Some(out.clone());
                                                        }
                                                    }
                                                });
                                            }

                                            if !backlinks.is_empty() {
                                                ui.horizontal_wrapped(|ui| {
                                                    ui.label(RichText::new("Backlinks:").color(t.text_weak));
                                                    for bl in &backlinks {
                                                        if ui.button(RichText::new(bl).color(t.accent_soft)).clicked() {
                                                            nav_to_page = Some(bl.clone());
                                                        }
                                                    }
                                                });
                                            }
                                        });
                                    ui.add_space(6.0);
                                }
                            }
                        });
                }
            }

            if let Some(target) = nav_to_page {
                self.load_page(&target);
                self.view = View::Pages;
            }

            // Bottom status bar
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "Pages: {} | ADRs: {} | Links: {}",
                        self.stats.page_count, self.stats.adr_count, self.stats.link_count
                    ))
                    .color(t.text_weak),
                );
                ui.separator();
                ui.label(RichText::new(&self.status_msg).color(t.text));
            });
    }
}

fn render_markdown_line(
    ui: &mut egui::Ui,
    text: &str,
    t: &theme::Tokens,
    nav_to_page: &mut Option<String>,
) {
    let links = extract_wiki_links(text);
    if links.is_empty() {
        ui.label(RichText::new(text).color(t.text));
        return;
    }

    let mut remaining = text;
    while let Some(start) = remaining.find("[[") {
        let before = &remaining[..start];
        if !before.is_empty() {
            ui.label(RichText::new(before).color(t.text));
        }
        remaining = &remaining[start + 2..];
        if let Some(end) = remaining.find("]]") {
            let target_raw = &remaining[..end];
            let (target, display) = match target_raw.split_once('|') {
                Some((tgt, label)) => (tgt.trim(), label.trim()),
                None => (target_raw.trim(), target_raw.trim()),
            };

            if ui.button(RichText::new(format!("🔗 {display}")).color(t.accent)).clicked() {
                *nav_to_page = Some(target.to_string());
            }

            remaining = &remaining[end + 2..];
        } else {
            break;
        }
    }

    if !remaining.is_empty() {
        ui.label(RichText::new(remaining).color(t.text));
    }
}

fn default_db() -> PathBuf {
    if let Ok(p) = std::env::var("WIKICRAFT_DB") {
        if !p.trim().is_empty() {
            return PathBuf::from(p);
        }
    }
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("wikicraft").join("wikicraft.db")
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--smoke") {
        println!("smoke: ok (wikicraft-app)");
        return Ok(());
    }

    let db_path = default_db();
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let store = match Store::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to open database at {}: {e}", db_path.display());
            std::process::exit(1);
        }
    };

    let ctx = Ctx {
        store,
        confinement: Confinement(None),
        bus: Bus::new(Bus::default_root()),
        actor: std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "desktop".to_string()),
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1160.0, 740.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("WikiCraft Studio"),
        ..Default::default()
    };

    let app_db_path = db_path.clone();
    eframe::run_native(
        "WikiCraft",
        native_options,
        Box::new(move |cc| {
            let app = App::new(ctx, app_db_path);
            shell::install(&cc.egui_ctx, &app.settings);
            Ok(Box::new(app))
        }),
    )
}
