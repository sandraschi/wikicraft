//! `wikicraft-app`: Native eframe/egui desktop studio for WikiCraft.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use craft_biz::bus::Bus;
use eframe::egui::{self, Color32, RichText};
use std::path::PathBuf;
use wikicraft_core::{Confinement, Ctx};
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
    view: View,
    stats: WikiStats,
    pages_list: Vec<PageSummary>,
    selected_page: Option<Page>,
    selected_page_backlinks: Vec<String>,
    adrs_list: Vec<Adr>,
    search_query: String,
    search_results: Vec<SearchResult>,
    status_msg: String,
}

impl App {
    fn new(ctx: Ctx) -> Self {
        let mut app = Self {
            ctx,
            view: View::Pages,
            stats: WikiStats {
                page_count: 0,
                adr_count: 0,
                link_count: 0,
            },
            pages_list: Vec::new(),
            selected_page: None,
            selected_page_backlinks: Vec::new(),
            adrs_list: Vec::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            status_msg: "Ready".to_string(),
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
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("📖 WikiCraft Studio")
                    .color(Color32::from_rgb(56, 189, 248))
                    .strong(),
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
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("Actor: {}", self.ctx.actor));
            });
        });

        ui.separator();

        match self.view {
            View::Pages => {
                ui.horizontal(|ui| {
                    // Left list
                    ui.vertical(|ui| {
                        ui.set_width(260.0);
                        ui.heading(RichText::new("Documentation Tree").size(15.0));
                        ui.separator();

                        let mut to_load = None;
                        egui::ScrollArea::vertical()
                            .id_salt("page_list_scroll")
                            .show(ui, |ui| {
                                for p in &self.pages_list {
                                    let is_sel = self
                                        .selected_page
                                        .as_ref()
                                        .map(|sp| sp.id == p.id)
                                        .unwrap_or(false);
                                    if ui
                                        .selectable_label(
                                            is_sel,
                                            format!("{} (v{})", p.title, p.version),
                                        )
                                        .clicked()
                                    {
                                        to_load = Some(p.id.clone());
                                    }
                                }
                            });
                        if let Some(id) = to_load {
                            self.load_page(&id);
                        }
                    });

                    ui.separator();

                    // Right detail
                    ui.vertical(|ui| {
                        if let Some(ref p) = self.selected_page {
                            ui.heading(RichText::new(&p.title).size(20.0));
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("ID: {}", p.id))
                                        .color(Color32::from_rgb(140, 140, 160)),
                                );
                                ui.label(
                                    RichText::new(format!("Version: {}", p.version))
                                        .color(Color32::from_rgb(140, 140, 160)),
                                );
                                ui.label(
                                    RichText::new(format!("Updated: {}", p.updated_at))
                                        .color(Color32::from_rgb(140, 140, 160)),
                                );
                            });
                            ui.separator();

                            ui.label(RichText::new("Markdown Content:").strong());
                            egui::ScrollArea::vertical()
                                .id_salt("page_content_scroll")
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut p.content.clone())
                                            .font(egui::TextStyle::Monospace)
                                            .desired_width(f32::INFINITY)
                                            .interactive(false),
                                    );
                                });

                            if !self.selected_page_backlinks.is_empty() {
                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Backlinks:").strong());
                                    for bl in &self.selected_page_backlinks {
                                        ui.colored_label(Color32::from_rgb(100, 180, 240), bl);
                                    }
                                });
                            }
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.label("Select a page from the list to preview.");
                            });
                        }
                    });
                });
            }

            View::Adrs => {
                ui.heading(RichText::new("Architecture Decision Records (ADRs)").size(18.0));
                ui.separator();

                egui::Grid::new("adr_grid")
                    .striped(true)
                    .spacing([20.0, 10.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("ID").strong());
                        ui.label(RichText::new("Title").strong());
                        ui.label(RichText::new("Status").strong());
                        ui.label(RichText::new("Decision").strong());
                        ui.label(RichText::new("Updated").strong());
                        ui.end_row();

                        for adr in &self.adrs_list {
                            ui.label(&adr.id);
                            ui.label(&adr.title);

                            let color = match adr.status.as_str() {
                                "accepted" => Color32::from_rgb(46, 204, 113),
                                "proposed" => Color32::from_rgb(241, 196, 15),
                                "rejected" => Color32::from_rgb(231, 76, 60),
                                "superseded" => Color32::from_rgb(149, 165, 166),
                                _ => Color32::WHITE,
                            };
                            ui.colored_label(color, format!("[{}]", adr.status));
                            ui.label(&adr.decision);
                            ui.label(&adr.updated_at);
                            ui.end_row();
                        }
                    });
            }

            View::Search => {
                ui.heading(
                    RichText::new("Full-Text Documentation Search (SQLite FTS5)").size(18.0),
                );
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Search query:");
                    let edit = ui.text_edit_singleline(&mut self.search_query);
                    if edit.changed() || ui.button("Search").clicked() {
                        self.run_search();
                    }
                });

                ui.add_space(8.0);
                egui::ScrollArea::vertical()
                    .id_salt("search_scroll")
                    .show(ui, |ui| {
                        for r in &self.search_results {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&r.title).strong().size(15.0));
                                    ui.label(
                                        RichText::new(&r.id)
                                            .color(Color32::from_rgb(140, 140, 160)),
                                    );
                                });
                                ui.label(RichText::new(&r.snippet).italics());
                            });
                            ui.add_space(4.0);
                        }
                    });
            }

            View::Graph => {
                ui.heading(RichText::new("Wiki-Link Connections & Topology").size(18.0));
                ui.separator();
                ui.label(format!(
                    "Total tracked bi-directional links: {}",
                    self.stats.link_count
                ));
                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .id_salt("graph_scroll")
                    .show(ui, |ui| {
                        for p in &self.pages_list {
                            let outgoing =
                                self.ctx.store.get_outgoing_links(&p.id).unwrap_or_default();
                            let backlinks =
                                self.ctx.store.get_backlinks(&p.id).unwrap_or_default();

                            if !outgoing.is_empty() || !backlinks.is_empty() {
                                ui.group(|ui| {
                                    ui.label(RichText::new(&p.title).strong());
                                    if !outgoing.is_empty() {
                                        ui.horizontal(|ui| {
                                            ui.label("Links to:");
                                            for out in &outgoing {
                                                ui.colored_label(
                                                    Color32::from_rgb(52, 152, 219),
                                                    out,
                                                );
                                            }
                                        });
                                    }
                                    if !backlinks.is_empty() {
                                        ui.horizontal(|ui| {
                                            ui.label("Referenced by:");
                                            for bl in &backlinks {
                                                ui.colored_label(
                                                    Color32::from_rgb(155, 89, 182),
                                                    bl,
                                                );
                                            }
                                        });
                                    }
                                });
                                ui.add_space(4.0);
                            }
                        }
                    });
            }
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!(
                "Pages: {} | ADRs: {} | Links: {}",
                self.stats.page_count, self.stats.adr_count, self.stats.link_count
            ));
            ui.separator();
            ui.label(&self.status_msg);
        });
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
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("WikiCraft Studio"),
        ..Default::default()
    };

    eframe::run_native(
        "WikiCraft",
        native_options,
        Box::new(|_cc| Ok(Box::new(App::new(ctx)))),
    )
}
