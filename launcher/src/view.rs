//! Every screen. Same shell, spacing and widgets as the reference design; all colours,
//! type and widgets come from theme.rs.
use crate::theme::*;
use crate::flow::{self, Stage, World};
use crate::game;
use crate::{Launcher, LogKind, Page, RunKind, GAME_EXE, KOFI_URL, PAYPAL_URL, PLAYABLE_PERCENT, REPO_URL, VERSION};
use eframe::egui::{self, pos2, vec2, Align2, Color32, Rect, Sense, Stroke, Ui};
use std::time::{Duration, Instant};

const TABS: [&str; 3] = ["Game", "Options", "About"];

/// (kind, status, title, summary)
const NEWS: [(&str, &str, &str, &str); 6] = [
    ("MILESTONE", "DONE", "Menus and loading screens render", "The title artwork, first-boot setup, legal notices and loading screens draw natively at a locked 30 fps."),
    ("MILESTONE", "DONE", "Open world loads", "With the Series S memory profile the game enters the open world and its frame graph runs, about 190 render targets per frame."),
    ("MILESTONE", "DONE", "First world geometry captured", "One of the game's own render passes draws the ground in true 3D perspective on an NVIDIA GPU."),
    ("TOOLING", "NEW", "Shader precompile resumes", "The shader and pipeline cache is built once, in the background of your own time, and picks up where it stopped."),
    ("TOOLING", "NEW", "One-file launcher", "The runtime ships inside this launcher. Pick your game folder and press Play."),
    ("NEXT", "IN PROGRESS", "Geometry passes feed the final image", "Wave64 mesh shaders are the remaining wall. Translating them is the next big step."),
];

fn window_controls(ui: &mut Ui, r: Rect, ctx: &egui::Context) {
    if icon_button(ui, Rect::from_min_size(pos2(r.max.x - 76., 12.), vec2(32., 32.)), "win-min", Icon::Minimize, false, "") {
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
    }
    if icon_button(ui, Rect::from_min_size(pos2(r.max.x - 42., 12.), vec2(32., 32.)), "win-close", Icon::Close, true, "") {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

fn cover(ui: &mut Ui, tex: &egui::TextureHandle, rect: Rect, radius: u8, tint: Color32) {
    let [iw, ih] = tex.size();
    let img_aspect = iw as f32 / ih as f32;
    let aspect = rect.width() / rect.height();
    let (uw, uh) = if aspect > img_aspect { (1., img_aspect / aspect) } else { (aspect / img_aspect, 1.) };
    let (cx, cy) = (0.5, 0.5);
    let uv = Rect::from_min_max(pos2(cx - uw / 2., cy - uh / 2.), pos2(cx + uw / 2., cy + uh / 2.));
    egui::Image::new((tex.id(), rect.size())).uv(uv).corner_radius(radius).tint(tint).paint_at(ui, rect);
}

impl Launcher {
    fn animating(&self) -> bool {
        self.running.is_some()
    }

    pub(crate) fn draw(&mut self, ctx: &egui::Context) {
        ctx.request_repaint_after(Duration::from_millis(if self.animating() { 16 } else { 500 }));
        let time = self.started.elapsed().as_secs_f32();
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ctx, |ui| {
            let r = ui.max_rect();
            let p = ui.painter().clone();
            p.rect_filled(r, RADIUS_WINDOW, BG_000);
            let drag = ui.interact(Rect::from_min_max(pos2(SIDEBAR, 0.), pos2(r.max.x - 90., 44.)), ui.id().with("drag"), Sense::drag());
            if drag.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            let fade_in = ease(self.page_at.elapsed().as_secs_f32() / 0.2);
            if fade_in < 1. {
                ctx.request_repaint();
            }
            self.sidebar(ui, r);
            self.top_bar(ui, r);
            let area = Rect::from_min_max(pos2(SIDEBAR + MARGIN, 0.), pos2(r.max.x - MARGIN, r.max.y - MARGIN));
            let mut page = ui.new_child(egui::UiBuilder::new().max_rect(area.translate(vec2(0., 6. * (1. - fade_in)))));
            page.set_opacity(fade_in);
            match self.page {
                Page::Play => self.play_page(&mut page, area, time),
                Page::News => self.news_page(&mut page, area),
                Page::Setup => self.setup_page(&mut page, area, time),
                Page::Logs => self.logs_page(&mut page, area),
                Page::Links => self.links_page(&mut page, area),
                Page::Settings => self.settings_page(&mut page, area),
            }
            window_controls(ui, r, ctx);
            p.rect_stroke(r, RADIUS_WINDOW, Stroke::new(1.0_f32, LINE), egui::StrokeKind::Inside);
        });
        if self.confirm_precompile {
            self.precompile_modal(ctx);
        }
    }

    // ------------------------------------------------------------------ shell

    fn sidebar(&mut self, ui: &mut Ui, r: Rect) {
        let p = ui.painter().clone();
        let bar = Rect::from_min_max(r.min, pos2(SIDEBAR, r.max.y));
        p.rect_filled(bar, egui::CornerRadius { nw: RADIUS_WINDOW, sw: RADIUS_WINDOW, ne: 0, se: 0 }, BG_100);
        p.line_segment([pos2(SIDEBAR - 0.5, 0.), pos2(SIDEBAR - 0.5, r.max.y)], Stroke::new(1.0_f32, LINE));
        p.image(self.mark.id(), Rect::from_center_size(pos2(SIDEBAR / 2., 32.), vec2(36., 36.)), Rect::from_min_max(pos2(0., 0.), pos2(1., 1.)), Color32::WHITE);
        let item = |y: f32| Rect::from_center_size(pos2(SIDEBAR / 2., y), vec2(40., 40.));
        let entries = [
            (80., Page::Play, Icon::Play, "Play"),
            (128., Page::News, Icon::News, "Progress notes"),
            (176., Page::Setup, Icon::Download, "Setup"),
            (224., Page::Logs, Icon::Server, "Logs"),
            (272., Page::Links, Icon::Friends, "Links"),
        ];
        for (y, page, icon, tip) in entries {
            if nav_item(ui, item(y), ("nav", y as i32), icon, tip, self.page == page, false) {
                self.go(page);
            }
        }
        if nav_item(ui, item(r.max.y - 92.), "nav-settings", Icon::Gear, "Settings", self.page == Page::Settings, false) {
            self.go(Page::Settings);
        }
        let copied = self.copied_at.is_some_and(|t| t.elapsed() < Duration::from_millis(1500));
        let vr = Rect::from_center_size(pos2(SIDEBAR / 2., r.max.y - 44.), vec2(SIDEBAR - 6., 30.));
        let hit = ui.interact(vr, ui.id().with("version"), Sense::click());
        p.text(pos2(vr.center().x, vr.center().y - 6.), Align2::CENTER_CENTER, if copied { "" } else { "v" }, font(9.5, "semibold"), TEXT_FAINT);
        p.text(
            pos2(vr.center().x, vr.center().y + 7.),
            Align2::CENTER_CENTER,
            if copied { "Copied" } else { VERSION },
            font(10., "semibold"),
            if copied { SUCCESS } else if hit.hovered() { TEXT_MUTED } else { TEXT_FAINT },
        );
        hand(ui, &hit);
        if hit.clicked() {
            ui.ctx().copy_text(format!("ScarlettPort launcher v{VERSION}"));
            self.copied_at = Some(Instant::now());
        }
        hit.on_hover_text(format!("Launcher v{VERSION}. Click to copy."));
    }

    fn top_bar(&mut self, ui: &mut Ui, r: Rect) {
        let p = ui.painter().clone();
        let title = match self.page {
            Page::Play => "",
            Page::News => "Progress notes",
            Page::Setup => "Setup",
            Page::Logs => "Logs",
            Page::Links => "Links",
            Page::Settings => "Settings",
        };
        p.text(pos2(SIDEBAR + MARGIN, 30.), Align2::LEFT_CENTER, title, font(20., "bold"), TEXT);
        // Live frame rate pill, where the reference shows the signed-in player.
        if let Some(fps) = self.fps {
            let label = format!("{fps:.1} FPS");
            let w = text_width(ui, &label, font(13., "semibold")) + 36.;
            let pill = Rect::from_min_size(pos2(r.max.x - 88. - w, 12.), vec2(w, 32.));
            p.rect_filled(pill, RADIUS_MD, BG_100);
            p.rect_stroke(pill, RADIUS_MD, Stroke::new(1.0_f32, LINE), egui::StrokeKind::Inside);
            dot(&p, pos2(pill.min.x + 15., pill.center().y), SUCCESS);
            p.text(pos2(pill.min.x + 28., pill.center().y), Align2::LEFT_CENTER, label, font(13., "semibold"), TEXT);
        }
    }

    // ------------------------------------------------------------------ play

    fn play_page(&mut self, ui: &mut Ui, area: Rect, time: f32) {
        let header = Rect::from_min_max(pos2(area.min.x, 56.), pos2(area.max.x, 276.));
        self.play_header(ui, header, time);

        let mut y = header.max.y + 16.;
        let mut row = |h: f32| {
            let rect = Rect::from_min_max(pos2(area.min.x, y), pos2(area.max.x, y + h));
            y += h + 6.;
            rect
        };
        let missing = game::missing(&self.game_checks);
        let (status, color) = if !self.game_ok {
            ("Not set".to_string(), Some(WARNING))
        } else if missing.is_empty() {
            ("Ready".to_string(), Some(SUCCESS))
        } else {
            (format!("Missing {}", missing.iter().map(|m| m.trim_end_matches(".exe")).collect::<Vec<_>>().join(", ")), Some(WARNING))
        };
        let path = if self.game_ok { game::summary(&self.game_checks) } else { format!("Pick the folder that contains {GAME_EXE}") };
        if section_row(ui, row(52.), "row-game", Icon::Folder, "Game files", &path, Status { text: &status, color }, true) {
            self.settings_tab = 0;
            self.go(Page::Settings);
        }
        let (status, color) = if self.runtime_dir.is_some() { ("Ready", Some(SUCCESS)) } else { ("Unpacking…", None) };
        let note = self.runtime_note.clone();
        if section_row(ui, row(52.), "row-runtime", Icon::Server, "XCL runtime", &note, Status { text: status, color }, true) {
            self.settings_tab = 2;
            self.go(Page::Settings);
        }
        let stage = self.stage();
        let (meta, status, color) = match stage {
            Stage::BuildingShaders => match self.shader {
                Some((d, t, s)) => {
                    let eta = self.eta_secs.map(|e| format!("  ·  about {} left", flow::duration(e))).unwrap_or_default();
                    (format!("{d} of {t} shaders  ·  {s} s elapsed{eta}"), format!("{}%", d * 100 / t.max(1)), None)
                }
                None => ("Starting the shader build…".to_string(), "Building".to_string(), None),
            },
            Stage::Ready => ("Built for this runtime".to_string(), "Ready".to_string(), Some(SUCCESS)),
            Stage::ShadersNotBuilt if self.shaders_stale() => ("The runtime changed since the shaders were built. Rebuild them (resumes where it stopped).".to_string(), "Stale".to_string(), Some(WARNING)),
            Stage::ShadersNotBuilt => ("Build the shader cache once (about 45 min, resumable)".to_string(), "Not built".to_string(), Some(WARNING)),
            Stage::GameNotSet => ("Needs the game folder first".to_string(), "Waiting".to_string(), None),
        };
        if section_row(ui, row(52.), "row-shaders", Icon::Download, "Shaders", &meta, Status { text: &status, color }, true) {
            self.go(Page::Setup);
        }
        let world = self.world();
        let (meta, status, color) = match world {
            World::Waiting => ("Build the shaders first".to_string(), "Waiting", None),
            World::NeedPlay => ("Play once to record world pipelines".to_string(), "No journal", None),
            World::Available => ("Build the pipelines your last run recorded (2 threads, low priority)".to_string(), "Build", Some(WARNING)),
            World::Building => (
                match self.world_progress {
                    Some((d, l)) => format!("{d} done, {l} left"),
                    None => "Starting…".to_string(),
                },
                "Building",
                None,
            ),
            World::Done => ("Up to date with your last run".to_string(), "Ready", Some(SUCCESS)),
        };
        if section_row(ui, row(52.), "row-world", Icon::Server, "World pipelines", &meta, Status { text: status, color }, world == World::Available) {
            self.start(RunKind::Pipelines);
        }
        let prog = row(52.);
        section_row(ui, prog, "row-progress", Icon::Info, "Progress to playable", "Menus, loading and the open world run. Geometry passes are next.", Status { text: &format!("{PLAYABLE_PERCENT:.0}%"), color: None }, false);
        let track = Rect::from_min_max(pos2(prog.min.x + 14., prog.max.y - 5.), pos2(prog.max.x - 14., prog.max.y - 3.));
        progress(&ui.painter().clone(), track, Some(PLAYABLE_PERCENT / 100.), time);
        let (status, color) = match self.fps {
            Some(f) => (format!("{f:.1} fps"), Some(SUCCESS)),
            None => ("Not running".to_string(), None),
        };
        if section_row(ui, row(52.), "row-fps", Icon::History, "Frame rate", "Read live from the game log", Status { text: &status, color }, true) {
            self.go(Page::Logs);
        }
    }

    fn play_header(&mut self, ui: &mut Ui, rect: Rect, time: f32) {
        let p = ui.painter().clone();
        let hover = ui.ctx().animate_bool_with_time(ui.id().with("hdr-hover"), ui.rect_contains_pointer(rect), 0.5);
        // Cover-crop the art into the wide card; zoom 3% on hover.
        let [iw, ih] = self.art.size();
        let img_aspect = iw as f32 / ih as f32;
        let aspect = rect.width() / rect.height();
        let zoom = 1. + 0.03 * ease(hover);
        let (uw, uh) = if aspect > img_aspect { (1. / zoom, img_aspect / aspect / zoom) } else { (aspect / img_aspect / zoom, 1. / zoom) };
        let (cx, cy) = (0.5, 0.55);
        let uv = Rect::from_min_max(pos2(cx - uw / 2., (cy - uh / 2.).max(0.)), pos2(cx + uw / 2., (cy - uh / 2.).max(0.) + uh));
        egui::Image::new((self.art.id(), rect.size())).uv(uv).corner_radius(RADIUS_LG).paint_at(ui, rect);
        vertical_fill(&p, rect, RADIUS_LG as f32, |v| Color32::from_black_alpha((((v - 0.15) / 0.85).clamp(0., 1.).powf(1.3) * 215.) as u8));
        p.rect_stroke(rect, RADIUS_LG, Stroke::new(1.0_f32, LINE), egui::StrokeKind::Inside);

        let label = if self.game_ok { "CHANGE GAME FOLDER" } else { "SELECT GAME FOLDER" };
        if button(ui, Rect::from_min_size(pos2(rect.max.x - 266., rect.min.y + 16.), vec2(250., CONTROL)), "hdr-folder", label, Kind::Secondary, !self.folder_busy, Some(Icon::Folder)) {
            self.pick_folder();
        }

        let x = rect.min.x + 26.;
        let bottom = rect.max.y - 24.;
        let white = |a: f32| fade(Color32::WHITE, a);
        spaced(&p, pos2(x, bottom - 70.), Align2::LEFT_BOTTOM, "XCL RUNTIME  ·  WINDOWS  ·  EARLY PREVIEW", font(11., "bold"), white(0.75), 1.2);
        p.text(pos2(x, bottom - 30.), Align2::LEFT_BOTTOM, "ScarlettPort", font(30., "black"), Color32::WHITE);

        let running = self.running;
        let stage = self.stage();
        let (fraction, mut meta_color): (Option<Option<f32>>, Color32) = match running {
            Some(RunKind::Precompile) => (Some(self.shader.map(|(d, t, _)| d as f32 / t.max(1) as f32)), white(0.75)),
            Some(RunKind::Pipelines) => (Some(self.world_progress.map(|(d, l)| d as f32 / (d + l).max(1) as f32)), white(0.75)),
            Some(RunKind::Play) => (Some(None), white(0.75)),
            None => (None, white(0.75)),
        };
        let meta = if running == Some(RunKind::Precompile) {
            match self.shader {
                Some((d, t, secs)) => {
                    let eta = self.eta_secs.map(|e| format!("  ·  about {} left", flow::duration(e))).unwrap_or_default();
                    format!("Building shaders {d}/{t}  ·  {} elapsed{eta}", flow::duration(secs))
                }
                None => self.status.clone(),
            }
        } else if running.is_some() && !self.status.is_empty() {
            self.status.clone()
        } else if !self.game_ok {
            meta_color = WARNING;
            format!("Step 1: pick your game folder  ·  needs {GAME_EXE}")
        } else if stage == Stage::GameNotSet {
            meta_color = WARNING;
            format!("Game folder incomplete  ·  missing {}", game::missing(&self.game_checks).join(", "))
        } else if stage == Stage::ShadersNotBuilt {
            meta_color = WARNING;
            if self.shaders_stale() { "Runtime updated  ·  rebuild the shaders before playing".to_string() } else { "Step 2: build the shaders once (about 45 min)  ·  then press Play".to_string() }
        } else if !self.status.is_empty() {
            self.status.clone()
        } else {
            format!("{}  ·  XCL runtime {}", self.config.game_path, if self.runtime_dir.is_some() { "ready" } else { "unpacking" })
        };
        let shown = fit(ui, &meta, font(13., "regular"), rect.width() - 250.);
        p.text(pos2(x, bottom), Align2::LEFT_BOTTOM, &shown, font(13., "regular"), meta_color);

        let button_rect = Rect::from_min_max(pos2(rect.max.x - 26. - 190., bottom - 46.), pos2(rect.max.x - 26., bottom));
        if running.is_some() {
            if button_font(ui, button_rect, "hdr-stop", "Stop", Kind::Secondary, true, Some(Icon::Stop), font(15., "bold"), 0.) {
                self.stop_run();
            }
        } else {
            let (text, kind_font, tracking) = match stage {
                Stage::GameNotSet => ("SELECT GAME", font(13., "black"), 1.5),
                Stage::ShadersNotBuilt | Stage::BuildingShaders => ("BUILD SHADERS", font(13., "black"), 1.5),
                Stage::Ready => ("PLAY", font(15., "black"), 2.),
            };
            let enabled = match stage {
                Stage::GameNotSet => !self.folder_busy,
                _ => self.runtime_dir.is_some(),
            };
            let icon = if stage == Stage::Ready { Icon::Play } else if stage == Stage::GameNotSet { Icon::Folder } else { Icon::Download };
            if button_font(ui, button_rect, "hdr-play", text, Kind::Primary, enabled, Some(icon), kind_font, tracking) {
                match stage {
                    Stage::GameNotSet => self.pick_folder(),
                    Stage::ShadersNotBuilt => self.confirm_precompile = true,
                    Stage::Ready => self.start(RunKind::Play),
                    Stage::BuildingShaders => {}
                }
            }
            if stage != Stage::Ready && self.game_ok && self.runtime_dir.is_some() && link(ui, pos2(button_rect.center().x - 28., button_rect.max.y + 12.), Align2::LEFT_CENTER, "hdr-anyway", "play anyway") {
                self.start(RunKind::Play);
            }
        }
        if let Some(f) = fraction {
            let line = Rect::from_min_max(pos2(rect.min.x + 1., rect.max.y - 3.), pos2(rect.max.x - 1., rect.max.y - 1.));
            progress(&p.with_clip_rect(line), line, f, time);
        }
    }

    // ------------------------------------------------------------------ progress notes

    fn news_page(&mut self, ui: &mut Ui, area: Rect) {
        let p = ui.painter().clone();
        caption(&p, pos2(area.min.x, 72.), Align2::LEFT_TOP, &format!("Latest  ·  {} notes", NEWS.len()));
        let top = 100.;
        let gap = 16.;
        let hero_w = ((area.width() - gap) * 0.56).floor();
        let hero = Rect::from_min_max(pos2(area.min.x, top), pos2(area.min.x + hero_w, area.max.y));
        let hit = ui.interact(hero, ui.id().with("news-hero"), Sense::hover());
        let hov = ui.ctx().animate_bool_with_time(hit.id, hit.hovered(), QUICK);
        p.rect_filled(hero, RADIUS_MD, BG_100);
        let picture = Rect::from_min_size(hero.min, vec2(hero.width(), hero.width() * 9. / 16.));
        cover(ui, &self.art, picture, RADIUS_MD, Color32::WHITE);
        let mut scrim = egui::Mesh::default();
        let clear = Color32::from_black_alpha(0);
        let dark = Color32::from_black_alpha(215);
        let mid = hero.min.y + hero.height() * 0.55;
        scrim.colored_vertex(pos2(hero.min.x, mid), clear);
        scrim.colored_vertex(pos2(hero.max.x, mid), clear);
        scrim.colored_vertex(pos2(hero.max.x, hero.max.y), dark);
        scrim.colored_vertex(pos2(hero.min.x, hero.max.y), dark);
        scrim.add_triangle(0, 1, 2);
        scrim.add_triangle(0, 2, 3);
        p.add(scrim);
        let f = &NEWS[2];
        let tw = hero.width() - 48.;
        let sum = p.layout(f.3.into(), font(13., "regular"), TEXT_MUTED, tw);
        let title = p.layout(f.2.into(), font(24., "bold"), TEXT, tw);
        let mut y = hero.max.y - 22. - sum.size().y;
        p.galley(pos2(hero.min.x + 24., y), sum, TEXT_MUTED);
        y -= title.size().y + 6.;
        p.galley(pos2(hero.min.x + 24., y), title, TEXT);
        caption(&p, pos2(hero.min.x + 24., y - 8.), Align2::LEFT_BOTTOM, &format!("{}  ·  {}", f.0, f.1));
        p.rect_stroke(hero, RADIUS_MD, Stroke::new(1.0_f32, mix(LINE, LINE_HOVER, hov)), egui::StrokeKind::Inside);

        let x = hero.max.x + gap;
        let w = area.max.x - x;
        let row_h = 104.;
        let list = Rect::from_min_max(pos2(x, top), area.max);
        ui.scope_builder(egui::UiBuilder::new().max_rect(list), |ui| {
            egui::ScrollArea::vertical().id_salt("news-archive").auto_shrink([false, false]).show(ui, |ui| {
                let start = ui.cursor().min;
                let w = w - 10.;
                let others: Vec<usize> = (0..NEWS.len()).filter(|i| *i != 2).collect();
                ui.allocate_space(vec2(w, (others.len() as f32 * (row_h + 12.) - 12.).max(0.)));
                let p = ui.painter().clone();
                for (n, idx) in others.iter().enumerate() {
                    let post = &NEWS[*idx];
                    let y = start.y + n as f32 * (row_h + 12.);
                    let row = Rect::from_min_size(pos2(start.x, y), vec2(w, row_h));
                    let hit = ui.interact(row, ui.id().with(("news-row", n)), Sense::hover());
                    let h = ui.ctx().animate_bool_with_time(hit.id, hit.hovered(), QUICK);
                    p.rect_filled(row, RADIUS_MD, mix(BG_100, BG_200, h));
                    let thumb = Rect::from_min_size(pos2(row.min.x + 12., row.min.y + 17.), vec2(124., 69.75));
                    p.rect_filled(thumb, 6, BG_300);
                    cover(ui, &self.art, thumb, 6, mix(Color32::from_gray(200), Color32::WHITE, h));
                    let tx = thumb.max.x + 14.;
                    let tw = row.max.x - tx - 12.;
                    caption(&p, pos2(tx, row.min.y + 14.), Align2::LEFT_TOP, &format!("{}  ·  {}", post.0, post.1));
                    p.text(pos2(tx, row.min.y + 34.), Align2::LEFT_TOP, fit(ui, post.2, font(15., "bold"), tw), font(15., "bold"), TEXT);
                    let sum = p.layout(post.3.into(), font(12., "regular"), TEXT_MUTED, tw);
                    let clipped = p.with_clip_rect(Rect::from_min_size(pos2(tx, row.min.y + 56.), vec2(tw, 36.)));
                    clipped.galley(pos2(tx, row.min.y + 56.), sum, TEXT_MUTED);
                    p.rect_stroke(row, RADIUS_MD, Stroke::new(1.0_f32, mix(LINE, LINE_HOVER, h)), egui::StrokeKind::Inside);
                }
            });
        });
    }

    // ------------------------------------------------------------------ setup

    fn setup_page(&mut self, ui: &mut Ui, area: Rect, time: f32) {
        let p = ui.painter().clone();
        let top = 72.;
        caption(&p, pos2(area.min.x, top), Align2::LEFT_TOP, "Shader precompile");
        let text = "Builds the compute-shader and pipeline cache for your game once, so play sessions start smoothly. It takes about 45 minutes and resumes where it stopped if you close it.";
        let g = p.layout(text.into(), font(13., "regular"), TEXT_MUTED, area.width().min(720.));
        let gh = g.size().y;
        p.galley(pos2(area.min.x, top + 22.), g, TEXT_MUTED);

        let fy = top + 22. + gh + 22.;
        let (done, total, secs) = self.shader.unwrap_or((0, 0, 0));
        let facts = [
            ("Shaders done", if total > 0 { format!("{done} / {total}") } else { "-".into() }),
            ("Elapsed", if total > 0 { format!("{} min {} s", secs / 60, secs % 60) } else { "-".into() }),
            ("Pipelines made", self.pipelines.map(|n| n.to_string()).unwrap_or_else(|| "-".into())),
            ("Progress", if total > 0 { format!("{}%", done * 100 / total.max(1)) } else { "-".into() }),
        ];
        let cw = (area.width() - 30.) / 4.;
        for (i, (k, v)) in facts.iter().enumerate() {
            let r = Rect::from_min_size(pos2(area.min.x + i as f32 * (cw + 10.), fy), vec2(cw, 58.));
            p.rect_filled(r, RADIUS_MD, BG_100);
            p.rect_stroke(r, RADIUS_MD, Stroke::new(1.0_f32, LINE), egui::StrokeKind::Inside);
            caption(&p, pos2(r.min.x + 14., r.min.y + 12.), Align2::LEFT_TOP, k);
            p.text(pos2(r.min.x + 14., r.min.y + 30.), Align2::LEFT_TOP, v, font(16., "bold"), TEXT);
        }
        let py = fy + 80.;
        let precompiling = self.running == Some(RunKind::Precompile);
        let frac = if precompiling { Some(self.shader.map(|(d, t, _)| d as f32 / t.max(1) as f32)).unwrap_or(None) } else { Some(if total > 0 { done as f32 / total as f32 } else { 0. }) };
        let frac = if precompiling && self.shader.is_none() { None } else { frac };
        progress(&p, Rect::from_min_size(pos2(area.min.x, py), vec2(area.width(), 6.)), frac, time);
        let line = if precompiling { self.status.clone() } else if !self.game_ok { format!("Pick a game folder containing {GAME_EXE} first.") } else { self.status.clone() };
        p.text(pos2(area.min.x, py + 18.), Align2::LEFT_TOP, fit(ui, &line, font(13., "semibold"), area.width()), font(13., "semibold"), TEXT);

        let by = py + 52.;
        if precompiling {
            if button(ui, Rect::from_min_size(pos2(area.min.x, by), vec2(120., CONTROL)), "setup-stop", "Stop", Kind::Secondary, true, Some(Icon::Stop)) {
                self.stop_run();
            }
        } else if button(ui, Rect::from_min_size(pos2(area.min.x, by), vec2(220., CONTROL)), "setup-go", "Precompile shaders", Kind::Primary, self.running.is_none() && self.runtime_dir.is_some(), Some(Icon::Download)) {
            if self.game_ok {
                self.confirm_precompile = true;
            } else {
                self.settings_tab = 0;
                self.go(Page::Settings);
            }
        }
        if button(ui, Rect::from_min_size(pos2(area.min.x + 232., by), vec2(120., CONTROL)), "setup-logs", "View log", Kind::Ghost, true, None) {
            self.go(Page::Logs);
        }
    }

    fn precompile_modal(&mut self, ctx: &egui::Context) {
        let mut close = false;
        let closed = modal(ctx, "precompile", vec2(520., 250.), "Precompile shaders", "Runs once and resumes if interrupted.", true, |ui, body, foot| {
            let p = ui.painter().clone();
            let text = "This uses your CPU and GPU heavily for about 45 minutes. You can leave the launcher open and keep using the PC; Stop is always available.";
            let g = p.layout(text.into(), font(13., "regular"), TEXT, body.width());
            p.galley(body.min, g, TEXT);
            if button(ui, Rect::from_min_size(foot.min, vec2(100., CONTROL)), "pc-cancel", "Cancel", Kind::Ghost, true, None) {
                close = true;
            }
            if button(ui, Rect::from_min_size(pos2(foot.max.x - 130., foot.min.y), vec2(130., CONTROL)), "pc-go", "Start", Kind::Primary, true, Some(Icon::Play)) {
                close = true;
                self.start(RunKind::Precompile);
            }
        });
        if closed || close {
            self.confirm_precompile = false;
        }
    }

    // ------------------------------------------------------------------ logs

    fn logs_page(&mut self, ui: &mut Ui, area: Rect) {
        let p = ui.painter().clone();
        caption(&p, pos2(area.min.x, 72.), Align2::LEFT_TOP, &format!("Live log  ·  {} lines", self.logs.len()));
        if link(ui, pos2(area.max.x, 72.), Align2::RIGHT_TOP, "log-clear", "Clear") {
            self.logs.clear();
        }
        if link(ui, pos2(area.max.x - 56., 72.), Align2::RIGHT_TOP, "log-copy", "Copy all") {
            let all: Vec<&str> = self.logs.iter().map(|(t, _)| t.as_str()).collect();
            ui.ctx().copy_text(all.join("\n"));
        }
        let panel = Rect::from_min_max(pos2(area.min.x, 100.), area.max);
        p.rect_filled(panel, RADIUS_MD, BG_100);
        p.rect_stroke(panel, RADIUS_MD, Stroke::new(1.0_f32, LINE), egui::StrokeKind::Inside);
        let inner = panel.shrink2(vec2(14., 10.));
        let logs = &self.logs;
        ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
            egui::ScrollArea::vertical().id_salt("log-scroll").stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
                ui.set_width(inner.width() - 12.);
                if logs.is_empty() {
                    ui.label(egui::RichText::new("Nothing yet. Press Play or Precompile shaders.").font(font(13., "regular")).color(TEXT_FAINT));
                }
                for (t, k) in logs {
                    let c = match k {
                        LogKind::Info => ACCENT,
                        LogKind::Out => TEXT_MUTED,
                        LogKind::Error => mix(DANGER, TEXT, 0.2),
                    };
                    ui.add(egui::Label::new(egui::RichText::new(t).font(egui::FontId::monospace(12.)).color(c)).wrap());
                }
            });
        });
    }

    // ------------------------------------------------------------------ links

    fn links_page(&mut self, ui: &mut Ui, area: Rect) {
        let p = ui.painter().clone();
        caption(&p, pos2(area.min.x, 72.), Align2::LEFT_TOP, "Project");
        let mut y = 100.;
        let rows = [
            ("GitHub repository", "ZigZagBoi1/ScarlettPort: source, issues and builds", REPO_URL, Icon::News, "Open"),
            ("Donate on PayPal", "paypal.me/SalwanStudios  ·  send as Friends & Family to avoid fees", PAYPAL_URL, Icon::User, "Open"),
            ("Support on Ko-fi", "ko-fi.com/ziggymar", KOFI_URL, Icon::Plus, "Open"),
        ];
        for (title, meta, url, icon, status) in rows {
            let r = Rect::from_min_max(pos2(area.min.x, y), pos2(area.max.x, y + 58.));
            if section_row(ui, r, ("link", url), icon, title, meta, Status { text: status, color: None }, true) {
                ui.ctx().open_url(egui::OpenUrl::new_tab(url));
            }
            y += 68.;
        }
    }

    // ------------------------------------------------------------------ settings

    fn settings_page(&mut self, ui: &mut Ui, area: Rect) {
        let p = ui.painter().clone();
        let top = 60.;
        for (i, name) in TABS.iter().enumerate() {
            let rect = Rect::from_min_size(pos2(area.min.x, top + i as f32 * 42.), vec2(190., 36.));
            let hit = ui.interact(rect, ui.id().with(("tab", i)), Sense::click());
            let h = ui.ctx().animate_bool_with_time(hit.id, hit.hovered(), QUICK);
            let sel = self.settings_tab == i;
            p.rect_filled(rect, RADIUS_MD, if sel { BG_300 } else { fade(BG_200, h) });
            if sel {
                p.rect_filled(Rect::from_min_size(pos2(rect.min.x, rect.center().y - 9.), vec2(3., 18.)), 2, ACCENT);
            }
            p.text(pos2(rect.min.x + 16., rect.center().y), Align2::LEFT_CENTER, *name, font(14., if sel { "semibold" } else { "regular" }), if sel || h > 0.5 { TEXT } else { TEXT_MUTED });
            hand(ui, &hit);
            if hit.clicked() {
                self.settings_tab = i;
            }
        }
        let c = Rect::from_min_max(pos2(area.min.x + 214., top), pos2(area.max.x, area.max.y));
        p.line_segment([pos2(c.min.x - 12., top), pos2(c.min.x - 12., area.max.y)], Stroke::new(1.0_f32, LINE));
        match self.settings_tab {
            0 => self.settings_game(ui, c),
            1 => self.settings_options(ui, c),
            _ => self.settings_about(ui, c),
        }
    }

    fn settings_game(&mut self, ui: &mut Ui, c: Rect) {
        let p = ui.painter().clone();
        caption(&p, pos2(c.min.x, c.min.y), Align2::LEFT_TOP, "Game folder");
        p.text(pos2(c.min.x, c.min.y + 20.), Align2::LEFT_TOP, format!("The folder that contains {GAME_EXE}. It is used read-only."), font(12.5, "regular"), TEXT_MUTED);
        let field = Rect::from_min_size(pos2(c.min.x, c.min.y + 46.), vec2(c.width(), 44.));
        text_field(ui, field, "game-edit", &mut self.game_edit, "Folder", Some(Icon::Folder), !self.folder_busy);
        let by = field.max.y + 10.;
        if button(ui, Rect::from_min_size(pos2(c.min.x, by), vec2(112., CONTROL)), "browse", "Browse…", Kind::Secondary, !self.folder_busy, None) {
            self.pick_folder();
        }
        if button(ui, Rect::from_min_size(pos2(c.min.x + 120., by), vec2(104., CONTROL)), "use-folder", "Verify", Kind::Primary, !self.folder_busy, Some(Icon::Check)) {
            self.use_folder(std::path::PathBuf::from(self.game_edit.trim()));
        }
        let (note, ok) = self.folder_note.clone();
        let note_color = if self.folder_busy { TEXT_MUTED } else if ok { SUCCESS } else { mix(DANGER, TEXT, 0.2) };
        let galley = p.layout(note, font(12.5, "regular"), note_color, c.width());
        p.galley(pos2(c.min.x, by + CONTROL + 12.), galley, note_color);
    }

    fn settings_options(&mut self, ui: &mut Ui, c: Rect) {
        let p = ui.painter().clone();
        caption(&p, pos2(c.min.x, c.min.y), Align2::LEFT_TOP, "Play options");
        let row = Rect::from_min_size(pos2(c.min.x, c.min.y + 28.), vec2(c.width().min(640.), 58.));
        p.rect_filled(row, RADIUS_MD, BG_100);
        p.rect_stroke(row, RADIUS_MD, Stroke::new(1.0_f32, LINE), egui::StrokeKind::Inside);
        p.text(pos2(row.min.x + 18., row.center().y - 9.), Align2::LEFT_CENTER, "Auto-advance menus", font(15., "semibold"), TEXT);
        p.text(pos2(row.min.x + 18., row.center().y + 10.), Align2::LEFT_CENTER, "Presses A every 6 s from 22 s to 70 s to pass the title and account screens.", font(12., "regular"), TEXT_MUTED);
        let sw = Rect::from_center_size(pos2(row.max.x - 44., row.center().y), vec2(40., 24.));
        let mut on = self.config.auto_menu;
        if switch(ui, sw, "auto-menu", &mut on) {
            self.config.auto_menu = on;
            self.save();
        }

        let top = row.max.y + 28.;
        caption(&p, pos2(c.min.x, top), Align2::LEFT_TOP, "Cache location");
        p.text(pos2(c.min.x, top + 20.), Align2::LEFT_TOP, "Compiled shaders go to <folder>\\dxil. It needs about 40 GB, so pick a drive with room.", font(12.5, "regular"), TEXT_MUTED);
        let field = Rect::from_min_size(pos2(c.min.x, top + 46.), vec2(c.width().min(640.), 44.));
        let busy = self.running.is_some();
        if text_field(ui, field, "cache-edit", &mut self.config.cache_dir, "Folder", Some(Icon::Folder), !busy).changed() {
            self.cache_free = None;
            self.save();
        }
        let by = field.max.y + 10.;
        if button(ui, Rect::from_min_size(pos2(c.min.x, by), vec2(150., CONTROL)), "cache-default", "Use default", Kind::Secondary, !busy, Some(Icon::Refresh)) {
            self.config.cache_dir = game::default_cache();
            self.cache_free = None;
            self.save();
        }
        let dir = self.config.cache_dir.trim().to_string();
        let drive = if dir.as_bytes().get(1) == Some(&b':') { format!("{}:", &dir[..1].to_uppercase()) } else { "That drive".to_string() };
        let (text, color) = match self.cache_free.and_then(|(_, f)| f) {
            Some(f) if f < game::LOW_SPACE => (format!("{drive} has only {} free. The shader cache needs about 40 GB.", game::human(f)), WARNING),
            Some(f) => (format!("{drive} has {} free.", game::human(f)), SUCCESS),
            None => ("Free space unknown: the folder or its drive was not found.".to_string(), WARNING),
        };
        p.text(pos2(c.min.x, by + CONTROL + 12.), Align2::LEFT_TOP, text, font(12.5, "regular"), color);
    }

    fn settings_about(&mut self, ui: &mut Ui, c: Rect) {
        let p = ui.painter().clone();
        p.image(self.mark.id(), Rect::from_min_size(c.min + vec2(-4., -4.), vec2(52., 52.)), Rect::from_min_max(pos2(0., 0.), pos2(1., 1.)), Color32::WHITE);
        p.text(pos2(c.min.x + 60., c.min.y + 4.), Align2::LEFT_TOP, "ScarlettPort", font(20., "bold"), TEXT);
        p.text(pos2(c.min.x + 60., c.min.y + 30.), Align2::LEFT_TOP, format!("Launcher v{VERSION}"), font(13., "regular"), TEXT_MUTED);
        let y = c.min.y + 72.;
        if button(ui, Rect::from_min_size(pos2(c.min.x, y), vec2(190., CONTROL)), "reunpack", "Re-extract runtime", Kind::Secondary, self.running.is_none(), Some(Icon::Refresh)) {
            let _ = std::fs::remove_file(crate::runtime::runtime_dir().join(".hash"));
            self.runtime_dir = None;
            self.runtime_note = "Unpacking runtime…".into();
            self.unpack_runtime();
        }
        if button(ui, Rect::from_min_size(pos2(c.min.x + 200., y), vec2(140., CONTROL)), "copy-version", "Copy version", Kind::Secondary, true, None) {
            ui.ctx().copy_text(format!("ScarlettPort launcher v{VERSION}"));
            self.copied_at = Some(Instant::now());
        }
        p.text(pos2(c.min.x, y + CONTROL + 14.), Align2::LEFT_TOP, format!("Runtime folder: {}", self.runtime_note), font(12.5, "regular"), TEXT_MUTED);
        let y = y + 100.;
        p.line_segment([pos2(c.min.x, y), pos2(c.max.x, y)], Stroke::new(1.0_f32, LINE));
        let text = "The launcher makes no network connections. The only outside access is the links you click, which open in your browser. Settings live in %LOCALAPPDATA%\\ScarlettPort\\launcher.json.";
        let g = p.layout(text.into(), font(13., "regular"), TEXT_MUTED, c.width().min(560.));
        p.galley(pos2(c.min.x, y + 20.), g, TEXT_MUTED);
        if link(ui, pos2(c.min.x, y + 100.), Align2::LEFT_CENTER, "site", "github.com/ZigZagBoi1/ScarlettPort") {
            ui.ctx().open_url(egui::OpenUrl::new_tab(REPO_URL));
        }
    }
}
