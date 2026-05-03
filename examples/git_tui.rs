#![allow(missing_docs)]
//! Git TUI — Real Git interface with status, log, diff, and branch management.
//!
//! A practical Git TUI that reads actual git repository data.
//!
//! Controls:
//!   1/2/3/4      — switch views (Status/Log/Diff/Branches)
//!   ↑/↓ or j/k   — navigate
//!   Enter        — stage/unstage (status) or checkout (branches)
//!   d            — view diff for selected file
//!   r            — refresh
//!   q            — quit

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{StatusBar, StatusSegment, TabBar, Toast, ToastKind};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use std::os::fd::AsFd;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq)]
enum GitView { Status, Log, Diff, Branches }

struct GitFile {
    status: char,
    path: String,
}

struct GitCommit {
    hash: String,
    author: String,
    date: String,
    message: String,
}

struct GitBranch {
    name: String,
    current: bool,
    remote: bool,
}

struct GitTui {
    should_quit: Arc<AtomicBool>,
    theme: Theme,
    area: Rect,

    view: GitView,
    tab_bar: TabBar,

    files: Vec<GitFile>,
    selected_file: usize,

    commits: Vec<GitCommit>,
    selected_commit: usize,

    branches: Vec<GitBranch>,
    selected_branch: usize,

    diff_content: String,

    status_bar: StatusBar,
    toasts: Vec<Toast>,

    dirty: bool,
    last_refresh: Instant,
}

impl GitTui {
    fn new(should_quit: Arc<AtomicBool>, theme: Theme) -> Self {
        let tabs = vec!["Status", "Log", "Diff", "Branches"];
        let mut tab_bar = TabBar::new_with_id(WidgetId::new(1), tabs);
        tab_bar.set_active(0);
        tab_bar.on_theme_change(&theme);

        let status_bar = StatusBar::new(WidgetId::new(2))
            .add_segment(StatusSegment::new("Git TUI").with_fg(theme.primary))
            .add_segment(StatusSegment::new("1-4: views | r: refresh | q: quit").with_fg(theme.fg_muted));

        let mut app = Self {
            should_quit,
            theme,
            area: Rect::new(0, 0, 80, 24),
            view: GitView::Status,
            tab_bar,
            files: Vec::new(),
            selected_file: 0,
            commits: Vec::new(),
            selected_commit: 0,
            branches: Vec::new(),
            selected_branch: 0,
            diff_content: String::new(),
            status_bar,
            toasts: Vec::new(),
            dirty: true,
            last_refresh: Instant::now(),
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        self.files = self.read_git_status();
        self.commits = self.read_git_log();
        self.branches = self.read_branches();
        self.last_refresh = Instant::now();
        self.dirty = true;
    }

    fn read_git_status(&self) -> Vec<GitFile> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        output.lines()
            .filter(|l| l.len() >= 3)
            .map(|l| GitFile {
                status: l.chars().next().unwrap_or('?'),
                path: l[3..].to_string(),
            })
            .collect()
    }

    fn read_git_log(&self) -> Vec<GitCommit> {
        let output = Command::new("git")
            .args(["log", "--oneline", "-20", "--format=%h|%an|%ar|%s"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        output.lines()
            .filter(|l| !l.is_empty())
            .map(|l| {
                let parts: Vec<&str> = l.splitn(4, '|').collect();
                GitCommit {
                    hash: parts.get(0).unwrap_or(&"").to_string(),
                    author: parts.get(1).unwrap_or(&"").to_string(),
                    date: parts.get(2).unwrap_or(&"").to_string(),
                    message: parts.get(3).unwrap_or(&"").to_string(),
                }
            })
            .collect()
    }

    fn read_branches(&self) -> Vec<GitBranch> {
        let output = Command::new("git")
            .args(["branch", "-a"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        output.lines()
            .filter(|l| !l.is_empty())
            .map(|l| {
                let current = l.starts_with("* ");
                let name = if current { l[2..].to_string() } else { l.trim().to_string() };
                let remote = name.starts_with("remotes/");
                GitBranch { name, current, remote }
            })
            .collect()
    }

    fn read_diff(&self, path: &str) -> String {
        Command::new("git")
            .args(["diff", "--color=never", "HEAD", "--", path])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_else(|| "No diff available".to_string())
    }

    fn read_full_diff(&self) -> String {
        Command::new("git")
            .args(["diff", "--color=never", "HEAD"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_else(|| "No changes to display".to_string())
    }

    fn toast(&mut self, msg: &str, kind: ToastKind) {
        let toast = Toast::new(WidgetId::new(100 + self.toasts.len()), msg)
            .with_kind(kind)
            .with_duration(Duration::from_secs(2))
            .with_theme(self.theme);
        self.toasts.push(toast);
        self.dirty = true;
    }

    fn is_git_repo(&self) -> bool {
        Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl Widget for GitTui {
    fn id(&self) -> WidgetId { WidgetId::new(0) }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect { self.area }
    fn set_area(&mut self, area: Rect) { self.area = area; self.dirty = true; }
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
    fn focusable(&self) -> bool { true }

    fn render(&self, area: Rect) -> Plane {
        let t = self.theme;
        let mut plane = Plane::new(0, area.width, area.height);

        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        let tab_h = 1u16;
        let status_h = 1u16;
        let content_y = tab_h;
        let content_h = area.height.saturating_sub(tab_h + status_h);

        // Tab bar
        let tab_plane = self.tab_bar.render(Rect::new(0, 0, area.width, tab_h));
        for (i, c) in tab_plane.cells.iter().enumerate() {
            if !c.transparent && i < plane.cells.len() {
                plane.cells[i] = c.clone();
            }
        }

        // Content
        if !self.is_git_repo() {
            let msg = "Not a git repository. Run 'git init' first.";
            let x = (area.width as usize - msg.len()) / 2;
            let y = content_y + content_h / 2;
            draw_text(&mut plane, x as u16, y, msg, t.error, t.bg, true);
        } else {
            match self.view {
                GitView::Status => self.render_status(&mut plane, content_y, content_h, t),
                GitView::Log => self.render_log(&mut plane, content_y, content_h, t),
                GitView::Diff => self.render_diff(&mut plane, content_y, content_h, t),
                GitView::Branches => self.render_branches(&mut plane, content_y, content_h, t),
            }
        }

        // Status bar
        let status_y = area.height.saturating_sub(1);
        let status_plane = self.status_bar.render(Rect::new(0, status_y, area.width, status_h));
        for (i, c) in status_plane.cells.iter().enumerate() {
            if !c.transparent && i < plane.cells.len() {
                let base = (status_y * area.width) as usize;
                if base + i < plane.cells.len() {
                    plane.cells[base + i] = c.clone();
                }
            }
        }

        // Toasts
        for (i, toast) in self.toasts.iter().enumerate() {
            let toast_y = status_y.saturating_sub(2 + i as u16);
            let toast_plane = toast.render(Rect::new(2, toast_y, area.width.saturating_sub(4), 1));
            for (j, c) in toast_plane.cells.iter().enumerate() {
                if !c.transparent && j < plane.cells.len() {
                    let base = (toast_y * area.width + 2) as usize;
                    if base + j < plane.cells.len() {
                        plane.cells[base + j] = c.clone();
                    }
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        match key.code {
            KeyCode::Char('q') => { self.should_quit.store(true, Ordering::SeqCst); true }
            KeyCode::Char('r') => { self.refresh(); self.toast("Refreshed", ToastKind::Info); true }
            KeyCode::Char('1') => { self.view = GitView::Status; self.tab_bar.set_active(0); self.dirty = true; true }
            KeyCode::Char('2') => { self.view = GitView::Log; self.tab_bar.set_active(1); self.dirty = true; true }
            KeyCode::Char('3') => { self.view = GitView::Diff; self.tab_bar.set_active(2); self.diff_content = self.read_full_diff(); self.dirty = true; true }
            KeyCode::Char('4') => { self.view = GitView::Branches; self.tab_bar.set_active(3); self.dirty = true; true }
            KeyCode::Char('d') => {
                if self.view == GitView::Status {
                    if let Some(file) = self.files.get(self.selected_file) {
                        self.diff_content = self.read_diff(&file.path);
                        self.view = GitView::Diff;
                        self.tab_bar.set_active(2);
                        self.dirty = true;
                    }
                }
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                match self.view {
                    GitView::Status => if self.selected_file + 1 < self.files.len() { self.selected_file += 1; self.dirty = true; }
                    GitView::Log => if self.selected_commit + 1 < self.commits.len() { self.selected_commit += 1; self.dirty = true; }
                    GitView::Branches => if self.selected_branch + 1 < self.branches.len() { self.selected_branch += 1; self.dirty = true; }
                    _ => {}
                }
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                match self.view {
                    GitView::Status => if self.selected_file > 0 { self.selected_file -= 1; self.dirty = true; }
                    GitView::Log => if self.selected_commit > 0 { self.selected_commit -= 1; self.dirty = true; }
                    GitView::Branches => if self.selected_branch > 0 { self.selected_branch -= 1; self.dirty = true; }
                    _ => {}
                }
                true
            }
            KeyCode::Enter => {
                if self.view == GitView::Branches {
                    if let Some(branch) = self.branches.get(self.selected_branch) {
                        if !branch.current {
                            let output = Command::new("git")
                                .args(["checkout", &branch.name])
                                .output();
                            match output {
                                Ok(o) if o.status.success() => {
                                    self.toast(&format!("Switched to {}", branch.name), ToastKind::Success);
                                    self.refresh();
                                }
                                _ => self.toast("Checkout failed", ToastKind::Error),
                            }
                        }
                    }
                } else if self.view == GitView::Status {
                    if let Some(file) = self.files.get(self.selected_file) {
                        let action = match file.status {
                            ' ' | 'M' | 'A' => "reset",
                            _ => "add",
                        };
                        let _ = Command::new("git").args([action, &file.path]).output();
                        self.refresh();
                    }
                }
                true
            }
            _ => false,
        }
    }
}

impl GitTui {
    fn render_status(&self, plane: &mut Plane, y: u16, _h: u16, t: Theme) {
        let header = "Status";
        draw_text(plane, 2, y, header, t.primary, t.bg, true);

        let sub_y = y + 2;
        if self.files.is_empty() {
            draw_text(plane, 2, sub_y, "Working tree clean", t.success, t.bg, false);
        } else {
            let staged: Vec<_> = self.files.iter().filter(|f| f.status == 'A' || f.status == 'M').collect();
            let modified: Vec<_> = self.files.iter().filter(|f| f.status == 'M' || f.status == ' ').collect();
            let untracked: Vec<_> = self.files.iter().filter(|f| f.status == '?').collect();

            let mut row = sub_y;
            if !staged.is_empty() {
                draw_text(plane, 2, row, &format!("Staged ({}):", staged.len()), t.success, t.bg, true);
                row += 1;
                for (i, file) in staged.iter().enumerate() {
                    let is_selected = self.view == GitView::Status && self.selected_file == i;
                    let fg = if is_selected { t.fg_on_accent } else { t.fg };
                    let bg = if is_selected { t.primary_active } else { t.bg };
                    draw_text(plane, 4, row, &format!("{} {}", file.status, file.path), fg, bg, is_selected);
                    row += 1;
                }
                row += 1;
            }

            if !modified.is_empty() {
                draw_text(plane, 2, row, &format!("Modified ({}):", modified.len()), t.warning, t.bg, true);
                row += 1;
                let offset = staged.len();
                for (i, file) in modified.iter().enumerate() {
                    let idx = offset + i;
                    let is_selected = self.view == GitView::Status && self.selected_file == idx;
                    let fg = if is_selected { t.fg_on_accent } else { t.fg };
                    let bg = if is_selected { t.primary_active } else { t.bg };
                    draw_text(plane, 4, row, &format!("{} {}", file.status, file.path), fg, bg, is_selected);
                    row += 1;
                }
                row += 1;
            }

            if !untracked.is_empty() {
                draw_text(plane, 2, row, &format!("Untracked ({}):", untracked.len()), t.fg_muted, t.bg, true);
                row += 1;
                let offset = staged.len() + modified.len();
                for (i, file) in untracked.iter().enumerate() {
                    let idx = offset + i;
                    let is_selected = self.view == GitView::Status && self.selected_file == idx;
                    let fg = if is_selected { t.fg_on_accent } else { t.fg };
                    let bg = if is_selected { t.primary_active } else { t.bg };
                    draw_text(plane, 4, row, &format!("{} {}", file.status, file.path), fg, bg, is_selected);
                    row += 1;
                }
            }
        }
    }

    fn render_log(&self, plane: &mut Plane, y: u16, _h: u16, t: Theme) {
        let header = "Commit History";
        draw_text(plane, 2, y, header, t.primary, t.bg, true);

        for (i, commit) in self.commits.iter().enumerate() {
            let row = y + 2 + i as u16;
            let is_selected = self.view == GitView::Log && self.selected_commit == i;
            let fg = if is_selected { t.fg_on_accent } else { t.fg };
            let bg = if is_selected { t.primary_active } else { t.bg };

            let hash = &commit.hash;
            let msg = if commit.message.len() > 40 { &commit.message[..40] } else { &commit.message };
            let line = format!("{} │ {} │ {}", hash, &commit.date[..10.min(commit.date.len())], msg);
            draw_text(plane, 2, row, &line, fg, bg, is_selected);
        }
    }

    fn render_diff(&self, plane: &mut Plane, y: u16, h: u16, t: Theme) {
        let header = "Diff";
        draw_text(plane, 2, y, header, t.primary, t.bg, true);

        let lines: Vec<&str> = self.diff_content.lines().collect();
        let visible = (h as usize).saturating_sub(3);

        for (i, line) in lines.iter().take(visible).enumerate() {
            let row = y + 2 + i as u16;
            let (fg, bold) = if line.starts_with('+') && !line.starts_with("+++") {
                (t.success, false)
            } else if line.starts_with('-') && !line.starts_with("---") {
                (t.error, false)
            } else if line.starts_with("@@") {
                (t.info, true)
            } else if line.starts_with("diff ") || line.starts_with("index ") || line.starts_with("--- ") || line.starts_with("+++ ") {
                (t.fg_muted, true)
            } else {
                (t.fg, false)
            };

            let truncated = if line.len() > plane.width as usize - 4 { &line[..plane.width as usize - 4] } else { line };
            draw_text(plane, 2, row, truncated, fg, t.bg, bold);
        }

        if lines.len() > visible {
            let more = format!("... {} more lines", lines.len() - visible);
            draw_text(plane, 2, y + h - 2, &more, t.fg_muted, t.bg, false);
        }
    }

    fn render_branches(&self, plane: &mut Plane, y: u16, _h: u16, t: Theme) {
        let header = "Branches";
        draw_text(plane, 2, y, header, t.primary, t.bg, true);

        let locals: Vec<_> = self.branches.iter().filter(|b| !b.remote).collect();
        let remotes: Vec<_> = self.branches.iter().filter(|b| b.remote).collect();

        let mut row = y + 2;
        if !locals.is_empty() {
            draw_text(plane, 2, row, "Local:", t.secondary, t.bg, true);
            row += 1;
            for (i, branch) in locals.iter().enumerate() {
                let is_selected = self.view == GitView::Branches && self.selected_branch == i;
                let fg = if branch.current { t.success } else if is_selected { t.fg_on_accent } else { t.fg };
                let bg = if is_selected { t.primary_active } else { t.bg };
                let marker = if branch.current { "* " } else { "  " };
                draw_text(plane, 4, row, &format!("{}{}", marker, branch.name), fg, bg, branch.current || is_selected);
                row += 1;
            }
            row += 1;
        }

        if !remotes.is_empty() {
            draw_text(plane, 2, row, "Remote:", t.secondary, t.bg, true);
            row += 1;
            let offset = locals.len();
            for (i, branch) in remotes.iter().enumerate() {
                let idx = offset + i;
                let is_selected = self.view == GitView::Branches && self.selected_branch == idx;
                let fg = if is_selected { t.fg_on_accent } else { t.fg_muted };
                let bg = if is_selected { t.primary_active } else { t.bg };
                draw_text(plane, 4, row, &format!("  {}", branch.name), fg, bg, is_selected);
                row += 1;
            }
        }
    }
}

fn draw_text(plane: &mut Plane, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    for (i, ch) in text.chars().enumerate() {
        let idx = (y * plane.width + x + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: ch,
                fg,
                bg,
                style: if bold { Styles::BOLD } else { Styles::empty() },
                transparent: false,
                skip: false,
            };
        }
    }
}

fn main() -> std::io::Result<()> {
    println!("Git TUI — Real Git interface");
    println!("1-4: views | r: refresh | q: quit");
    std::thread::sleep(Duration::from_millis(300));

    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let theme = Theme::nord();
    let git = GitTui::new(should_quit, theme);

    let mut app = App::new()?.title("Git TUI").fps(30).theme(theme);
    app.add_widget(Box::new(git), Rect::new(0, 0, w, h));

    app.on_tick(move |ctx, _| {
        if quit_check.load(Ordering::SeqCst) { ctx.stop(); }
    }).run(|_ctx| {});

    println!("\nGit TUI exited cleanly");
    Ok(())
}
