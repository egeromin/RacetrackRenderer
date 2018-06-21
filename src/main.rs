extern crate pxl;
use pxl::*;


const RACETRACK: &[u8] = include_bytes!("../racetrack.values");
const EPISODE: &[u8] = include_bytes!("../episode.values");
const TICKS_PER_SECOND: u32 = 60;
// const NUM_EPISODES: usize = EPISODE.len() / 2;


#[derive(Copy,Clone)]
enum TrackPoint {
    Boundary,
    End,
    Beginning,
    Empty,
}

use TrackPoint::*;


impl TrackPoint {

    fn from_byte(byte:u8) -> TrackPoint {
        match byte {
            0 => Boundary,
            1 => End,
            2 => Beginning,
            3 => Empty,
            _ => panic!("No phallic racetrack: got :{}", byte),
        }
    }

    fn pixel(self) -> Pixel {
        match self {
            Boundary => Pixel{red:0,green:0,blue:0},
            End => Pixel{red:0,green:255,blue:0},
            Beginning => Pixel{red:255,green:0,blue:0},
            Empty => Pixel{red:255,green:255,blue:255},
        }
    }
}

struct Game {
    track: Vec<TrackPoint>,
    episode_num: usize,
    tick: u32
}

impl Program for Game {
    fn new() -> Game {
        assert_eq!(RACETRACK.len(), 2500);
        let track = RACETRACK.iter().cloned().map(TrackPoint::from_byte).collect();
        Game{track, episode_num: 0, tick: 0}
    }

    fn render(&mut self, pixels: &mut [Pixel]) {
        for x in 0..50 {
            for y in 0..50 {
                let pixel = self.track[y*50 + x].pixel();
                for i in 0..5 {
                    for j in 0..5 {
                        pixels[(5*y+j+3)*256 + 5*x + i+3] = pixel;
                    }
                }
            }
        }
    }

    fn tick(&mut self, _events: &[Event]) {

        self.tick += 1;

        if self.tick % TICKS_PER_SECOND == 0 {

            // let (x, y) = EPISODE[2*self.episode_num: 2*self.episode_num+1]
            let x =  (EPISODE[2*self.episode_num] - 1) as usize;
            let y =  (EPISODE[2*self.episode_num+1] -1) as usize;

            self.track[50*y + x] = Boundary;
            self.episode_num = (self.episode_num + 1) % (EPISODE.len() / 2);

            if self.episode_num == 0 {
                self.track = RACETRACK.iter().cloned().map(TrackPoint::from_byte).collect();
            }

            self.tick = 0;
        }

    }
}

fn main() {
    run::<Game>();
}
