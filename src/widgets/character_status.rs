use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::i18n::i18n;
use crate::magpie_client::Readings;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum CharacterMood {
    Normal,
    VeryHard,
    Danger,
}

impl CharacterMood {
    fn label(&self) -> String {
        match self {
            CharacterMood::Normal => i18n("Doing fine"),
            CharacterMood::VeryHard => i18n("Working hard"),
            CharacterMood::Danger => i18n("Might be in trouble"),
        }
    }

    fn resource_path(&self) -> &'static str {
        match self {
            CharacterMood::Normal => "/io/github/rinta/PukuPuku/characters/normal.png",
            CharacterMood::VeryHard => "/io/github/rinta/PukuPuku/characters/very_hard.png",
            CharacterMood::Danger => "/io/github/rinta/PukuPuku/characters/danger.png",
        }
    }
}

fn compute_memory_usage_percent(readings: &Readings) -> u32 {
    let mem_total = if readings.mem_info.mem_total > 0 {
        readings.mem_info.mem_total
    } else {
        1
    };

    let mem_avail = if readings.mem_info.mem_available > readings.mem_info.mem_total {
        readings.mem_info.mem_free
    } else {
        readings.mem_info.mem_available
    };

    let memory_used = mem_total.saturating_sub(mem_avail);
    ((memory_used as f32 * 100. / mem_total as f32).round() as u32).min(100)
}

fn compute_drive_usage_percent(readings: &Readings) -> u32 {
    if readings.disks_info.is_empty() {
        return 0;
    }

    let mut sum = 0.0f32;
    for disk in &readings.disks_info {
        sum += disk.busy_percent;
    }

    ((sum / readings.disks_info.len() as f32).round() as u32).min(100)
}

fn mood_from_readings(readings: &Readings) -> CharacterMood {
    let cpu = readings.cpu.total_usage_percent.round() as u32;
    let mem = compute_memory_usage_percent(readings);
    let drive = compute_drive_usage_percent(readings);

    if cpu >= 90 || mem >= 90 || drive >= 90 {
        CharacterMood::Danger
    } else if cpu >= 60 || mem >= 75 || drive >= 75 {
        CharacterMood::VeryHard
    } else {
        CharacterMood::Normal
    }
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/rinta/PukuPuku/ui/widgets/character_status.ui")]
    pub struct CharacterStatusWidget {
        #[template_child]
        pub character_picture: TemplateChild<gtk::Image>,

        #[template_child]
        pub mood_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CharacterStatusWidget {
        const NAME: &'static str = "CharacterStatusWidget";
        type Type = super::CharacterStatusWidget;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CharacterStatusWidget {
        fn constructed(&self) {
            self.parent_constructed();
            self.mood_label.set_label("-");

            // Connect to GSettings changes for character-size
            let obj = self.obj();
            let settings = crate::settings!();
            settings.connect_changed(Some("character-size"), {
                let obj = obj.downgrade();
                move |_, _| {
                    if let Some(obj) = obj.upgrade() {
                        obj.on_settings_changed();
                    }
                }
            });

            obj.on_settings_changed();
        }
    }

    impl WidgetImpl for CharacterStatusWidget {}
    impl BinImpl for CharacterStatusWidget {}
}

glib::wrapper! {
    pub struct CharacterStatusWidget(ObjectSubclass<imp::CharacterStatusWidget>)
    @extends adw::Bin, gtk::Widget,
    @implements gtk::ConstraintTarget, gtk::Accessible, gtk::Buildable;
}

impl CharacterStatusWidget {
    pub fn update_from_readings(&self, readings: &Readings) {
        let mood = mood_from_readings(readings);
        let imp = self.imp();

        imp.mood_label.set_label(&mood.label());

        // Only set the resource if it exists to avoid warnings.
        if gio::resources_lookup_data(mood.resource_path(), gio::ResourceLookupFlags::NONE).is_ok() {
            imp.character_picture.set_resource(Some(mood.resource_path()));
        } else {
            imp.character_picture.set_resource(None);
        }

        // Update size from settings
        let size = crate::settings!().int("character-size").clamp(40, 800);
        imp.character_picture.set_pixel_size(size);
    }

    pub fn on_settings_changed(&self) {
        let size = crate::settings!().int("character-size").clamp(40, 800);
        self.imp().character_picture.set_pixel_size(size);
    }
}
