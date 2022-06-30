use std::{env};

use ffmpeg::{
    codec::packet::Packet,
    format::{input},
    media::Type,
    format::stream::Stream,
};

pub struct StreamCounter {
    ty: Type,
    index: usize,
    base: f64,
    count: u64,
    last_key: u64,
    last_pts: Option<i64>,
}
impl StreamCounter {
    pub fn new(stream: Stream<'_>) -> Self {
        let decoder = stream.codec().decoder();
        let time_base = decoder.time_base();
        let ty = decoder.medium();
        Self {
            ty,
            index: stream.index(),
            base: time_base.numerator() as f64 / time_base.denominator() as f64,
            count: 0,
            last_key: 0,
            last_pts: None,
        }
    }
    pub fn index(&self) -> usize { self.index } 
    pub fn consume(&mut self, packet: &Packet) {
        if let (true, Some(pts)) = (packet.is_key(), packet.pts()) {
            let bytes_delta = self.count - self.last_key;
            if let (true, Some(last_pts)) = (bytes_delta > 0, self.last_pts) {
                let pts_delta = pts - last_pts;
                let bps = bytes_delta as f64 / (pts_delta as f64 * self.base);
                let time = pts as f64 * self.base;
                println!("[{}] ty={:?} time={}, bps={}", self.index, self.ty, time, bps);
            }
            self.last_key = self.count;
            self.last_pts = Some(pts);
        }
        self.count += packet.size() as u64;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg::init()
        .expect("couldn't load or init ffmpeg");
    let infile = env::args().nth(1)
        .expect("please specify input file as command line argument");

    let mut ictx = input(&infile)?;
    println!("{:?}", ictx.metadata());
    let mut counters = ictx.streams().map(|stream| {
        StreamCounter::new(stream)
    }).collect::<Vec<_>>();

    // time base for audio is borken?
    if let Some(time_base) = counters.iter().find_map(|counter|
        match counter.ty {
            Type::Video => Some(counter.base),
            _ => None
        }
    ) {
        for counter in counters.iter_mut() {
            counter.base = time_base;
        }
    }

    for res in ictx.packets() {
        let (stream, packet) = res?;
        if let Some(counter) = counters.iter_mut().find(|d| d.index() == stream.index()) {
            counter.consume(&packet);
        }
    }
    
    Ok(())
}
