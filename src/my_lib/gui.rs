use std::path::PathBuf;

use eframe::{
    egui::{self, Button, CentralPanel, Context, RichText, TextBuffer, TextEdit, Grid},
    run_native, App, Frame, NativeOptions,
};

const WIN_WIDTH: f32 = 500.;
const WIN_HEIGHT: f32 = 640.;

#[derive(Default)]
struct Gui {
    starts: Vec<String>,
}

impl Gui {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            starts: vec![String::new()],
        }
    }
}

impl App for Gui {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Start");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, command) in self.starts.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        let start = ui.label((index + 1).to_string() + ".");

                        let cmd = TextEdit::singleline(command)
                            .hint_text("Command to be run.")
                            .min_size([WIN_WIDTH-100., 10.].into())
                            .show(ui);

                        // let cmd = ui.text_edit_singleline(command).labelled_by(start.id);
                    });
                }
            });
            ui.vertical_centered(|ui| {
                if ui.add_sized([100., 20.],  Button::new("+")).clicked() {
                    self.starts.push(String::new());
                }
            });
            ui.separator();
            ui.heading("Options");
            ui.separator();

            Grid::new("options_grid").show(ui, |ui| {
                ui.checkbox(&mut false, "aaa");
            });

        });
    }
}

pub fn gui() -> anyhow::Result<()> {
    let native_options = NativeOptions {
        // always_on_top: todo!(),
        // decorated: todo!(),
        // fullscreen: todo!(),
        // drag_and_drop_support: todo!(),
        // icon_data: todo!(),
        // initial_window_pos: todo!(),
        // min_window_size: todo!(),
        // transparent: todo!(),
        // mouse_passthrough: todo!(),
        // vsync: todo!(),
        // multisampling: todo!(),
        // depth_buffer: todo!(),
        // stencil_buffer: todo!(),
        // hardware_acceleration: todo!(),
        // default_theme: todo!(),
        // run_and_return: todo!(),
        // event_loop_builder: todo!(),
        // shader_version: todo!(),
        initial_window_size: Some([WIN_WIDTH, WIN_HEIGHT].into()),
        resizable: false,
        follow_system_theme: true,
        centered: true,
        ..Default::default()
    };

    run_native("SMA", native_options, Box::new(|cc| Box::new(Gui::new(cc))))
        .or_else(|e| anyhow::bail!(e.to_string()))?;

    Ok(())
}
