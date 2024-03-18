use hframe::Aware;

const IFRAME: &str = r#"
<iframe src="https://www.example.com/"></iframe>
"#;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PeakApp {
    #[serde(skip)]
    dropped_files: Vec<egui::DroppedFile>,
    #[serde(skip)]
    filename: String,
    #[serde(skip)]
    nothing_loaded: bool,
    email_raw: String,
    email_body: String,
    email_body_visible: bool,
    email_html_visible: bool,
    #[serde(skip)]
    headers: Vec<eml_parser::eml::HeaderField>,
    headers_visible: bool,
    recipients: String,
    recipients_visible: bool,
    senders: String,
    senders_visible: bool
}

impl Default for PeakApp {
    fn default() -> Self {
        Self {
            dropped_files: Vec::new(),
            filename: "Email".to_string(),
            nothing_loaded: true,
            email_raw: "No email file loaded".to_string(),
            email_body: r#"<iframe>Body Text</iframe>"#.to_string(),
            email_body_visible: false,
            email_html_visible: false,
            headers: Vec::new(),
            headers_visible: false,
            recipients: "example@recipient.com".to_string(),
            recipients_visible: false,
            senders: "example@sender.com".to_string(),
            senders_visible: false
        }
    }
}

impl PeakApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for PeakApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {     
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);

                ui.horizontal(|ui| {
                    if ui.small_button("Bring all to front").clicked() {
                        self.email_body_visible = true;
                        self.headers_visible = true;
                        self.email_html_visible = true;
                        self.recipients_visible = true;
                        self.senders_visible = true;
                    }
                    if ui.small_button("Hide all").clicked() {
                        self.email_body_visible = false;
                        self.headers_visible = false;
                        self.email_html_visible = false;
                        self.recipients_visible = false;
                        self.senders_visible = false;
                    }
                    if ui.small_button("Body").clicked() {
                        self.email_body_visible = !&self.email_body_visible;
                    }
                    if ui.small_button("HTML").clicked() {
                        self.email_html_visible = !&self.email_html_visible;
                    }
                    if ui.small_button("Headers").clicked() {
                        self.headers_visible = !&self.headers_visible;
                    }
                    if ui.small_button("Recipients").clicked() {
                        self.recipients_visible = !&self.recipients_visible;
                    }
                    if ui.small_button("Senders").clicked() {
                        self.senders_visible = !&self.senders_visible;
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("PEAK - Phishing Email Analysis Kit");

            egui::Window::new(&self.filename)
            .open(&mut self.email_body_visible)
            .scroll2(true)
            .max_height(800.0)
            .show(ctx, |ui| {
                ui.label(&self.email_body);
            })
            .aware();

            hframe::HtmlWindow::new("Email Body")
            .open(&mut self.email_html_visible)
            .content(&self.email_body).show(ctx);

            egui::Window::new("Headers")
            .open(&mut self.headers_visible)
            .scroll2(true)
            .max_height(800.0)
            .show(ctx, |ui| {
                //ui.label(&self.headers);
                let mut i = 1;
                for header in &self.headers {
                    //ui.label(format!("{} - {}", header.name, header.value));
                    egui::CollapsingHeader::new(header.name.to_string())
                        .id_source(egui::Id::new(i))
                        .show(ui, |ui| { 
                            ui.label(header.value.to_string());
                        });
                    i += 1;
                }
            })
            .aware();

            egui::Window::new("Senders")
            .open(&mut self.senders_visible)
            .show(ctx, |ui| {
                ui.label(&self.senders);
            })
            .aware();

            egui::Window::new("Recipients")
            .open(&mut self.recipients_visible)
            .show(ctx, |ui| {
                ui.label(&self.recipients);
            })
            .aware();

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
    
                ui.label("Drag-and-drop .eml files onto the window!");

                ui.add_space(20.0);
    
                // Show dropped files (if any):
                if !self.dropped_files.is_empty() {
                    ui.group(|ui| {
                        ui.set_max_width(600.0);

                        ui.label("Dropped files:");
    
                        for file in &self.dropped_files {
                            let mut info = if let Some(path) = &file.path {
                                self.nothing_loaded = false;
                                path.display().to_string()
                            } else if !file.name.is_empty() {
                                self.nothing_loaded = false;
                                file.name.clone()
                            } else {
                                "???".to_owned()
                            };
    
                            let mut additional_info = vec![];
                            if !file.mime.is_empty() {
                                additional_info.push(format!("type: {}", file.mime));
                            }
                            if let Some(bytes) = &file.bytes {
                                additional_info.push(format!("{} bytes", bytes.len()));
                            }
                            if !additional_info.is_empty() {
                                info += &format!(" ({})", additional_info.join(", "));
                            }
    
                            ui.label(info);
                        }
                    });
                }


                ui.add_space(20.0);

                if ui.add_enabled(!self.nothing_loaded, egui::Button::new("Analyze!").min_size(egui::Vec2::new(100.0, 50.0))).clicked() {
                    self.email_body_visible = true;
                    self.email_html_visible = true;
                    self.headers_visible = true;
                    self.recipients_visible = true;
                    self.senders_visible = true;

                    for file in &self.dropped_files {
                        self.filename = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };

                        self.email_raw = if let Some(bytes) = &file.bytes {
                            std::str::from_utf8(bytes).unwrap().to_string()
                        } else {
                            "No email file loaded".to_owned()
                        };
                    }

                    let eml = eml_parser::EmlParser::from_string(self.email_raw.clone())
                        .with_body()
                        .parse();
                    
                    if eml.is_ok() {
                        let eml = eml.unwrap();

                        self.email_body = if let Some (body) = eml.body {
                            body.to_string()
                        } else {
                            "Failed to parse body".to_owned()
                        };
                        
                        //todo: convert body to html, might already be??

                        self.headers = eml.headers;

                        self.recipients = if let Some (recipient) = eml.from {
                            recipient.to_string()
                        } else {
                            "Failed to parse recipient".to_owned()
                        };

                        self.senders = if let Some (senders) = eml.to {
                            senders.to_string()
                        } else {
                            "Failed to parse recipient".to_owned()
                        };
                    }
                }
            });

            ui.add_space(20.0);

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/tagumi/PEAK/blob/master/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
            
        });

        hframe::sync(ctx);

        preview_files_being_dropped(ctx);

        // Collect dropped files:
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                self.dropped_files = i.raw.dropped_files.clone();
            }
        });

    }
}

fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::*;
    use std::fmt::Write as _;

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = ctx.input(|i| {
            let mut text = "Dropping files:\n".to_owned();
            for file in &i.raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }
            text
        });

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}