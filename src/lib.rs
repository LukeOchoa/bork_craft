mod windows;

pub mod borkcraft;

pub use borkcraft::*;

mod increment {
    pub struct Inc {
        counter: i64,
    }
    impl Inc {
        pub fn up(&mut self) -> i64 {
            self.counter = self.counter + 1;
            self.counter
        }
        //pub fn down(&mut self) -> i64 {
        //    self.counter = self.counter - 1;
        //    self.counter
        //}
        pub fn reset(&mut self) -> i64 {
            self.counter = 0;
            self.counter
        }
        pub fn new() -> Inc {
            Inc { counter: 0 }
        }
    }
}

type MagicError = Box<dyn std::error::Error>;
fn try_access<T>(
    try_me: &std::sync::Arc<std::sync::Mutex<T>>,
    mut f: impl FnMut(std::sync::MutexGuard<T>),
) -> Result<(), MagicError> {
    match try_me.try_lock() {
        Ok(access) => {
            f(access);
            Ok(())
            // Do magic
        }
        Err(_) => Err(crate::err_tools::ErrorX::new_box(
            "try_access was used currently, try again later...",
        )),
    }
}

pub mod string_tools {
    fn quick_maker(amount: i32, character: &str) -> String {
        let mut s = String::default();
        for _ in 0..amount {
            s = format!("{}{}", s, character)
        }
        s
    }

    pub fn newliner(amount: i32) -> String {
        quick_maker(amount, "\n")
    }

    pub fn tabber(amount: i32) -> String {
        quick_maker(amount, "\t")
    }
}
pub mod err_tools {
    #[derive(Debug)]
    pub struct ErrorX {
        details: String,
    }

    impl ErrorX {
        pub fn _new(msg: &str) -> ErrorX {
            ErrorX {
                details: msg.to_string(),
            }
        }
        pub fn new_box(msg: &str) -> Box<ErrorX> {
            Box::new(ErrorX {
                details: msg.to_string(),
            })
        }
    }

    impl std::fmt::Display for ErrorX {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self.details)
        }
    }

    impl std::error::Error for ErrorX {
        fn description(&self) -> &str {
            &self.details
        }
    }
}

pub mod eframe_tools {
    use eframe::egui::{ScrollArea, Ui};
    pub fn scroll_and_vert(ui: &mut Ui, id: impl std::hash::Hash, f: impl Fn(&mut Ui)) {
        ScrollArea::vertical()
            .id_source(id)
            .show(ui, |ui| ui.horizontal_wrapped(|ui| f(ui)));
    }
}
