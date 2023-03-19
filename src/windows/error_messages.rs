use crate::windows::client_windows::{GenericWindow, Loglet};
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct ErrorMessage {
    pub display: GenericWindow,
    pub receiver: Receiver<Loglet>,
    pub sender: Sender<Loglet>,
}

impl ErrorMessage {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        ErrorMessage {
            display: GenericWindow::default(),
            receiver,
            sender,
        }
    }

    pub fn try_update_log(&mut self) {
        //! Receive any error messages from differnt Threads, Tokios, Async fns
        self.receiver.try_iter().for_each(|loglet| {
            GenericWindow::push_loglet(&mut self.display, loglet);
        });
    }
    pub fn sender_clone(&self) -> Sender<Loglet> {
        //! Provide a sender for async functions or new Threads/Tokios...
        self.sender.clone()
    }
}
