//! ScarlettPort launcher: native Rust (eframe/egui), fixed-size frameless window.
//! No network access except opening the project links in the browser.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod art;
mod display;
mod flow;
mod game;
mod runtime;
mod theme;
mod view;

use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    time::Instant,
};

pub const GAME_EXE: &str = "game_scarlett_master.exe";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REPO_URL: &str = "https://github.com/ZigZagBoi1/ScarlettPort";
pub const PAYPAL_URL: &str = "https://www.paypal.me/SalwanStudios";
pub const KOFI_URL: &str = "https://ko-fi.com/ziggymar";
pub const PLAYABLE_PERCENT: f32 = 25.;
const MAX_LOG: usize = 3000;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Play,
    News,
    Setup,
    Logs,
    Links,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RunKind {
    Play,
    Precompile,
    Pipelines,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LogKind {
    Info,
    Out,
    Error,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub game_path: String,
    #[serde(default)]
    pub auto_menu: bool,
    /// Where the DXIL cache lives (`<cache>\dxil`); empty until first run picks the default.
    #[serde(default)]
    pub cache_dir: String,
    /// Runtime hash the shader cache was built for; a different runtime makes it stale.
    #[serde(default)]
    pub shaders_hash: String,
    /// Size of the pipeline journal when the world pipelines were last built.
    #[serde(default)]
    pub world_len: u64,
    /// Window mode, resolution, frame rate cap, vsync and monitor (passed to the runtime).
    #[serde(default)]
    pub display: display::DisplayCfg,
}

pub enum Message {
    Log(String, LogKind),
    Fps(f32),
    Shader { done: u64, total: u64, secs: u64 },
    Pipelines(u32),
    World { done: u64, left: u64 },
    Ended(RunKind, Option<i32>),
    RuntimeReady(Result<PathBuf, String>),
    Picked(Option<PathBuf>),
}

pub struct Launcher {
    pub page: Page,
    pub page_at: Instant,
    pub started: Instant,
    pub config: Config,
    pub runtime_dir: Option<PathBuf>,
    pub runtime_note: String,
    pub game_ok: bool,
    pub game_checks: Vec<game::FileCheck>,
    pub runtime_hash: Option<String>,
    pub eta: flow::Eta,
    pub eta_secs: Option<u64>,
    pub world_progress: Option<(u64, u64)>,
    pub cache_free: Option<(Instant, Option<u64>)>,
    pub journal: Option<(Instant, Option<u64>)>,
    pub game_edit: String,
    pub folder_note: (String, bool),
    pub folder_busy: bool,
    pub settings_tab: usize,
    pub monitors: Vec<display::Monitor>,
    /// The open dropdown on the Display tab (0 resolution, 1 monitor) and its scroll offset.
    pub dd: Option<u8>,
    pub dd_scroll: usize,
    pub running: Option<RunKind>,
    pub stop: Option<Arc<AtomicBool>>,
    pub logs: Vec<(String, LogKind)>,
    pub fps: Option<f32>,
    pub shader: Option<(u64, u64, u64)>,
    pub pipelines: Option<u32>,
    pub status: String,
    pub confirm_precompile: bool,
    pub copied_at: Option<Instant>,
    pub tx: Sender<Message>,
    pub rx: Receiver<Message>,
    pub mark: egui::TextureHandle,
    pub art: egui::TextureHandle,
    pub shot: Option<(String, u32)>,
}

pub fn config_path() -> PathBuf {
    runtime::data_dir().join("launcher.json")
}

pub fn valid_game(dir: &std::path::Path) -> bool {
    dir.join(GAME_EXE).is_file()
}

/// Walks up from a picked folder so choosing a subfolder still finds the game folder.
pub fn normalize(dir: &std::path::Path) -> Option<PathBuf> {
    let mut d = dir.to_path_buf();
    for _ in 0..4 {
        if valid_game(&d) {
            return Some(d);
        }
        if !d.pop() {
            break;
        }
    }
    None
}

impl Launcher {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install(&cc.egui_ctx);
        let (tx, rx) = channel();
        let mark = {
            let img = image::load_from_memory(include_bytes!("../assets/logo.png")).unwrap().to_rgba8();
            let size = [img.width() as usize, img.height() as usize];
            cc.egui_ctx.load_texture("mark", egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()), egui::TextureOptions::LINEAR)
        };
        let art = cc.egui_ctx.load_texture("art", art::header(), egui::TextureOptions::LINEAR);
        let mut config: Config = std::fs::read(config_path())
            .ok()
            .and_then(|v| serde_json::from_slice(&v).ok())
            .unwrap_or_default();
        let fresh_cache = config.cache_dir.trim().is_empty();
        if fresh_cache {
            config.cache_dir = game::default_cache();
        }
        let game_ok = !config.game_path.is_empty() && valid_game(std::path::Path::new(&config.game_path));
        let game_checks = if game_ok { game::verify(std::path::Path::new(&config.game_path)) } else { Vec::new() };
        let game_edit = config.game_path.clone();
        let l = Launcher {
            page: Page::Play,
            page_at: Instant::now(),
            started: Instant::now(),
            config,
            runtime_dir: None,
            runtime_note: "Unpacking runtime…".into(),
            game_ok,
            game_checks,
            runtime_hash: None,
            eta: flow::Eta::default(),
            eta_secs: None,
            world_progress: None,
            cache_free: None,
            journal: None,
            game_edit,
            folder_note: (String::new(), true),
            folder_busy: false,
            settings_tab: 0,
            monitors: display::monitors(),
            dd: None,
            dd_scroll: 0,
            running: None,
            stop: None,
            logs: Vec::new(),
            fps: None,
            shader: None,
            pipelines: None,
            status: String::new(),
            confirm_precompile: false,
            copied_at: None,
            tx,
            rx,
            mark,
            art,
            shot: std::env::args().find_map(|a| a.strip_prefix("--screenshot=").map(|p| (p.to_string(), 0))),
        };
        if fresh_cache {
            l.save();
        }
        l.unpack_runtime();
        let mut l = l;
        // `--page=settings` opens Settings on its Display tab (`--tab=game|display|options|about`)
        if std::env::args().any(|a| a == "--page=settings") {
            l.page = Page::Settings;
            l.settings_tab = 1;
        }
        if let Some(t) = std::env::args().find_map(|a| a.strip_prefix("--tab=").map(str::to_string)) {
            l.settings_tab = ["game", "display", "options", "about"].iter().position(|n| *n == t).unwrap_or(l.settings_tab);
        }
        l
    }

    pub fn unpack_runtime(&self) {
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Message::RuntimeReady(runtime::ensure()));
        });
    }

    pub fn save(&self) {
        if let Some(dir) = config_path().parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_vec_pretty(&self.config) {
            let _ = std::fs::write(config_path(), json);
        }
    }

    pub fn go(&mut self, page: Page) {
        if self.page != page {
            self.page = page;
            self.page_at = Instant::now();
        }
    }

    pub fn log(&mut self, text: impl Into<String>, kind: LogKind) {
        self.logs.push((text.into(), kind));
        if self.logs.len() > MAX_LOG {
            self.logs.drain(..self.logs.len() - MAX_LOG);
        }
    }

    pub fn use_folder(&mut self, dir: PathBuf) {
        match normalize(&dir) {
            Some(found) => {
                self.config.game_path = found.display().to_string();
                self.game_edit = self.config.game_path.clone();
                self.game_ok = true;
                self.game_checks = game::verify(&found);
                let missing = game::missing(&self.game_checks);
                self.folder_note = if missing.is_empty() {
                    (format!("All game files found: {}", game::summary(&self.game_checks)), true)
                } else {
                    (format!("Found {GAME_EXE}, but missing {}.", missing.join(", ")), false)
                };
                self.save();
            }
            None => {
                self.game_ok = false;
                self.game_checks = Vec::new();
                self.folder_note = (format!("That folder does not contain {GAME_EXE}."), false);
            }
        }
    }

    pub fn pick_folder(&mut self) {
        self.folder_busy = true;
        let tx = self.tx.clone();
        let start = PathBuf::from(self.game_edit.trim());
        std::thread::spawn(move || {
            let mut d = rfd::FileDialog::new().set_title("Select your game folder");
            if start.is_dir() {
                d = d.set_directory(&start);
            }
            let _ = tx.send(Message::Picked(d.pick_folder()));
        });
    }

    pub fn start(&mut self, kind: RunKind) {
        if self.running.is_some() {
            return;
        }
        let Some(rt) = self.runtime_dir.clone() else {
            self.status = "The runtime is still unpacking.".into();
            return;
        };
        if !self.game_ok {
            self.status = format!("Pick a game folder containing {GAME_EXE} first.");
            self.go(Page::Settings);
            self.settings_tab = 0;
            return;
        }
        let game = PathBuf::from(&self.config.game_path);
        let _ = std::fs::create_dir_all(game::dxil_dir(&self.config.cache_dir));
        let mut cmd = Command::new(runtime::xcl_exe());
        cmd.current_dir(&rt)
            .env("XCL_NO_WATCHDOG", "1")
            .env("XCL_DXC_DIR", &rt)
            .env("XCL_DXIL_CACHE", game::dxil_dir(&self.config.cache_dir))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
        match kind {
            RunKind::Play => {
                cmd.env("XCL_DXC_THREADS", "4").env("XCL_PSO_PRELOAD", "0").arg("run").arg(&game);
                for (k, v) in display::env(&self.config.display, self.monitors.len()) {
                    cmd.env(k, v);
                }
                if self.config.auto_menu {
                    let at: Vec<String> = (22..=70).step_by(6).map(|s| format!("{s}:A")).collect();
                    cmd.env("XCL_PRESS_AT", at.join(","));
                }
                self.fps = None;
                self.status = "Starting…".into();
                self.log(format!("> xcl.exe run \"{}\"", game.display()), LogKind::Info);
            }
            RunKind::Precompile => {
                let pack = game.join("shaders.rpf");
                if !pack.is_file() {
                    self.status = "No shaders.rpf in the game folder.".into();
                    return;
                }
                cmd.env("XCL_DXC_THREADS", "8")
                    .env("XCL_PIPELINE_THREADS", "3")
                    .arg("shaders")
                    .arg(&pack)
                    .args(["--precompile", "--pipelines", "--only", "cs"]);
                self.shader = None;
                self.pipelines = None;
                self.eta.reset();
                self.eta_secs = None;
                self.status = "Building shaders…".into();
                self.log("> xcl.exe shaders shaders.rpf --precompile --pipelines --only cs", LogKind::Info);
            }
            RunKind::Pipelines => {
                cmd.arg("pipelines").arg(&game).args(["--threads", "2"]);
                self.world_progress = None;
                self.status = "Building world pipelines…".into();
                self.log(format!("> xcl.exe pipelines \"{}\" --threads 2", game.display()), LogKind::Info);
            }
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // CREATE_NO_WINDOW, plus BELOW_NORMAL_PRIORITY_CLASS for the background builds
            let low = if kind == RunKind::Play { 0 } else { 0x0000_4000 };
            cmd.creation_flags(0x0800_0000 | low);
        }
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                self.status = format!("Could not start the runtime: {e}");
                self.log(self.status.clone(), LogKind::Error);
                return;
            }
        };
        let stop = Arc::new(AtomicBool::new(false));
        self.stop = Some(stop.clone());
        self.running = Some(kind);
        let tx = self.tx.clone();
        for (stream, is_err) in [
            (child.stdout.take().map(|s| Box::new(s) as Box<dyn Read + Send>), false),
            (child.stderr.take().map(|s| Box::new(s) as Box<dyn Read + Send>), true),
        ] {
            if let Some(stream) = stream {
                let tx = tx.clone();
                std::thread::spawn(move || read_lines(stream, tx, is_err));
            }
        }
        std::thread::spawn(move || {
            let code = loop {
                if stop.load(Ordering::Relaxed) {
                    let _ = child.kill();
                    break child.wait().ok().and_then(|s| s.code());
                }
                match child.try_wait() {
                    Ok(Some(s)) => break s.code(),
                    Ok(None) => std::thread::sleep(std::time::Duration::from_millis(150)),
                    Err(_) => break None,
                }
            };
            std::thread::sleep(std::time::Duration::from_millis(300)); // let the readers drain
            let _ = tx.send(Message::Ended(kind, code));
        });
    }

    pub fn game_complete(&self) -> bool {
        self.game_ok && !self.game_checks.is_empty() && game::missing(&self.game_checks).is_empty()
    }

    pub fn stage(&self) -> flow::Stage {
        flow::stage(&flow::Inputs {
            game_complete: self.game_complete(),
            precompiling: self.running == Some(RunKind::Precompile),
            built_hash: &self.config.shaders_hash,
            runtime_hash: self.runtime_hash.as_deref(),
        })
    }

    pub fn world(&self) -> flow::World {
        let len = self.journal.and_then(|(_, l)| l);
        flow::world(self.stage(), self.running == Some(RunKind::Pipelines), len, self.config.world_len)
    }

    /// True when shaders were built, but by a different runtime than the one in this launcher.
    pub fn shaders_stale(&self) -> bool {
        !self.config.shaders_hash.is_empty() && self.runtime_hash.as_deref().is_some_and(|h| h != self.config.shaders_hash)
    }

    pub fn stop_run(&mut self) {
        if let Some(s) = &self.stop {
            s.store(true, Ordering::Relaxed);
            self.status = "Stopping…".into();
        }
    }

    /// `--screenshot=FILE` renders the window to a PNG (the launcher's own framebuffer, so
    /// nothing in front of it can end up in the picture), then exits.
    fn screenshot(&mut self, ctx: &egui::Context) {
        let Some((path, frames)) = &mut self.shot else { return };
        *frames += 1;
        if *frames == 90 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let image = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = image {
            let ppp = ctx.pixels_per_point();
            let _ = ppp;
            let [w, h] = img.size;
            if let Some(buf) = image::RgbaImage::from_raw(w as u32, h as u32, img.as_raw().to_vec()) {
                let _ = buf.save(&*path);
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ctx.request_repaint();
    }

    fn pump(&mut self, ctx: &egui::Context) {
        if self.journal.is_none_or(|(t, _)| t.elapsed().as_secs() >= 2) {
            let len = if self.game_ok { game::journal_len(std::path::Path::new(&self.config.game_path)) } else { None };
            self.journal = Some((Instant::now(), len));
        }
        if self.cache_free.is_none_or(|(t, _)| t.elapsed().as_secs() >= 3) {
            self.cache_free = Some((Instant::now(), game::free_space(std::path::Path::new(self.config.cache_dir.trim()))));
        }
        while let Ok(m) = self.rx.try_recv() {
            match m {
                Message::Log(t, k) => {
                    parse_line(&t, &self.tx);
                    self.log(t, k);
                }
                Message::Fps(f) => {
                    self.fps = Some(f);
                    if self.running == Some(RunKind::Play) {
                        self.status = format!("Running  ·  {f:.1} fps");
                    }
                }
                Message::Shader { done, total, secs } => {
                    self.shader = Some((done, total, secs));
                    self.eta_secs = self.eta.remaining_secs(done, total, secs);
                }
                Message::World { done, left } => self.world_progress = Some((done, left)),
                Message::Pipelines(n) => self.pipelines = Some(n),
                Message::Ended(kind, code) => {
                    self.running = None;
                    self.stop = None;
                    self.fps = None;
                    if code == Some(0) {
                        match kind {
                            RunKind::Precompile => {
                                if let Some(h) = &self.runtime_hash {
                                    self.config.shaders_hash = h.clone();
                                    self.save();
                                }
                            }
                            RunKind::Pipelines => {
                                if let Some(l) = game::journal_len(std::path::Path::new(&self.config.game_path)) {
                                    self.config.world_len = l;
                                    self.save();
                                }
                            }
                            RunKind::Play => {}
                        }
                    }
                    let note = match code {
                        Some(0) => "Finished.".to_string(),
                        Some(c) => format!("Stopped (exit code {c})."),
                        None => "Stopped.".to_string(),
                    };
                    self.status = note.clone();
                    self.log(
                        format!(
                            "{} ended. {note}",
                            match kind {
                                RunKind::Play => "Game",
                                RunKind::Precompile => "Shader build",
                                RunKind::Pipelines => "World pipeline build",
                            }
                        ),
                        if matches!(code, Some(0)) { LogKind::Info } else { LogKind::Error },
                    );
                }
                Message::RuntimeReady(r) => match r {
                    Ok(dir) => {
                        self.runtime_hash = Some(runtime::bundle_hash().to_string());
                        self.runtime_dir = Some(dir.clone());
                        self.runtime_note = dir.display().to_string();
                    }
                    Err(e) => {
                        self.runtime_note = format!("Could not unpack the runtime: {e}");
                        self.log(self.runtime_note.clone(), LogKind::Error);
                    }
                },
                Message::Picked(p) => {
                    self.folder_busy = false;
                    if let Some(p) = p {
                        self.use_folder(p);
                    }
                }
            }
            ctx.request_repaint();
        }
    }
}

/// Splits a byte stream on \n and \r (progress lines are rewritten in place with \r).
fn read_lines(mut stream: Box<dyn Read + Send>, tx: Sender<Message>, is_err: bool) {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let flush = |buf: &mut Vec<u8>, tx: &Sender<Message>| {
        if !buf.is_empty() {
            let text = String::from_utf8_lossy(buf).trim_end().to_string();
            buf.clear();
            if !text.trim().is_empty() {
                let _ = tx.send(Message::Log(text, if is_err { LogKind::Out } else { LogKind::Out }));
            }
        }
    };
    while let Ok(n) = stream.read(&mut chunk) {
        if n == 0 {
            break;
        }
        for &b in &chunk[..n] {
            if b == b'\n' || b == b'\r' {
                flush(&mut buf, &tx);
            } else {
                buf.push(b);
            }
        }
    }
    flush(&mut buf, &tx);
}

/// Pulls progress out of runtime output: `  2600/26503 (718 s)`, `pipelines: 75 made`,
/// `... 27.2 fps over the last 300`.
fn parse_line(line: &str, tx: &Sender<Message>) {
    let t = line.trim();
    if let Some((done, total, secs)) = flow::parse_shader(t) {
        let _ = tx.send(Message::Shader { done, total, secs });
    }
    if let Some((done, left)) = flow::parse_pipelines_progress(t) {
        let _ = tx.send(Message::World { done, left });
    }
    if let Some(i) = t.find("pipelines:") {
        let n: String = t[i + 10..].trim_start().chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(n) = n.parse() {
            let _ = tx.send(Message::Pipelines(n));
        }
    }
    if t.contains("frames presented") || t.contains("fps") {
        if let Some(i) = t.find("fps") {
            let before = t[..i].trim_end();
            let num = before.rsplit(|c: char| !(c.is_ascii_digit() || c == '.')).next().unwrap_or("");
            if let Ok(f) = num.parse::<f32>() {
                let _ = tx.send(Message::Fps(f));
            }
        }
    }
}

impl eframe::App for Launcher {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0., 0., 0., 0.]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.pump(ctx);
        self.draw(ctx);
        self.screenshot(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop_run();
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
}

fn icon() -> egui::IconData {
    match image::load_from_memory(include_bytes!("../assets/logo.png")).map(|i| i.to_rgba8()) {
        Ok(i) => egui::IconData { width: i.width(), height: i.height(), rgba: i.into_raw() },
        Err(_) => egui::IconData::default(),
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180., 660.])
            .with_icon(icon())
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(true),
        ..Default::default()
    };
    eframe::run_native("ScarlettPort", options, Box::new(|cc| Ok(Box::new(Launcher::new(cc)))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Vec<String> {
        let (tx, rx) = channel();
        parse_line(s, &tx);
        rx.try_iter()
            .map(|m| match m {
                Message::Shader { done, total, secs } => format!("shader {done}/{total} {secs}"),
                Message::Pipelines(n) => format!("pipes {n}"),
                Message::Fps(f) => format!("fps {f}"),
                _ => String::new(),
            })
            .collect()
    }

    #[test]
    fn parses_progress() {
        assert_eq!(parse("  2600/26503 (718 s)"), vec!["shader 2600/26503 718"]);
        assert_eq!(parse("pipelines: 75 made"), vec!["pipes 75"]);
        assert_eq!(parse("D3D12.X: 600 frames presented, 27.2 fps over the last 300"), vec!["fps 27.2"]);
        assert!(parse("hello world").is_empty());
    }
}
