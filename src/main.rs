// Importing local settings for Rusty Snake
mod settings;

use ggez::{Context, ContextBuilder, GameResult, event, graphics, timer};
use ggez::event::{KeyCode, KeyMods};
use rand::Rng;
use std::cmp;
use settings::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(PartialEq)]
// Checking state for Gameplay
// This could be single-mode, bot, or competitive
enum AppState {
    Menu,
    Playing,    
    Competitive, 
    GameOver,
}

struct MainState {
    snake: Vec<Point>,
    snake_dir: Point,
    score: i32,
    auto_play: bool,

    player_snake: Vec<Point>,
    player_snake_dir: Point,
    bot_snake: Vec<Point>,
    player_score: i32,
    bot_score: i32,

    food: Point,
    block: Point,
    timer: f32,
    game_over: bool,
    app_state: AppState,
    fullscreen: bool,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let mut s = MainState {
            snake: vec![],
            snake_dir: Point { x: 1, y: 0 },
            score: 0,
            auto_play: false,

            player_snake: vec![],
            player_snake_dir: Point { x: 1, y: 0 },
            bot_snake: vec![],
            player_score: 0,
            bot_score: 0,

            food: Point { x: 0, y: 0 },
            block: Point { x: 0, y: 0 },
            timer: 0.0,
            game_over: false,
            app_state: AppState::Menu,
            fullscreen: false,
        };
        s.reset();
        Ok(s)
    }

    fn reset(&mut self) {
        self.game_over = false;
        self.timer = 0.0;
        match self.app_state {
            AppState::Competitive => {
                self.player_snake = vec![Point { x: GRID_WIDTH / 4, y: GRID_HEIGHT / 2 }];
                self.player_snake_dir = Point { x: 1, y: 0 };
                self.bot_snake = vec![Point { x: 3 * GRID_WIDTH / 4, y: GRID_HEIGHT / 2 }];
                self.player_score = 0;
                self.bot_score = 0;
                let obstacles: Vec<Point> = self.player_snake
                    .iter()
                    .chain(self.bot_snake.iter())
                    .copied()
                    .collect();
                self.food = Self::spawn_food(&obstacles);
                self.block = Self::spawn_block(&obstacles, self.food);
            },
            _ => {
                self.snake = vec![Point { x: GRID_WIDTH / 2, y: GRID_HEIGHT / 2 }];
                self.snake_dir = Point { x: 1, y: 0 };
                self.score = 0;
                self.food = Self::spawn_food(&self.snake);
                self.block = Self::spawn_block(&self.snake, self.food);
            }
        }
    }

    fn spawn_food(obstacles: &Vec<Point>) -> Point {
        let mut rng = rand::thread_rng();
        loop {
            let point = Point {
                x: rng.gen_range(0..GRID_WIDTH),
                y: rng.gen_range(0..GRID_HEIGHT),
            };
            if !obstacles.contains(&point) {
                return point;
            }
        }
    }

    fn spawn_block(obstacles: &Vec<Point>, food: Point) -> Point {
        let mut rng = rand::thread_rng();
        loop {
            let point = Point {
                x: rng.gen_range(0..GRID_WIDTH),
                y: rng.gen_range(0..GRID_HEIGHT),
            };
            if !obstacles.contains(&point) && point != food {
                return point;
            }
        }
    }

    fn choose_move(&self) -> Option<Point> {
        let head = self.snake[0];
        let possible_moves = vec![
            ("UP", Point { x: head.x, y: head.y - 1 }),
            ("DOWN", Point { x: head.x, y: head.y + 1 }),
            ("LEFT", Point { x: head.x - 1, y: head.y }),
            ("RIGHT", Point { x: head.x + 1, y: head.y }),
        ];
        let mut safe_moves = vec![];
        for (_name, p) in possible_moves {
            if p.x < 0 || p.x >= GRID_WIDTH || p.y < 0 || p.y >= GRID_HEIGHT {
                continue;
            }
            if self.snake.contains(&p) {
                continue;
            }
            safe_moves.push(p);
        }
        if safe_moves.is_empty() {
            return None;
        }
        safe_moves.sort_by_key(|p| (p.x - self.food.x).abs() + (p.y - self.food.y).abs());
        Some(safe_moves[0])
    }

    fn choose_move_for_snake(snake: &Vec<Point>, obstacles: &Vec<Point>, food: Point) -> Option<Point> {
        let head = snake[0];
        let possible_moves = vec![
            ("UP", Point { x: head.x, y: head.y - 1 }),
            ("DOWN", Point { x: head.x, y: head.y + 1 }),
            ("LEFT", Point { x: head.x - 1, y: head.y }),
            ("RIGHT", Point { x: head.x + 1, y: head.y }),
        ];
        let mut safe_moves = vec![];
        for (_name, p) in possible_moves {
            if p.x < 0 || p.x >= GRID_WIDTH || p.y < 0 || p.y >= GRID_HEIGHT {
                continue;
            }
            if snake.contains(&p) {
                continue;
            }
            if obstacles.contains(&p) {
                continue;
            }
            safe_moves.push(p);
        }
        if safe_moves.is_empty() {
            return None;
        }
        safe_moves.sort_by_key(|p| (p.x - food.x).abs() + (p.y - food.y).abs());
        Some(safe_moves[0])
    }

    fn update_single(&mut self) {
        let new_head = if self.auto_play {
            self.choose_move()
        } else {
            let head = self.snake[0];
            Some(Point { x: head.x + self.snake_dir.x, y: head.y + self.snake_dir.y })
        };

        if let Some(new_head) = new_head {
            if new_head.x < 0 || new_head.x >= GRID_WIDTH || new_head.y < 0 || new_head.y >= GRID_HEIGHT || self.snake.contains(&new_head) {
                self.game_over = true;
                return;
            }
            self.snake.insert(0, new_head);
            if new_head == self.food {
                self.score += 10;
                self.food = Self::spawn_food(&self.snake);
            } else {
                self.snake.pop();
            }
            if new_head == self.block {
                let new_length = cmp::max(1, self.snake.len() / 2);
                self.snake.truncate(new_length);
                self.block = Self::spawn_block(&self.snake, self.food);
                self.score = cmp::max(0, self.score - 5);
            }
        } else {
            self.game_over = true;
        }
    }

    fn update_competitive(&mut self) {
        let mut player_dead = false;
        let mut bot_dead = false;

        if let Some(player_head) = self.player_snake.first().copied() {
            let new_head = Point { x: player_head.x + self.player_snake_dir.x, y: player_head.y + self.player_snake_dir.y };
            if new_head.x < 0 || new_head.x >= GRID_WIDTH || new_head.y < 0 || new_head.y >= GRID_HEIGHT {
                player_dead = true;
            }
            if self.player_snake.contains(&new_head) {
                player_dead = true;
            }
            if self.bot_snake.contains(&new_head) {
                player_dead = true;
            }
            if !player_dead {
                self.player_snake.insert(0, new_head);
                if new_head == self.food {
                    self.player_score += 10;
                    let obstacles: Vec<Point> = self.player_snake
                        .iter()
                        .chain(self.bot_snake.iter())
                        .copied()
                        .collect();
                    self.food = Self::spawn_food(&obstacles);
                    self.block = Self::spawn_block(&obstacles, self.food);
                } else {
                    self.player_snake.pop();
                }
                if new_head == self.block {
                    let new_length = cmp::max(1, self.player_snake.len() / 2);
                    self.player_snake.truncate(new_length);
                    let obstacles: Vec<Point> = self.player_snake
                        .iter()
                        .chain(self.bot_snake.iter())
                        .copied()
                        .collect();
                    self.block = Self::spawn_block(&obstacles, self.food);
                    self.player_score = cmp::max(0, self.player_score - 5);
                }
            }
        }

        let obstacles: Vec<Point> = self.bot_snake
            .iter()
            .chain(self.player_snake.iter())
            .copied()
            .collect();
        let bot_new_head = Self::choose_move_for_snake(&self.bot_snake, &obstacles, self.food);
        if let Some(new_head) = bot_new_head {
            if new_head.x < 0 || new_head.x >= GRID_WIDTH || new_head.y < 0 || new_head.y >= GRID_HEIGHT {
                bot_dead = true;
            }
            if self.bot_snake.contains(&new_head) {
                bot_dead = true;
            }
            if self.player_snake.contains(&new_head) {
                bot_dead = true;
            }
            if !bot_dead {
                self.bot_snake.insert(0, new_head);
                if new_head == self.food {
                    self.bot_score += 10;
                    let obs: Vec<Point> = self.player_snake
                        .iter()
                        .chain(self.bot_snake.iter())
                        .copied()
                        .collect();
                    self.food = Self::spawn_food(&obs);
                    self.block = Self::spawn_block(&obs, self.food);
                } else {
                    self.bot_snake.pop();
                }
                if new_head == self.block {
                    let new_length = cmp::max(1, self.bot_snake.len() / 2);
                    self.bot_snake.truncate(new_length);
                    let obs: Vec<Point> = self.player_snake
                        .iter()
                        .chain(self.bot_snake.iter())
                        .copied()
                        .collect();
                    self.block = Self::spawn_block(&obs, self.food);
                    self.bot_score = cmp::max(0, self.bot_score - 5);
                }
            }
        } else {
            bot_dead = true;
        }

        if player_dead || bot_dead {
            self.game_over = true;
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        match self.app_state {
            AppState::Playing => {
                let dt = timer::delta(ctx).as_secs_f32();
                self.timer += dt;
                if self.timer >= MOVE_INTERVAL {
                    self.timer -= MOVE_INTERVAL;
                    self.update_single();
                }
                if self.game_over {
                    self.app_state = AppState::GameOver;
                }
            },
            AppState::Competitive => {
                let dt = timer::delta(ctx).as_secs_f32();
                self.timer += dt;
                if self.timer >= MOVE_INTERVAL {
                    self.timer -= MOVE_INTERVAL;
                    self.update_competitive();
                }
                if self.game_over {
                    self.app_state = AppState::GameOver;
                }
            },
            _ => {},
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        use graphics::{Color, DrawMode, DrawParam, Mesh, Rect, Text};
        graphics::clear(ctx, Color::BLACK);
        let (screen_width, screen_height) = graphics::drawable_size(ctx);
        let cell_size = (screen_width / GRID_WIDTH as f32).min(screen_height / GRID_HEIGHT as f32);
        let grid_pixel_width = cell_size * GRID_WIDTH as f32;
        let grid_pixel_height = cell_size * GRID_HEIGHT as f32;
        let offset_x = (screen_width - grid_pixel_width) / 2.0;
        let offset_y = (screen_height - grid_pixel_height) / 2.0;

        match self.app_state {
            AppState::Menu => {
                let menu_text = Text::new("Self-Playing Snake\n\nPress 1 for Manual Play\nPress 2 for Self-Play\nPress 3 for Competitive Mode\n\nPress F11 to toggle Full Screen");
                let dest_point = ggez::mint::Point2 { x: screen_width / 4.0, y: screen_height / 4.0 };
                graphics::draw(ctx, &menu_text, (dest_point, Color::WHITE))?;
            },
            AppState::Playing => {
                let food_rect = Rect::new(offset_x + self.food.x as f32 * cell_size, offset_y + self.food.y as f32 * cell_size, cell_size, cell_size);
                let food_mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), food_rect, Color::GREEN)?;
                graphics::draw(ctx, &food_mesh, DrawParam::default())?;
                let block_rect = Rect::new(offset_x + self.block.x as f32 * cell_size, offset_y + self.block.y as f32 * cell_size, cell_size, cell_size);
                let block_color = Color::new(1.0, 0.65, 0.0, 1.0);
                let block_mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), block_rect, block_color)?;
                graphics::draw(ctx, &block_mesh, DrawParam::default())?;
                for segment in &self.snake {
                    let seg_rect = Rect::new(offset_x + segment.x as f32 * cell_size, offset_y + segment.y as f32 * cell_size, cell_size, cell_size);
                    let seg_mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), seg_rect, Color::WHITE)?;
                    graphics::draw(ctx, &seg_mesh, DrawParam::default())?;
                }
                let score_text = Text::new(format!("Score: {}", self.score));
                graphics::draw(ctx, &score_text, (ggez::mint::Point2 { x: 5.0, y: 5.0 }, Color::BLUE))?;
            },
            AppState::Competitive => {
                let food_rect = Rect::new(offset_x + self.food.x as f32 * cell_size, offset_y + self.food.y as f32 * cell_size, cell_size, cell_size);
                let food_mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), food_rect, Color::GREEN)?;
                graphics::draw(ctx, &food_mesh, DrawParam::default())?;
                let block_rect = Rect::new(offset_x + self.block.x as f32 * cell_size, offset_y + self.block.y as f32 * cell_size, cell_size, cell_size);
                let block_color = Color::new(1.0, 0.65, 0.0, 1.0);
                let block_mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), block_rect, block_color)?;
                graphics::draw(ctx, &block_mesh, DrawParam::default())?;
                for segment in &self.player_snake {
                    let seg_rect = Rect::new(offset_x + segment.x as f32 * cell_size, offset_y + segment.y as f32 * cell_size, cell_size, cell_size);
                    let seg_mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), seg_rect, Color::WHITE)?;
                    graphics::draw(ctx, &seg_mesh, DrawParam::default())?;
                }
                for segment in &self.bot_snake {
                    let seg_rect = Rect::new(offset_x + segment.x as f32 * cell_size, offset_y + segment.y as f32 * cell_size, cell_size, cell_size);
                    let seg_mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), seg_rect, Color::YELLOW)?;
                    graphics::draw(ctx, &seg_mesh, DrawParam::default())?;
                }
                let score_text = Text::new(format!("Player: {}   Bot: {}", self.player_score, self.bot_score));
                graphics::draw(ctx, &score_text, (ggez::mint::Point2 { x: 5.0, y: 5.0 }, Color::BLUE))?;
            },
            AppState::GameOver => {
                let game_over_text = if self.app_state == AppState::GameOver {
                    if self.player_snake.is_empty() && self.bot_snake.is_empty() {
                        Text::new("Game Over! It's a tie!\nPress Y to Play Again\nPress N to Quit\n\nPress F11 to toggle Full Screen")
                    } else if self.player_snake.is_empty() {
                        Text::new("Game Over! Bot wins!\nPress Y to Play Again\nPress N to Quit\n\nPress F11 to toggle Full Screen")
                    } else if self.bot_snake.is_empty() {
                        Text::new("Game Over! You win!\nPress Y to Play Again\nPress N to Quit\n\nPress F11 to toggle Full Screen")
                    } else {
                        Text::new(format!("Game Over! Final Score: {}\nPress Y to Play Again\nPress N to Quit\n\nPress F11 to toggle Full Screen", self.score))
                    }
                } else {
                    Text::new(format!("Game Over! Final Score: {}\nPress Y to Play Again\nPress N to Quit\n\nPress F11 to toggle Full Screen", self.score))
                };
                let dest_point = ggez::mint::Point2 { x: screen_width / 4.0, y: screen_height / 4.0 };
                graphics::draw(ctx, &game_over_text, (dest_point, Color::RED))?;
            },
        }
        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods, _repeat: bool) {
        if keycode == KeyCode::F11 {
            self.fullscreen = !self.fullscreen;
            let new_mode = if self.fullscreen {
                ggez::conf::WindowMode::default().fullscreen_type(ggez::conf::FullscreenType::Desktop)
            } else {
                ggez::conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT).resizable(true)
            };
            graphics::set_mode(ctx, new_mode).unwrap();
            return;
        }
        match self.app_state {
            AppState::Menu => {
                match keycode {
                    KeyCode::Key1 => {
                        self.auto_play = false;
                        self.app_state = AppState::Playing;
                        self.reset();
                    },
                    KeyCode::Key2 => {
                        self.auto_play = true;
                        self.app_state = AppState::Playing;
                        self.reset();
                    },
                    KeyCode::Key3 => {
                        self.app_state = AppState::Competitive;
                        self.reset();
                    },
                    _ => {},
                }
            },
            AppState::Playing => {
                if !self.auto_play {
                    match keycode {
                        KeyCode::Up => {
                            if self.snake_dir.y != 1 {
                                self.snake_dir = Point { x: 0, y: -1 };
                            }
                        },
                        KeyCode::Down => {
                            if self.snake_dir.y != -1 {
                                self.snake_dir = Point { x: 0, y: 1 };
                            }
                        },
                        KeyCode::Left => {
                            if self.snake_dir.x != 1 {
                                self.snake_dir = Point { x: -1, y: 0 };
                            }
                        },
                        KeyCode::Right => {
                            if self.snake_dir.x != -1 {
                                self.snake_dir = Point { x: 1, y: 0 };
                            }
                        },
                        _ => {},
                    }
                }
            },
            AppState::Competitive => {
                match keycode {
                    KeyCode::Up => {
                        if self.player_snake_dir.y != 1 {
                            self.player_snake_dir = Point { x: 0, y: -1 };
                        }
                    },
                    KeyCode::Down => {
                        if self.player_snake_dir.y != -1 {
                            self.player_snake_dir = Point { x: 0, y: 1 };
                        }
                    },
                    KeyCode::Left => {
                        if self.player_snake_dir.x != 1 {
                            self.player_snake_dir = Point { x: -1, y: 0 };
                        }
                    },
                    KeyCode::Right => {
                        if self.player_snake_dir.x != -1 {
                            self.player_snake_dir = Point { x: 1, y: 0 };
                        }
                    },
                    _ => {},
                }
            },
            AppState::GameOver => {
                match keycode {
                    KeyCode::Y => {
                        self.reset();
                        self.app_state = AppState::Menu;
                    },
                    KeyCode::N => {
                        ggez::event::quit(ctx);
                    },
                    _ => {},
                }
            },
        }
    }
}

pub fn main() -> GameResult {
    let (mut ctx, event_loop) = ContextBuilder::new("Self-Playing Snake", "Author")
        .window_setup(ggez::conf::WindowSetup::default().title("Self-Playing Snake"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT).resizable(true))
        .build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
