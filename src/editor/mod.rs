//! The editor (F2): a screen of its own, its pages reached by the drop-down at the top [David, 2026-11-12: "the new
//! editor should have a drop-down menu at the top that brings you to the desired page (software map, tectonics, world
//! gen, registrar, etc)"].
//!
//! Only the software map exists yet; the other pages are listed greyed until they do (grey out, don't hide). Where
//! every setting and control goes, and what each is called, is David's, asked before it's placed [2026-11-12: "organizing
//! the UI is super important to me"]. The bar at the top holds the drop-down and nothing else.

mod software_map;

use crate::clocks::PassTimes;

/// The editor's pages, in the drop-down's order.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    SoftwareMap,
    Tectonics,
    WorldGen,
    Registrar,
}

impl Page {
    const ALL: [Page; 4] = [Page::SoftwareMap, Page::Tectonics, Page::WorldGen, Page::Registrar];

    /// Its name in the drop-down, in David's words [2026-11-12: "we should probably call them Tectonic Sim, World Gen, and
    /// Registrar, since 'compiler/interpreter' would go inside the world gen page"].
    fn name(self) -> &'static str {
        match self {
            Page::SoftwareMap => "Software map",
            Page::Tectonics => "Tectonic Sim",
            Page::WorldGen => "World Gen",
            Page::Registrar => "Registrar",
        }
    }

    /// Whether the page is built yet.
    fn exists(self) -> bool {
        self == Page::SoftwareMap
    }
}

pub struct Editor {
    pub open: bool,
    page: Page,
}

impl Editor {
    pub fn new() -> Self {
        Self { open: false, page: Page::SoftwareMap }
    }

    /// The editor over the whole screen, if it's open; `times` are each pass's recent times, by its id.
    pub fn show(&mut self, ctx: &egui::Context, times: &[PassTimes]) {
        if !self.open {
            return;
        }
        let mut root = egui::Ui::new(ctx.clone(), "the editor".into(), egui::UiBuilder::new().layer_id(egui::LayerId::background()).max_rect(ctx.viewport_rect()));
        egui::Panel::top("the editor's bar").frame(egui::Frame::new().fill(egui::Color32::BLACK).inner_margin(6.0)).show(&mut root, |ui| {
            egui::ComboBox::from_id_salt("the editor's page").selected_text(self.page.name()).show_ui(ui, |ui| {
                for page in Page::ALL {
                    ui.add_enabled_ui(page.exists(), |ui| {
                        ui.selectable_value(&mut self.page, page, page.name());
                    });
                }
            });
        });
        egui::CentralPanel::no_frame().show(&mut root, |ui| match self.page {
            Page::SoftwareMap => software_map::draw(ui, times),
            Page::Tectonics | Page::WorldGen | Page::Registrar => {}
        });
    }
}
