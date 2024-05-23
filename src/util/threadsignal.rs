use std::{future::Future, pin::Pin, process::Output, rc::Rc, sync::{Arc, RwLock}};

pub struct MonoThreadConnection<T> {
    pub callback: Box<dyn Fn(T) -> () + Send + Sync + 'static>,
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
    connections: Vec<Arc<RwLock<MonoThreadConnection<T>>>>
}

impl<T> MonoThreadSignal<T> where T: Send + Clone + Sync + 'static {
    pub fn new() -> Self {
        Self {
            connections: Vec::new()
        }
    }

    pub fn remove_connection(&mut self, connection: &Arc<RwLock<MonoThreadConnection<T>>>) {
        if self.connections.len() == 0 {return;}
        else if self.connections.len() == 1 {
            if *self.connections.get(0).unwrap().read().unwrap() == *connection.read().unwrap() {
                self.connections.pop();
            }
            //otherwise, it may have been called without checking if the connection was already disconnected
        }
        else {
            let index = self.connections.iter().position(|v| *v.read().unwrap() == *connection.read().unwrap());
            match index {
                Some(i) =>{ self.connections.swap_remove(i); },
                None => {}
            }
        }
    }

    pub async fn dispatch(&mut self, value: T) where T: Copy + Send + Sync {
        for connection in &mut self.connections {
            let copy = value.clone();
            let clonedfn = connection.clone();
            std::thread::spawn(move || {(clonedfn.read().unwrap().callback)(copy)});
            //tokio::task::spawn(((*connection.as_ref().borrow_mut()).callback)(&value));
        }
    }
    /// get a connection to an event that may happen in the future that runs on a different thread
    /// 
    /// Example Usage:
    /// 
    /// ```
    /// signal.connect(|v| {
    ///     println!("Hello World!");
    /// });
    /// ```
    pub fn connect(&mut self, callback: impl Fn(T) -> () + Send + Sync + 'static) -> Arc<RwLock<MonoThreadConnection<T>>> {
        let c = Arc::new(RwLock::new(MonoThreadConnection {
            callback: Box::new(callback),
            once: false,
            disconnected: false
        }));

        self.connections.push(c.clone());

        c
    }
}