use std::f32::consts::PI;
use std::thread::sleep;
use std::time::{Duration, Instant};

use helpers::{
    rotate_rect, run, square_at_position, GlowColor, State, TextManager, TextType, Timings,
    SCREEN_HEIGHT, SCREEN_WIDTH,
};
use rand::rngs::ThreadRng;
use rand::Rng;
use speedy2d::color::Color as SpeedyColor;
use speedy2d::dimen::Vector2;
use speedy2d::Graphics2D;

mod helpers;

const END_OF_CUBE_OUT: f32 = 0.3;
const FAST_LINES: usize = 3;

#[derive(Clone, Copy, Debug)]
struct PositionAndRotation {
    position: Vector2<f32>,
    rotation: f32,
}

struct MyState {
    rng: ThreadRng,
    games: Vec<Vec<Set>>,
    score: u32,

    set_start_at: Option<Instant>,
    current_set_index: usize,
    current_game_index: usize,
    current_set_positions: (
        Vec<(PositionAndRotation, PositionAndRotation)>,
        Vec<(PositionAndRotation, PositionAndRotation)>,
        Vec<(PositionAndRotation, PositionAndRotation)>,
    ),

    goal_line: f32,

    column_r: f32,
    column_g: f32,
    column_b: f32,

    goal_red_cubes: Vec<PositionAndRotation>,
    goal_green_cubes: Vec<PositionAndRotation>,
    goal_blue_cubes: Vec<PositionAndRotation>,
}

impl State for MyState {
    fn on_draw(
        &mut self,
        _timings: &Timings,
        text_manager: &mut TextManager,
        graphics: &mut Graphics2D,
    ) {
        let set_duration = if self.current_game_index == 0 {
            2000
        } else if self.current_game_index < FAST_LINES {
            800
        } else {
            10
        };
        let game_no = self.current_game_index + 1;

        if self.current_game_index == self.games.len() {
            text_manager.draw_text(
                graphics,
                256,
                TextType::Glow(GlowColor::Gold),
                (SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 / 2.0),
                self.score.to_string(),
            );

            sleep(Duration::from_millis(100));
            return;
        }

        text_manager.draw_text(
            graphics,
            128,
            TextType::Glow(GlowColor::White),
            (SCREEN_WIDTH as f32 / 4.0, 150.0),
            self.score.to_string(),
        );

        text_manager.draw_text(
            graphics,
            80,
            TextType::Gray,
            (SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 - 100.0),
            format!(
                "Game n°{game_no} — Set {}/{}",
                self.current_set_index + 1,
                self.games[self.current_game_index].len()
            ),
        );

        self.draw_goals(
            graphics,
            text_manager,
            self.column_r,
            &self.goal_red_cubes,
            Color::Red,
        );
        self.draw_goals(
            graphics,
            text_manager,
            self.column_g,
            &self.goal_green_cubes,
            Color::Green,
        );
        self.draw_goals(
            graphics,
            text_manager,
            self.column_b,
            &self.goal_blue_cubes,
            Color::Blue,
        );

        let set_start_at = match self.set_start_at {
            Some(start_at) => {
                if start_at.elapsed().as_millis() > set_duration {
                    let set_value = self.games[self.current_game_index][self.current_set_index];
                    if set_value.red > self.goal_red_cubes.len() as u32 {
                        self.goal_red_cubes = self.generate_cube_positions(
                            Vector2::new(self.column_r, self.goal_line),
                            set_value.red,
                        );
                    }
                    if set_value.green > self.goal_green_cubes.len() as u32 {
                        self.goal_green_cubes = self.generate_cube_positions(
                            Vector2::new(self.column_g, self.goal_line),
                            set_value.green,
                        );
                    }
                    if set_value.blue > self.goal_blue_cubes.len() as u32 {
                        self.goal_blue_cubes = self.generate_cube_positions(
                            Vector2::new(self.column_b, self.goal_line),
                            set_value.blue,
                        );
                    }

                    self.current_set_index += 1;

                    if self.current_set_index >= self.games[self.current_game_index].len() {
                        self.score += self.goal_red_cubes.len() as u32
                            * self.goal_green_cubes.len() as u32
                            * self.goal_blue_cubes.len() as u32;

                        self.goal_red_cubes = vec![];
                        self.goal_green_cubes = vec![];
                        self.goal_blue_cubes = vec![];
                        self.current_set_index = 0;
                        self.current_game_index += 1;
                    }

                    self.set_start_at = Some(Instant::now());
                    self.prepare_set_positions_and_rotations();
                    return;
                }

                start_at
            }
            None => {
                self.current_game_index = 0;
                self.current_set_index = 0;

                self.set_start_at = Some(Instant::now());
                self.prepare_set_positions_and_rotations();

                return;
            }
        };

        let percentage_of_set = set_start_at.elapsed().as_millis() as f32 / set_duration as f32;

        if self.current_game_index < FAST_LINES {
            self.draw_cubes_going_out(
                graphics,
                percentage_of_set,
                &self.current_set_positions.0,
                Color::Red,
            );
            self.draw_cubes_going_out(
                graphics,
                percentage_of_set,
                &self.current_set_positions.1,
                Color::Green,
            );
            self.draw_cubes_going_out(
                graphics,
                percentage_of_set,
                &self.current_set_positions.2,
                Color::Blue,
            );
        }

        if percentage_of_set >= END_OF_CUBE_OUT {
            let value = self.games[self.current_game_index][self.current_set_index].red;
            text_manager.draw_text(
                graphics,
                60,
                TextType::Glow(GlowColor::Gold),
                (self.column_r, SCREEN_HEIGHT as f32 / 2.0 + 100.0),
                value.to_string(),
            );

            if value > self.goal_red_cubes.len() as u32 {
                self.goal_red_cubes = self
                    .generate_cube_positions(Vector2::new(self.column_r, self.goal_line), value);
            }
        }
        if percentage_of_set >= END_OF_CUBE_OUT + (1.0 - END_OF_CUBE_OUT) / 3.0 {
            let value = self.games[self.current_game_index][self.current_set_index].green;
            text_manager.draw_text(
                graphics,
                60,
                TextType::Glow(GlowColor::Gold),
                (self.column_g, SCREEN_HEIGHT as f32 / 2.0 + 100.0),
                value.to_string(),
            );

            if value > self.goal_green_cubes.len() as u32 {
                self.goal_green_cubes = self
                    .generate_cube_positions(Vector2::new(self.column_g, self.goal_line), value);
            }
        }
        if percentage_of_set
            >= END_OF_CUBE_OUT + (1.0 - END_OF_CUBE_OUT) / 3.0 + (1.0 - END_OF_CUBE_OUT) / 3.0
        {
            let value = self.games[self.current_game_index][self.current_set_index].blue;
            text_manager.draw_text(
                graphics,
                60,
                TextType::Glow(GlowColor::Gold),
                (self.column_b, SCREEN_HEIGHT as f32 / 2.0 + 100.0),
                value.to_string(),
            );

            if value > self.goal_blue_cubes.len() as u32 {
                self.goal_blue_cubes = self
                    .generate_cube_positions(Vector2::new(self.column_b, self.goal_line), value);
            }
        }
    }
}

impl MyState {
    fn draw_goals(
        &self,
        graphics: &mut Graphics2D,
        text_manager: &mut TextManager,
        column: f32,
        cubes: &Vec<PositionAndRotation>,
        color: Color,
    ) {
        for position_and_rotation in cubes {
            draw_cube(graphics, *position_and_rotation, color, 40.0);
        }
        text_manager.draw_text(
            graphics,
            80,
            if cubes.is_empty() {
                TextType::Gray
            } else {
                TextType::Glow(GlowColor::Gold)
            },
            (column, self.goal_line + 110.0),
            cubes.len().to_string(),
        );
    }

    fn draw_cubes_going_out(
        &self,
        graphics: &mut Graphics2D,
        percentage_of_set: f32,
        cubes: &Vec<(PositionAndRotation, PositionAndRotation)>,
        color: Color,
    ) {
        for (cube_start, cube_end) in cubes {
            let position_and_rotation = if percentage_of_set > END_OF_CUBE_OUT {
                *cube_end
            } else {
                let percentage_of_cube_out = percentage_of_set / END_OF_CUBE_OUT;

                PositionAndRotation {
                    position: if self.current_game_index > FAST_LINES {
                        cube_end.position
                    } else {
                        Vector2::new(
                            interp(
                                cube_start.position.x,
                                cube_end.position.x,
                                percentage_of_cube_out,
                            ),
                            interp(
                                cube_start.position.y,
                                cube_end.position.y,
                                percentage_of_cube_out,
                            ),
                        )
                    },
                    rotation: interp(
                        cube_start.rotation,
                        cube_end.rotation,
                        percentage_of_cube_out,
                    ),
                }
            };

            draw_cube(graphics, position_and_rotation, color, 30.0);
        }
    }

    fn prepare_set_positions_and_rotations(&mut self) {
        if self.current_game_index == self.games.len() {
            return;
        }

        // Red
        let start = self.generate_cube_positions(
            Vector2::new(SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 - 300.0),
            self.games[self.current_game_index][self.current_set_index].red,
        );
        let end = self.generate_cube_positions(
            Vector2::new(self.column_r, SCREEN_HEIGHT as f32 / 2.0),
            self.games[self.current_game_index][self.current_set_index].red,
        );
        self.current_set_positions.0 = start.into_iter().zip(end).collect();

        // Green
        let start = self.generate_cube_positions(
            Vector2::new(SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 - 300.0),
            self.games[self.current_game_index][self.current_set_index].green,
        );
        let end = self.generate_cube_positions(
            Vector2::new(self.column_g, SCREEN_HEIGHT as f32 / 2.0),
            self.games[self.current_game_index][self.current_set_index].green,
        );
        self.current_set_positions.1 = start.into_iter().zip(end).collect();

        // Blue
        let start = self.generate_cube_positions(
            Vector2::new(SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 - 300.0),
            self.games[self.current_game_index][self.current_set_index].blue,
        );
        let end = self.generate_cube_positions(
            Vector2::new(self.column_b, SCREEN_HEIGHT as f32 / 2.0),
            self.games[self.current_game_index][self.current_set_index].blue,
        );
        self.current_set_positions.2 = start.into_iter().zip(end).collect();
    }

    fn generate_cube_positions(
        &mut self,
        position: Vector2<f32>,
        number: u32,
    ) -> Vec<PositionAndRotation> {
        (0..number)
            .map(|_| {
                let random_translation = Vector2::new(
                    (self.rng.gen::<f32>() - 0.5) * 30.0,
                    (self.rng.gen::<f32>() - 0.5) * 30.0,
                );
                let random_rotation = (self.rng.gen::<f32>() - 0.5) * 2.0 * PI;
                let random_position = position + random_translation;

                PositionAndRotation {
                    position: random_position,
                    rotation: random_rotation,
                }
            })
            .collect()
    }
}

fn draw_cube(
    graphics: &mut Graphics2D,
    position_and_rotation: PositionAndRotation,
    color: Color,
    size: f32,
) {
    let PositionAndRotation { position, rotation } = position_and_rotation;

    let mut square = square_at_position(position, size * 1.1);
    rotate_rect(&mut square, position, rotation);
    graphics.draw_quad(
        square,
        match color {
            Color::Red => SpeedyColor::from_hex_rgb(0xef4444),
            Color::Green => SpeedyColor::from_hex_rgb(0x10b981),
            Color::Blue => SpeedyColor::from_hex_rgb(0x06b6d4),
        },
    );
    let mut square = square_at_position(position, size);
    rotate_rect(&mut square, position, rotation);
    graphics.draw_quad(
        square,
        match color {
            Color::Red => SpeedyColor::from_hex_rgb(0xdc2626),
            Color::Green => SpeedyColor::from_hex_rgb(0x059669),
            Color::Blue => SpeedyColor::from_hex_rgb(0x0891b2),
        },
    );
}

#[derive(Clone, Copy, Debug)]
enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Default, Clone, Copy)]
struct Set {
    blue: u32,
    green: u32,
    red: u32,
}

fn main() {
    let rng = rand::thread_rng();

    let games: Vec<Vec<Set>> = input()
        .trim()
        .lines()
        .map(|line| line.trim().split_once(": ").unwrap().1)
        .map(|line| {
            line.split("; ")
                .map(|set_as_string| {
                    let mut set = Set::default();
                    set_as_string.split(", ").for_each(|cubes| {
                        if cubes.ends_with(" blue") {
                            set.blue = cubes.trim_end_matches(" blue").parse().unwrap()
                        } else if cubes.ends_with(" red") {
                            set.red = cubes.trim_end_matches(" red").parse().unwrap()
                        } else if cubes.ends_with(" green") {
                            set.green = cubes.trim_end_matches(" green").parse().unwrap()
                        } else {
                            panic!();
                        }
                    });

                    set
                })
                .collect()
        })
        .collect();

    let goal_line = 400.0;

    let column_r = (SCREEN_WIDTH as f32 / 4.0) * 1.0;
    let column_g = (SCREEN_WIDTH as f32 / 4.0) * 2.0;
    let column_b = (SCREEN_WIDTH as f32 / 4.0) * 3.0;

    run(MyState {
        rng,
        games,
        score: 0,

        set_start_at: None,
        current_game_index: 0,
        current_set_index: 0,
        current_set_positions: (vec![], vec![], vec![]),

        goal_line,

        column_r,
        column_g,
        column_b,

        goal_red_cubes: vec![],
        goal_green_cubes: vec![],
        goal_blue_cubes: vec![],
    });
}

fn interp(start: f32, end: f32, percentage: f32) -> f32 {
    if start > end {
        start - (start - end) * percentage
    } else {
        start + (end - start) * percentage
    }
}

fn input() -> &'static str {
    "
    Game 1: 4 blue, 16 green, 2 red; 5 red, 11 blue, 16 green; 9 green, 11 blue; 10 blue, 6 green, 4 red
    Game 2: 15 green, 20 red, 8 blue; 12 green, 7 red; 10 green, 2 blue, 15 red; 13 blue, 15 red
    Game 3: 8 red, 2 blue; 3 green, 10 blue, 10 red; 7 green, 4 blue, 7 red; 8 red, 6 green, 13 blue; 4 green, 3 blue, 10 red; 7 blue, 7 green, 5 red
    Game 4: 13 green, 14 blue, 9 red; 6 green, 14 red, 18 blue; 9 red, 11 green, 3 blue; 11 green, 10 red, 14 blue; 17 blue, 3 red, 4 green; 17 blue, 1 red, 9 green
    Game 5: 2 green, 1 red; 8 blue, 2 green, 6 red; 5 blue, 9 red, 2 green; 3 green, 8 red, 6 blue; 6 blue, 5 red
    Game 6: 3 green, 7 blue, 5 red; 3 green, 6 red; 11 blue, 6 red, 1 green
    Game 7: 8 red, 4 green, 11 blue; 12 blue, 1 green, 5 red; 6 red, 1 green, 5 blue; 12 blue, 2 green, 2 red; 4 blue, 4 green, 3 red; 9 blue, 4 green, 8 red
    Game 8: 1 red, 4 green; 6 red, 1 green; 10 red; 1 blue, 2 green; 4 green, 3 red; 1 blue, 8 red
    Game 9: 9 blue, 13 green, 1 red; 10 green, 4 blue, 4 red; 3 red, 4 blue, 14 green; 13 blue, 1 red, 12 green
    Game 10: 2 blue, 16 red, 2 green; 1 green, 16 red, 6 blue; 9 red, 3 green; 1 green, 2 blue, 8 red; 8 red, 6 blue, 3 green
    Game 11: 7 green, 11 red, 12 blue; 3 blue, 6 green, 6 red; 10 blue, 13 green; 1 red, 13 green, 9 blue; 2 blue, 2 red, 13 green; 2 red, 3 blue, 15 green
    Game 12: 3 green, 2 red, 2 blue; 7 green, 5 blue; 1 blue, 1 red, 3 green
    Game 13: 2 green, 2 red, 3 blue; 3 blue, 3 red, 3 green; 3 green, 2 red; 2 blue, 3 red, 3 green; 2 green, 3 red, 1 blue
    Game 14: 4 green, 9 red; 11 green, 10 red, 12 blue; 6 red, 3 green, 12 blue; 5 green, 4 red, 4 blue; 18 blue, 7 red, 11 green; 16 blue, 4 red, 10 green
    Game 15: 5 green, 2 red, 9 blue; 18 green, 6 red, 20 blue; 11 blue, 12 green, 11 red; 9 red, 17 blue, 16 green; 7 green, 1 red, 9 blue
    Game 16: 9 blue, 11 green; 8 green, 2 blue; 1 red, 6 green, 4 blue
    Game 17: 2 red, 2 green, 2 blue; 7 blue, 4 green, 3 red; 2 red, 8 blue, 1 green; 2 red, 6 blue, 2 green; 4 blue, 3 red; 4 green, 5 red, 6 blue
    Game 18: 6 green, 7 red; 3 blue, 6 green, 1 red; 6 red, 3 blue, 5 green
    Game 19: 6 red, 4 green, 5 blue; 2 red, 4 blue, 13 green; 1 green, 1 blue, 2 red; 4 green
    Game 20: 7 red, 17 blue, 6 green; 3 blue, 6 green, 8 red; 7 blue, 6 red, 1 green; 3 green; 8 red, 7 green, 14 blue
    Game 21: 5 red, 3 blue, 7 green; 1 blue, 2 red, 5 green; 2 blue, 8 green, 3 red; 3 blue, 8 red, 4 green; 5 red, 1 blue, 3 green
    Game 22: 2 red, 6 green, 1 blue; 3 red, 3 green, 1 blue; 2 green, 7 red, 2 blue; 5 green, 1 red
    Game 23: 2 red, 16 green, 1 blue; 1 red, 12 green, 3 blue; 12 green, 1 blue, 3 red
    Game 24: 7 red, 1 blue, 12 green; 2 red, 19 green, 3 blue; 19 green, 1 blue, 12 red; 6 green, 16 red, 5 blue; 11 red, 4 blue, 12 green
    Game 25: 2 blue, 3 red, 8 green; 4 blue, 2 red, 9 green; 2 red, 7 blue
    Game 26: 17 red, 8 blue, 3 green; 3 green, 13 red, 4 blue; 20 red, 1 green, 6 blue; 7 blue, 2 red, 2 green; 20 red, 8 blue; 2 green, 16 red, 8 blue
    Game 27: 3 blue, 17 green, 19 red; 16 green, 5 red, 6 blue; 17 green, 16 red, 4 blue
    Game 28: 1 green, 7 red, 1 blue; 8 green, 12 red, 1 blue; 1 blue, 9 red, 1 green
    Game 29: 3 green, 3 blue, 2 red; 3 green, 2 red, 1 blue; 3 green, 2 red, 3 blue; 3 blue, 3 red, 4 green
    Game 30: 3 red, 8 blue, 3 green; 1 green, 1 red; 17 green, 17 blue; 19 green, 15 blue, 1 red; 1 green, 2 red, 16 blue
    Game 31: 11 green, 11 blue, 14 red; 6 blue, 15 green, 2 red; 11 blue, 19 green, 2 red
    Game 32: 9 red, 2 green; 7 green, 4 blue, 2 red; 6 red, 5 green, 1 blue; 4 red, 4 blue, 1 green; 8 red, 6 green
    Game 33: 6 blue, 16 red, 9 green; 5 red, 7 blue, 13 green; 1 green, 9 blue, 1 red; 4 green, 9 blue, 17 red; 2 green, 10 red, 13 blue; 9 red, 1 blue, 14 green
    Game 34: 2 red, 2 green, 4 blue; 3 blue, 2 green; 1 green, 1 red, 2 blue; 1 red, 3 blue, 3 green; 2 green, 8 blue, 2 red; 3 blue, 1 red
    Game 35: 4 red, 14 blue, 2 green; 1 green, 15 blue, 1 red; 1 blue, 2 red, 1 green
    Game 36: 4 blue, 1 red, 2 green; 2 green, 15 blue, 8 red; 7 blue, 1 red; 7 red, 1 green, 1 blue
    Game 37: 2 blue, 1 green, 5 red; 2 blue, 2 green, 4 red; 2 blue, 5 red, 8 green; 3 green, 2 blue, 1 red; 1 red, 1 blue, 5 green; 2 blue, 1 red, 8 green
    Game 38: 2 blue, 4 green, 11 red; 7 green, 6 red, 2 blue; 1 green, 3 red, 1 blue; 4 blue, 4 green, 4 red; 2 red, 5 blue, 2 green
    Game 39: 7 green, 7 blue, 2 red; 11 blue, 4 green, 8 red; 10 red, 4 green, 1 blue; 8 green, 9 blue; 9 green, 4 red; 1 green, 8 blue
    Game 40: 1 green, 13 blue; 6 blue, 7 red; 8 red; 1 green, 13 blue, 3 red; 1 green, 16 red, 13 blue; 14 blue, 14 red, 1 green
    Game 41: 5 green, 2 blue, 10 red; 4 green, 2 blue, 5 red; 6 green, 9 red, 1 blue; 4 red, 1 blue; 1 red, 3 green, 2 blue; 3 red
    Game 42: 17 green, 11 blue, 11 red; 5 blue, 11 green, 9 red; 10 blue, 13 red, 4 green; 8 green, 4 blue, 15 red
    Game 43: 1 red, 3 blue; 1 green, 3 blue, 1 red; 2 blue, 1 green; 2 green, 1 blue; 1 red, 3 blue
    Game 44: 7 green, 5 red, 1 blue; 6 green, 1 blue, 5 red; 2 blue, 6 green; 3 green, 2 red; 4 green; 6 red
    Game 45: 16 red, 14 blue, 19 green; 1 red, 5 green, 6 blue; 16 blue, 2 green, 1 red; 15 green, 6 red, 16 blue
    Game 46: 8 blue, 2 green; 4 red, 3 green, 6 blue; 1 green, 8 blue, 3 red; 3 green, 12 blue, 1 red
    Game 47: 9 green, 3 blue; 1 green, 1 blue; 4 blue, 9 green, 6 red; 8 green, 4 blue, 6 red; 6 red, 12 green, 1 blue; 4 blue, 7 green
    Game 48: 11 green, 4 blue, 1 red; 11 blue, 8 red, 9 green; 4 blue, 3 red, 7 green; 10 blue, 2 green, 9 red; 8 green, 2 blue, 2 red
    Game 49: 8 green, 1 blue, 5 red; 1 green, 1 blue; 3 green, 4 red, 2 blue; 1 blue, 7 green, 1 red; 1 blue, 7 green, 3 red; 5 red, 5 green
    Game 50: 2 green, 2 red, 4 blue; 8 blue, 2 green, 7 red; 4 blue, 5 red; 9 red, 4 blue; 5 blue, 9 red; 2 green, 8 red, 6 blue
    Game 51: 6 green, 1 red, 2 blue; 2 red, 4 blue, 6 green; 9 blue, 4 green
    Game 52: 7 green, 3 red, 12 blue; 8 blue, 9 red, 5 green; 2 blue, 10 green, 8 red; 12 red, 5 green, 3 blue; 8 red, 8 green, 12 blue; 2 green
    Game 53: 2 green, 9 blue, 5 red; 6 red, 3 green; 5 red, 2 green
    Game 54: 9 red, 13 blue; 1 green, 9 red, 16 blue; 12 red, 1 blue, 4 green
    Game 55: 1 red, 2 blue, 3 green; 1 blue; 1 red, 5 blue, 3 green; 1 blue, 3 green; 5 blue
    Game 56: 1 green, 4 red, 1 blue; 1 blue, 2 red, 13 green; 5 blue, 4 red; 13 green, 3 red, 3 blue
    Game 57: 13 blue, 2 red, 7 green; 3 green, 4 red, 14 blue; 3 red, 3 green, 3 blue; 7 blue, 5 green, 1 red
    Game 58: 6 red; 1 blue, 4 red, 2 green; 3 green, 1 blue; 7 green, 1 red; 6 red, 13 green, 1 blue; 3 red, 13 green, 1 blue
    Game 59: 5 green, 10 red, 8 blue; 7 red, 3 green, 2 blue; 6 green, 3 red, 6 blue
    Game 60: 2 green, 5 red, 15 blue; 2 green, 9 blue; 9 blue, 8 green, 3 red; 2 green, 6 red, 2 blue
    Game 61: 8 blue, 3 green, 4 red; 1 red, 10 blue, 1 green; 4 red, 5 green, 3 blue; 3 red, 8 blue, 5 green
    Game 62: 19 blue, 3 red, 14 green; 1 green, 7 blue, 1 red; 15 red, 20 blue, 6 green; 8 red, 4 green, 14 blue
    Game 63: 13 red, 1 blue; 18 red, 4 green; 6 green, 9 red, 1 blue; 7 green, 1 blue, 9 red; 5 red, 1 blue, 4 green; 5 green, 1 blue, 17 red
    Game 64: 2 green, 1 blue, 5 red; 2 red, 5 green; 6 red, 4 green
    Game 65: 1 blue, 7 green, 1 red; 7 red, 1 green; 1 blue, 3 green, 3 red; 7 red, 3 green; 3 green, 7 red; 1 blue, 4 green
    Game 66: 7 green, 6 blue, 8 red; 4 green, 9 red, 3 blue; 6 green, 4 blue; 5 blue, 2 green; 6 red, 4 green, 2 blue
    Game 67: 10 blue, 17 green, 17 red; 11 red, 9 blue, 9 green; 9 blue, 19 red, 5 green; 5 red, 3 blue, 20 green; 11 red, 1 blue, 7 green
    Game 68: 9 green, 4 red, 5 blue; 11 blue, 9 green, 2 red; 11 blue, 2 red, 6 green; 2 green, 6 red, 3 blue; 1 blue, 6 green, 4 red
    Game 69: 3 red, 15 blue, 1 green; 4 red, 14 blue, 2 green; 4 red, 18 blue, 4 green
    Game 70: 3 red, 8 green; 2 red, 6 green; 4 red, 2 blue, 2 green; 8 red, 1 green, 2 blue; 6 red, 3 blue, 4 green; 13 green, 8 red
    Game 71: 3 green, 17 red; 2 red, 3 green; 2 green, 8 red, 1 blue; 11 red, 4 blue; 3 green, 11 red, 3 blue
    Game 72: 1 red, 17 blue, 8 green; 2 red, 11 blue, 16 green; 3 red, 16 blue, 1 green; 2 red, 3 green, 10 blue
    Game 73: 1 blue, 10 green, 8 red; 19 green, 10 red, 5 blue; 3 green, 13 red, 8 blue; 12 green, 4 blue; 2 green, 10 blue, 12 red
    Game 74: 17 blue, 7 red, 10 green; 16 blue, 5 red; 9 blue, 7 green, 2 red; 10 red, 4 green, 14 blue
    Game 75: 10 green, 5 blue, 4 red; 7 red, 10 blue, 7 green; 7 blue, 9 green, 2 red
    Game 76: 13 green, 16 red, 20 blue; 4 red, 14 blue, 5 green; 12 red, 1 blue, 8 green
    Game 77: 4 red, 2 green; 8 blue, 3 green, 2 red; 5 blue, 7 green, 3 red
    Game 78: 12 green, 8 red, 8 blue; 10 green, 9 red, 10 blue; 16 blue, 1 red, 17 green; 4 red, 15 green, 13 blue
    Game 79: 4 green, 2 red; 15 red, 3 blue; 15 red, 5 green
    Game 80: 4 blue, 1 green, 13 red; 13 red, 1 blue, 5 green; 5 blue, 9 red; 3 blue, 3 green; 1 red; 3 red, 7 green, 6 blue
    Game 81: 10 red, 3 green, 4 blue; 2 red, 5 green, 16 blue; 3 green, 1 blue; 9 blue, 2 green, 12 red
    Game 82: 1 green, 9 blue, 1 red; 10 blue, 1 red, 1 green; 1 green, 7 blue; 8 blue
    Game 83: 1 blue, 5 red; 2 blue, 3 red; 1 green, 2 blue, 1 red; 2 red, 1 blue, 1 green; 1 green, 1 blue; 2 red, 1 green
    Game 84: 5 red, 14 blue, 2 green; 6 blue, 5 red, 8 green; 12 green, 3 blue, 5 red; 2 red, 10 green; 9 green, 14 blue
    Game 85: 2 blue, 2 red; 14 red, 6 green, 5 blue; 5 green, 4 blue, 6 red; 8 red, 5 blue, 6 green
    Game 86: 1 blue, 10 red; 4 red; 9 blue, 18 red, 3 green; 1 green, 1 blue, 7 red; 3 green, 8 red, 9 blue; 14 red, 2 green, 4 blue
    Game 87: 1 green, 11 red, 8 blue; 1 green, 11 red, 2 blue; 7 red, 4 blue; 6 blue, 1 red, 2 green; 13 blue, 2 green; 6 blue, 12 red, 3 green
    Game 88: 2 blue, 4 red, 8 green; 4 blue, 7 red; 3 red, 10 green, 4 blue; 9 green, 3 blue, 5 red; 4 red, 6 blue, 3 green
    Game 89: 6 red, 10 green; 15 green, 15 red, 10 blue; 15 red, 1 green, 4 blue; 13 red, 6 blue, 4 green
    Game 90: 17 green, 2 red, 1 blue; 6 green; 1 blue, 1 green; 1 blue, 16 green, 3 red; 14 green, 1 red
    Game 91: 3 blue, 8 green; 3 green, 7 red, 9 blue; 12 blue; 9 red, 7 blue, 4 green; 1 green, 7 red, 1 blue
    Game 92: 11 blue, 9 red, 12 green; 1 blue, 14 red, 6 green; 9 green, 6 red, 6 blue
    Game 93: 1 red, 2 blue; 3 blue, 6 green; 1 red, 4 green, 3 blue
    Game 94: 3 green, 3 blue; 1 red, 3 blue, 9 green; 3 blue, 10 green, 3 red; 10 green, 6 blue, 2 red; 9 blue, 14 green, 2 red; 1 red, 4 blue, 1 green
    Game 95: 7 blue, 10 green; 3 blue, 5 green, 2 red; 4 blue, 10 green, 12 red; 6 green, 2 red, 6 blue
    Game 96: 2 blue, 18 green, 8 red; 13 green, 3 blue, 3 red; 3 blue, 15 red, 8 green; 13 green, 10 red, 2 blue
    Game 97: 14 blue, 2 red; 15 blue, 1 green, 2 red; 3 red, 6 blue, 1 green; 1 green, 14 blue, 4 red
    Game 98: 4 blue, 9 red; 10 red, 1 green, 11 blue; 7 blue, 1 red; 1 red, 6 blue, 1 green
    Game 99: 7 red, 6 green, 2 blue; 8 red; 16 green, 7 red, 4 blue
    Game 100: 1 red, 1 green, 9 blue; 6 blue, 4 green, 3 red; 4 red, 2 green; 3 green, 2 red, 11 blue; 6 green, 5 blue, 1 red
    
    "
}
