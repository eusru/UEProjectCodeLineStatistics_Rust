use eframe::egui;
use rfd::FileDialog;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

// 全局排除目录
const EXCLUDE_DIR: [&str; 4] = ["Intermediate", "Binaries", "Saved", ".vs"];
// 需要统计的代码文件后缀
const INCLUDE_EXT: [&str; 3] = ["h", "cpp", "inl"];

#[derive(Default)]
struct StatResult {
    files: usize,
    total_lines: usize,
    code_lines: usize,
}

struct UELocApp {
    root_dir: Option<PathBuf>,
    result: StatResult,
    font_inited: bool,
}

impl Default for UELocApp {
    fn default() -> Self {
        Self {
            root_dir: None,
            result: StatResult::default(),
            font_inited: false,
        }
    }
}

/* ---------------- 字体初始化 ---------------- */
fn init_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "wenkai".to_owned(),
        egui::FontData::from_static(include_bytes!("../fonts/LXGWWenKai-Regular.ttf")),
    );

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "wenkai".to_owned());

    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(0, "wenkai".to_owned());

    ctx.set_fonts(fonts);
}

/* ---------------- 统计逻辑 ---------------- */
fn is_comment_or_empty(line: &str) -> bool {
    let s = line.trim();
    s.is_empty() || s.starts_with("//") || s.starts_with("/*") || s.starts_with('*')
}

fn should_skip(path: &Path) -> bool {
    // 全局排除
    if path.components().any(|c| {
        let name = c.as_os_str().to_string_lossy();
        EXCLUDE_DIR.iter().any(|d| *d == name)
    }) {
        return true;
    }

    // Plugins 特殊排除
    let components: Vec<_> = path.components().collect();
    for (i, comp) in components.iter().enumerate() {
        let name = comp.as_os_str().to_string_lossy();
        if name == "Plugins" {
            if let Some(next) = components.get(i + 1) {
                let next_name = next.as_os_str().to_string_lossy();
                if next_name == "Intermediate" || next_name == "ThirdParty" {
                    return true;
                }
            }
        }
    }

    false
}

fn should_count(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| INCLUDE_EXT.contains(&e))
        .unwrap_or(false)
}

fn stat_ue_code(root: &Path) -> StatResult {
    let mut result = StatResult::default();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        if should_skip(path) || !should_count(path) {
            continue;
        }

        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            result.files += 1;

            for line in reader.lines().flatten() {
                result.total_lines += 1;
                if !is_comment_or_empty(&line) {
                    result.code_lines += 1;
                }
            }
        }
    }

    result
}

/* ---------------- GUI ---------------- */
impl eframe::App for UELocApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if !self.font_inited {
            init_fonts(ctx);
            self.font_inited = true;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // 整体居中 + 列式布局
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading("UE 工程代码统计工具");
                ui.add_space(30.0);

                // 选择目录按钮
                if ui.add_sized([200.0, 40.0], egui::Button::new("选择 UE 工程目录")).clicked() {
                    if let Some(dir) = FileDialog::new().pick_folder() {
                        self.result = stat_ue_code(&dir);
                        self.root_dir = Some(dir);
                    }
                }

                ui.add_space(30.0);
                ui.separator();
                ui.add_space(20.0);

                // 显示结果
                if let Some(dir) = &self.root_dir {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.add_space(10.0);
                            ui.label(format!("工程路径：{}", dir.display()));
                            ui.add_space(5.0);
                            ui.label(format!("文件数量：{}", self.result.files));
                            ui.add_space(5.0);
                            ui.label(format!("总代码行数：{}", self.result.total_lines));
                            ui.add_space(5.0);
                            ui.label(format!("有效代码行数：{}", self.result.code_lines));
                            ui.add_space(10.0);
                        });
                    });
                } else {
                    ui.group(|ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label("尚未选择工程目录");
                        });
                    });
                }
            });
        });
    }
}


/* ---------------- main ---------------- */
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "UE LOC GUI",
        options,
        Box::new(|_cc| Box::new(UELocApp::default())),
    )
}
