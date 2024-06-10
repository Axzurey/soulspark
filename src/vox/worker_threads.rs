use std::{collections::HashMap, sync::{atomic::AtomicI64, mpsc::{self, Sender}, Arc}, thread};

use parking_lot::RwLock;

use crate::engine::surfacevertex::SurfaceVertex;

use super::{chunk::Chunk, chunk_manager::mesh_slice_arrayed};

pub fn spawn_chunk_worker_thread(
    id: usize,
    send_back: Sender<(usize, i32, i32, u32, ((Vec<SurfaceVertex>, Vec<u32>, u32), (Vec<SurfaceVertex>, Vec<u32>, u32)))>
) -> Sender<(i32, i32, u32, HashMap<u32, Arc<RwLock<Chunk>>>)> {
    let (send, recv) = mpsc::channel();
    

    thread::spawn(move || {
        while let Ok((chunk_x, chunk_z, y_slice, chunks)) = recv.recv() {
            let result = mesh_slice_arrayed(chunk_x, chunk_z, y_slice, &chunks);
            send_back.send((id, chunk_x, chunk_z, y_slice, result)).unwrap();
        }
        
    });

    send
}

pub fn spawn_chunk_loop(
    num_workers: usize
) {
    let (frommain, frommainrecv) = mpsc::channel();
    let (tomain, tomainrecv) = mpsc::channel();
    let (sendworker, recvworker) = mpsc::channel();

    let available_chunks = AtomicI64::new(0);

    thread::spawn(move || {
        while let Ok(get) = recvworker.recv() {
            tomain.send(get).unwrap();
        }
    });
    thread::spawn(move || {
        let workers = (0..num_workers).map(|id| {
            spawn_chunk_worker_thread(id, sendworker.clone())
        }).collect::<Vec<_>>();

        while let Ok(get) = frommainrecv.recv() {
            
        }
    });

    (frommain, tomainrecv)
}