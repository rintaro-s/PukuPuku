use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use crate::benchmark;
use crate::i18n::i18n;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/rinta/PukuPuku/ui/benchmark_page/page.ui")]
    pub struct BenchmarkPage {
        #[template_child]
        pub run_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub cpu_only_check: TemplateChild<gtk::CheckButton>,

        #[template_child]
        pub spinner: TemplateChild<gtk::Spinner>,

        #[template_child]
        pub status_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub cpu_value_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub io_write_value_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub io_read_value_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BenchmarkPage {
        const NAME: &'static str = "BenchmarkPage";
        type Type = super::BenchmarkPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for BenchmarkPage {
        fn constructed(&self) {
            self.parent_constructed();

            self.run_button.connect_clicked({
                let weak = self.obj().downgrade();
                move |_| {
                    if let Some(obj) = weak.upgrade() {
                        obj.run_benchmarks();
                    }
                }
            });
        }
    }

    impl WidgetImpl for BenchmarkPage {}
    impl BinImpl for BenchmarkPage {}
}

glib::wrapper! {
    pub struct BenchmarkPage(ObjectSubclass<imp::BenchmarkPage>)
    @extends adw::Bin, gtk::Widget,
    @implements gtk::ConstraintTarget, gtk::Accessible, gtk::Buildable;
}

impl BenchmarkPage {
    pub fn run_benchmarks(&self) {
        let imp = self.imp();

        imp.run_button.set_sensitive(false);
        imp.cpu_only_check.set_sensitive(false);
        imp.spinner.set_visible(true);
        imp.spinner.start();
        imp.status_label.set_label(&i18n("Running\u{2026}"));

        let cpu_only = imp.cpu_only_check.is_active();
        let io_dir = std::env::temp_dir().join("pukupuku");

        let (sender, receiver) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            if cpu_only {
                let cpu_result = benchmark::run_cpu_only_benchmark();
                let _ = sender.send(Ok((cpu_result, None)));
            } else {
                match benchmark::run_default_benchmarks(&io_dir) {
                    Ok(result) => {
                        let _ = sender.send(Ok((result.cpu, Some(result.io))));
                    }
                    Err(err) => {
                        let _ = sender.send(Err(err));
                    }
                }
            }
        });

        let weak = self.downgrade();

        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            let result = match receiver.try_recv() {
                Ok(result) => result,
                Err(std::sync::mpsc::TryRecvError::Empty) => return glib::ControlFlow::Continue,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    let Some(this) = weak.upgrade() else {
                        return glib::ControlFlow::Break;
                    };
                    let imp = this.imp();
                    imp.status_label
                        .set_label(&format!("{}", i18n("Failed")));
                    imp.spinner.stop();
                    imp.spinner.set_visible(false);
                    imp.run_button.set_sensitive(true);
                    return glib::ControlFlow::Break;
                }
            };

            let Some(this) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };

            let imp = this.imp();

            match result {
                Ok((cpu_result, io_result)) => {
                    imp.cpu_value_label
                        .set_label(&format!("{:.0}", cpu_result.iterations_per_second));

                    if let Some(io_res) = io_result {
                        imp.io_write_value_label
                            .set_label(&format!("{:.1}", io_res.write_mebibytes_per_second));
                        imp.io_read_value_label
                            .set_label(&format!("{:.1}", io_res.read_mebibytes_per_second));
                    } else {
                        imp.io_write_value_label.set_label("-");
                        imp.io_read_value_label.set_label("-");
                    }

                    imp.status_label.set_label(&i18n("Done"));
                }
                Err(err) => {
                    imp.status_label
                        .set_label(&format!("{}: {}", i18n("Failed"), err));
                }
            }

            imp.spinner.stop();
            imp.spinner.set_visible(false);
            imp.run_button.set_sensitive(true);
            imp.cpu_only_check.set_sensitive(true);

            glib::ControlFlow::Break
        });
    }
}
