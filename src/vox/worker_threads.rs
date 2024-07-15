use std::{collections::HashMap, sync::{atomic::AtomicI64, mpsc::{self, Receiver, Sender}, Arc}, thread};

use cgmath::Vector2;
use noise::OpenSimplex;
use stopwatch::Stopwatch;

use crate::{engine::surfacevertex::SurfaceVertex, internal::depthsort::Quad};

use super::{chunk::Chunk, chunk_manager::mesh_slice_arrayed};

pub fn spawn_chunk_meshing_worker_thread(
    id: usize,
    send_back: Sender<(usize, i32, i32, u32, ((Vec<SurfaceVertex>, Vec<u32>, u32), (Vec<SurfaceVertex>, Vec<u32>, u32, Vec<Quad>)))>
) -> Sender<(i32, i32, u32, HashMap<u32, Arc<Chunk>>)> {
    let (send, recv) = mpsc::channel();
    

    thread::spawn(move || {
        while let Ok((chunk_x, chunk_z, y_slice, chunks)) = recv.recv() {
            let t = Stopwatch::start_new();
            let result = mesh_slice_arrayed(chunk_x, chunk_z, y_slice, &chunks);
            send_back.send((id, chunk_x, chunk_z, y_slice, result)).unwrap();
        }
        
    });

    send
}

pub fn spawn_chunk_creation_worker_thread(
    id: usize,
    seed: u32,
    send_back: Sender<(usize, i32, i32, Arc<Chunk>)>
) -> Sender<(i32, i32)> {
    let (send, recv) = mpsc::channel();
    let noisegen = OpenSimplex::new(seed);
    thread::spawn(move || {
        while let Ok((chunk_x, chunk_z)) = recv.recv() {
            let result = Arc::new(Chunk::new(Vector2::new(chunk_x, chunk_z), noisegen, &mut HashMap::new()));
            send_back.send((id, chunk_x, chunk_z, result)).unwrap();
        }
        
    });

    send
}

pub fn spawn_chunk_creation_loop(
    num_workers: usize,
    seed: u32
) -> (
    Sender<(i32, i32)>,
    Receiver<(i32, i32, Arc<Chunk>)>
) {
    //unapologetically stolen from elttob
    let (frommain, frommainrecv) = mpsc::channel();
    let (tomain, tomainrecv) = mpsc::channel();

    let (worker_send_finished_chunks, worker_recv_finished_chunks) = mpsc::channel();
    let (send_idle_worker, recv_idle_worker) = mpsc::channel();

    for id in 0..num_workers {
        send_idle_worker.send(id).unwrap();
    }

    thread::spawn(move || {
        let send_idle_worker = send_idle_worker.clone();
        loop {
            let data: (usize, i32, i32, Arc<Chunk>) = worker_recv_finished_chunks.recv().unwrap();
            let id = data.0.clone();
            tomain.send((data.1, data.2, data.3)).unwrap();
            send_idle_worker.send(id).unwrap();
        }
    });

    thread::spawn(move || {
        let mut workers = (0..num_workers).map(|id| {
            spawn_chunk_creation_worker_thread(id, seed, worker_send_finished_chunks.clone())
        }).collect::<Vec<_>>();

        loop {
            let next_data = frommainrecv.recv().unwrap();
            let next = recv_idle_worker.recv().unwrap();
            let worker = &mut workers[next];
            worker.send(next_data).unwrap();
        }
    });

    (frommain, tomainrecv)
}

pub fn spawn_chunk_meshing_loop(
    num_workers: usize
) -> (
    Sender<(i32, i32, u32, HashMap<u32, Arc<Chunk>>)>,
    Receiver<(i32, i32, u32, ((Vec<SurfaceVertex>, Vec<u32>, u32), (Vec<SurfaceVertex>, Vec<u32>, u32, Vec<Quad>)))>
) {
    //unapologetically stolen from elttob
    let (frommain, frommainrecv) = mpsc::channel();
    let (tomain, tomainrecv) = mpsc::channel();

    let (worker_send_finished_chunks, worker_recv_finished_chunks) = mpsc::channel();
    let (send_idle_worker, recv_idle_worker) = mpsc::channel();

    for id in 0..num_workers {
        send_idle_worker.send(id).unwrap();
    }

    thread::spawn(move || {
        let send_idle_worker = send_idle_worker.clone();
        loop {
            let data: (usize, i32, i32, u32, ((Vec<SurfaceVertex>, Vec<u32>, u32), (Vec<SurfaceVertex>, Vec<u32>, u32, Vec<Quad>))) = worker_recv_finished_chunks.recv().unwrap();
            let id = data.0.clone();
            tomain.send((data.1, data.2, data.3, data.4)).unwrap();
            send_idle_worker.send(id).unwrap();
        }
    });

    thread::spawn(move || {
        let mut workers = (0..num_workers).map(|id| {
            spawn_chunk_meshing_worker_thread(id, worker_send_finished_chunks.clone())
        }).collect::<Vec<_>>();

        loop {
            let next_data = frommainrecv.recv().unwrap();
            let next = recv_idle_worker.recv().unwrap();
            let worker = &mut workers[next];
            worker.send(next_data).unwrap();
        }
    });

    (frommain, tomainrecv)
}