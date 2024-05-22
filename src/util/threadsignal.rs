use std::{cell::RefCell, future::Future, pin::Pin, rc::Rc};

pub struct MonoThreadConnection<T> {
    pub callback: Box<dyn FnMut(&T) -> ()>,
    pub once: bool,
    pub disconnected: bool
}

impl<T> PartialEq for MonoThreadConnection<T> {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

/**
 * A signal handler that only works on current thread
 */
pub struct MonoThreadSignal<T> {
    connections: Vec<Rc<RefCell<MonoThreadConnection<T>>>>
}

impl<T: Send + Sync> MonoThreadSignal<T> {
    pub fn new() -> Self {
        Self {
            connections: Vec::new()
        }
    }

    pub fn remove_connection(&mut self, connection: &Rc<RefCell<MonoThreadConnection<T>>>) {
        if self.connections.len() == 0 {return;}
        else if self.connections.len() == 1 {
            if self.connections.get(0).unwrap() == connection {
                self.connections.pop();
            }
            //otherwise, it may have been called without checking if the connection was already disconnected
        }
        else {
            let index = self.connections.iter().position(|v| v == connection);
            match index {
                Some(i) =>{ self.connections.swap_remove(i); },
                None => {}
            }
        }
    }

    pub async fn dispatch(&mut self, value: T) {
        for connection in &mut self.connections {
            let callback = connection.as_ref().borrow_mut().callback;
            tokio::task::spawn(callback(&value));
        }
    }
    /// get a connection to an event that may happen in the future and may or may not run on a separate thread
    /// 
    /// Example Usage:
    /// 
    /// ```
    /// signal.connect(|v| Box::pin(async {
    ///    println!("Hello World!");
    /// }));
    /// ```
    pub fn connect<R>(&mut self, callback: R) -> Rc<RefCell<MonoThreadConnection<T>>> where R: FnMut(&T) {
        let c = Rc::new(RefCell::new(MonoThreadConnection {
            callback: Box::new(callback),
            once: false,
            disconnected: false
        }));

        self.connections.push(c.clone());

        c
    }
}