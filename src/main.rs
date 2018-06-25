extern crate pxl;
extern crate rand;
extern crate byteorder;
#[macro_use] extern crate itertools;
use pxl::*;

use rand::{Rng, thread_rng};
use std::io::Cursor;
use byteorder::{NativeEndian, ReadBytesExt};
use std::f64;
use std::sync::{Arc, Mutex};
// use itertools::Itertools;


const RACETRACK: &[u8] = include_bytes!("../racetrack.values");
const EPISODE: &[u8] = include_bytes!("../episode.values");
const QVALS: &[u8] = include_bytes!("../q.values");


const TICKS_PER_SECOND: u32 = 60;
// const NUM_EPISODES: usize = EPISODE.len() / 2;

#[derive(Copy, Clone, PartialEq)]
enum TrackPoint {
    Boundary,
    End,
    Beginning,
    Empty,
}


use TrackPoint::*;
impl TrackPoint {
    fn from_byte(byte: u8) -> TrackPoint {
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
            Boundary => Pixel {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            },
            End => Pixel {
                red: 0.0,
                green: 1.0,
                blue: 0.0,
                alpha: 1.0,
            },
            Beginning => Pixel {
                red: 1.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            },
            Empty => Pixel {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 1.0,
            },
        }
    }
}

struct Game {
    track: Vec<TrackPoint>,
    start_positions: Vec<(i32, i32)>,
    episode_num: usize,
    tick: u32,
    qvalues: Vec<f64>,
    state: GameState,
    history: Vec<(i32, i32)>
}


fn start_state()-> GameState {
    GameState{
        position: (0, 0),
        velocity: (0, 0),
        ended: true,
        success: true
    }
}


impl Program for Game {
    fn new() -> Game {
        assert_eq!(RACETRACK.len(), 2500);
        let track = RACETRACK
            .iter()
            .cloned()
            .map(TrackPoint::from_byte)
            .collect::<Vec<TrackPoint>>();

        let start_positions: Vec<(i32, i32)> = iproduct!(0..50, 0..50)
            .into_iter()
            .filter(|&i| lookup_track(&track, i) == Beginning)
            .collect();

        let mut reader = Cursor::new(QVALS);

        let mut qvalues = vec![] as Vec<f64>;
        for i in 0..(QVALS.len()/8) {
            qvalues.push(reader.read_f64::<NativeEndian>().unwrap());
        }  // MUST IMPROVE

        for i in &start_positions { 
            let somestate = GameState{
                position: *i,
                velocity: (0, 0),
                ended: false,
                success: false
                };
            let action = Action{xacc: 1, yacc: 0};
            println!("({}, {}, {}, {}): {}", 
                     somestate.position.0,
                     somestate.position.1,
                     action.xacc,
                     action.yacc,
                     get_qvalue(&qvalues, somestate, action));
        }

        Game {
            track,
            start_positions,
            qvalues,
            episode_num: 0,
            tick: 0,
            state: start_state(),
            history: vec![]
        }
    }

    fn dimensions(&self) -> (usize, usize) {
        (506, 506)
    }

    fn render(&mut self, pixels: &mut [Pixel]) {
        for x in 0..50 {
            for y in 0..50 {
                let pixel = self.track[y * 50 + x].pixel();
                for i in 0..10 {
                    for j in 0..10 {
                        pixels[(10 * y + j + 3) * 506 + 10 * x + i + 3] = pixel;
                    }
                }
            }
        }
        for (x, y) in &self.history {
            let pixel = Boundary.pixel();
            for i in 0..10 {
                for j in 0..10 {
                    let pos = (10 * y + j + 3) * 506 + 10 * x + i + 3;
                    pixels[pos as usize] = pixel;
                }
            }
        }
    }

    fn tick(&mut self, _events: &[Event]) {
        self.tick += 1;

        if self.tick % TICKS_PER_SECOND == 0 {
            // let (x, y) = EPISODE[2*self.episode_num: 2*self.episode_num+1]
            // let x = (EPISODE[2 * self.episode_num] - 1) as usize;
            // let y = (EPISODE[2 * self.episode_num + 1] - 1) as usize;

            // self.track[50 * y + x] = Boundary;
            //

            if self.state.ended {
                self.state = random_start(&self.start_positions);
                self.history = vec![];
                self.history.push(self.state.position);
            } else {
                let action = greedy_action(&self.qvalues, self.state);
                println!("selected action {}, {}", action.xacc, action.yacc);
                self.state = next_state(&self.track, self.state, action);
                self.history.push(self.state.position);
            }

            // println!("state: {}, {}", self.state.position.0,
            //          self.state.position.1);
            if self.state.ended {
                println!("Ended with outcome {}", self.state.success);
            }
            
            // self.track[random_start(&self.start_positions)] = Boundary;
            // self.episode_num = (self.episode_num + 1) % (EPISODE.len() / 2);

            // if self.episode_num == 0 {
            //     self.track = RACETRACK
            //         .iter()
            //         .cloned()
            //         .map(TrackPoint::from_byte)
            //         .collect();
            // }

            // self.tick = 0;
        }
    }

    fn synthesizer(&self) -> Option<Arc<Mutex<Synthesizer>>> {
        Some(Arc::new(Mutex::new(Carillon{x: false})))
    }
}


struct Carillon {
    x: bool
}


impl Synthesizer for Carillon {

    fn synthesize(&mut self, samples_played: u64, output_buffer: &mut [Sample]) {
        let mut t = samples_played as f64 / SAMPLES_PER_SECOND as f64;
        for sample in output_buffer {
            let power = (t * 440.0 * f64::consts::PI * 2.0).sin() as f32;
            *sample = Sample{left: power, right: power};
            t += 1.0 / SAMPLES_PER_SECOND as f64;
        }
    }

}


#[derive(Copy, Clone)]
struct GameState {
    position: (i32, i32),
    velocity: (i32, i32),
    ended: bool,
    success: bool
}


#[derive(Copy, Clone)]
struct Action {
    xacc: i32,
    yacc: i32
}


impl Action {
    fn from_tuple(a: (i32, i32)) -> Action {
        Action{xacc: a.0, yacc: a.1}
    }
}


fn random_start(start_positions: &Vec<(i32, i32)>) -> GameState {
    let mut rng = thread_rng();
    let rand_index: usize = rng.gen_range(0, start_positions.len());
    return GameState{
        position: start_positions[rand_index],
        velocity: (0, 0),
        ended: false,
        success: false};
}


fn get_qvalue(qvalues: &Vec<f64>, state: GameState, 
              action: Action) -> f64 {
    let pos = state.position.0 + 50*state.position.1 + 50*50*(action.xacc+1) + 
        50*50*3*(action.yacc+1 );
    return qvalues[pos as usize];
}


fn greedy_action(qvalues: &Vec<f64>, state: GameState) -> Action {
    let actions: Vec<_> = iproduct!(-1..2, -1..2)
        .map(Action::from_tuple)
        .collect();
    let mut best_action = Action{xacc:0, yacc:0};
    let mut max_qvalue = std::f64::NEG_INFINITY;
    for action in actions {
        let qvalue = get_qvalue(qvalues, state, action);
        // println!("qvalue: {}", qvalue);
        if qvalue > max_qvalue {
            max_qvalue = qvalue;
            best_action = action;
        }
    }
    return best_action;
}


fn int_tup_from_float(tup: (f64, f64)) -> (i32, i32) {
    return (tup.0 as i32, tup.1 as i32);
}


fn lookup_track(track: &Vec<TrackPoint>,
                pos: (i32, i32)) -> TrackPoint {
    let pos_ind = (pos.0 + 50*pos.1) as usize;
    track[pos_ind]
}


fn check_bounds(x: i32) -> i32 {
    std::cmp::max(std::cmp::min(x, 5), -5)
}


fn update_velocity(velocity: (i32, i32),
                   action: Action) -> (i32, i32) {
    (check_bounds(velocity.0 + action.xacc),
     check_bounds(velocity.1 + action.yacc))
}


fn next_state(track: &Vec<TrackPoint>,
              state: GameState, action: Action) -> GameState {

    // let current_position = <(f64, f64)>::from(state.velocity);// as (f32, f32);
    let mut pos: (i32, i32) = state.position;
    let new_velocity = update_velocity(state.velocity, action);
    let mut current_position = (pos.0 as f64, pos.1 as f64);
    for _ in 0..5 {
        // current_position 
        current_position = (
            current_position.0 + 0.2 * (new_velocity.0 as f64),
            current_position.1 + 0.2 * (new_velocity.1 as f64)
        );

        pos = int_tup_from_float(current_position);

        if std::cmp::max(pos.0, pos.1) >= 50 || 
           std::cmp::min(pos.0, pos.1) < 0 {
            return GameState{
                position: (0, 0),
                velocity: (0, 0),
                ended: true,
                success: false
            };
        }

        let track_value = lookup_track(track, pos);

        if track_value == End {
            return GameState{
                position: (0, 0),
                velocity: (0, 0),
                ended: true,
                success: true
            };
        }
        else if track_value == Beginning || track_value == Boundary {
            return GameState{
                position: (0, 0),
                velocity: (0, 0),
                ended: true,
                success: false
            };
        }
    }

    return GameState{
        position: pos,
        velocity: new_velocity,
        ended: false,
        success: false
    };
}


fn main() {
    run::<Game>();
}


