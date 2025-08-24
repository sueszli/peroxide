mod audio;
mod dom;
mod p2p;
mod physics;
mod utils;
mod view;

use std::{cell::RefCell, rc::Rc};

use js_sys::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;
use web_sys::*;

use physics::*;
use utils::combinatorics::Thrush;

const PARTICLE_COUNT_WALL: usize = 8;
const PARTICLE_COUNT_PADDLE: usize = 12;
const PARTICLE_LIFETIME: f64 = 60.0;
const GRAVITY: f64 = 0.1;

const PONG_HTML: &str = r#"
    <div id="game-container">
        <canvas id="game" width="800" height="600"></canvas>
        <div id="score-display">
            <div id="player1-score">0</div>
            <div id="player2-score">0</div>
        </div>
    </div>
    <style>
        #game-container {
            margin-top: 5rem;
            display: flex;
            flex-direction: column;
            align-items: center;
            background: #000;
            padding: 1rem;
        }
        
        #game {
            border: 3px solid #fff;
            background: #000;
            image-rendering: pixelated;
            image-rendering: -moz-crisp-edges;
            image-rendering: crisp-edges;
            max-width: 90vw;
            max-height: 70vh;
            width: auto;
            height: auto;
        }
        
        #score-display {
            display: flex;
            justify-content: space-between;
            width: 100%;
            max-width: 800px;
            margin-top: 1rem;
            font-family: 'Lucida Console', monospace;
            color: #fff;
            font-size: 2rem;
            font-weight: bold;
        }
        
    </style>
"#;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: f64,
    max_life: f64,
}

impl Particle {
    fn new(x: f64, y: f64) -> Self {
        let angle = js_sys::Math::random() * 2.0 * std::f64::consts::PI;
        let speed = 2.0 + js_sys::Math::random() * 3.0;
        Self {
            x,
            y,
            vx: angle.cos() * speed,
            vy: angle.sin() * speed,
            life: PARTICLE_LIFETIME,
            max_life: PARTICLE_LIFETIME,
        }
    }

    fn update(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
        self.vy += GRAVITY;
        self.life -= 1.0;
    }

    fn is_alive(&self) -> bool {
        self.life > 0.0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameState {
    ball_x: f64,
    ball_y: f64,
    ball_vx: f64,
    ball_vy: f64,
    player_1_y: f64,
    player_2_y: f64,
    score_1: u32,
    score_2: u32,
    particles: Vec<Particle>,
    shake_x: f64,
    shake_y: f64,
    shake_intensity: f64,
    canvas_width: f64,
    canvas_height: f64,
}

impl GameState {
    fn new() -> Self {
        Self {
            ball_x: CANVAS_WIDTH / 2.0,
            ball_y: CANVAS_HEIGHT / 2.0,
            ball_vx: INITIAL_BALL_SPEED,
            ball_vy: 3.0,
            player_1_y: (CANVAS_HEIGHT - PADDLE_HEIGHT) / 2.0,
            player_2_y: (CANVAS_HEIGHT - PADDLE_HEIGHT) / 2.0,
            score_1: 0,
            score_2: 0,
            particles: Vec::new(),
            shake_x: 0.0,
            shake_y: 0.0,
            shake_intensity: 0.0,
            canvas_width: CANVAS_WIDTH,
            canvas_height: CANVAS_HEIGHT,
        }
    }

    fn add_particles(&mut self, x: f64, y: f64, count: usize) {
        for _ in 0..count {
            self.particles.push(Particle::new(x, y));
        }
    }

    fn update_particles(&mut self) {
        for particle in &mut self.particles {
            particle.update();
        }
        self.particles.retain(|p| p.is_alive());
    }

    fn update_shake(&mut self) {
        if self.shake_intensity > 0.0 {
            self.shake_x = (js_sys::Math::random() - 0.5) * self.shake_intensity * 0.3;
            self.shake_y = (js_sys::Math::random() - 0.5) * self.shake_intensity * 0.3;
            self.shake_intensity *= 0.95;
            if self.shake_intensity < 0.05 {
                self.shake_intensity = 0.0;
                self.shake_x = 0.0;
                self.shake_y = 0.0;
            }
        }
    }

    fn trigger_shake(&mut self, intensity: f64) {
        self.shake_intensity = intensity;
    }
}

fn update_ui_elements(state: &GameState) {
    state.score_1.to_string().pipe(|score_text| {
        let elem: Element = dom::get_element_by_id("player1-score").unwrap();
        elem.set_text_content(Some(&score_text));
    });
    state.score_2.to_string().pipe(|score_text| {
        let elem: Element = dom::get_element_by_id("player2-score").unwrap();
        elem.set_text_content(Some(&score_text));
    });
}

fn render_frame(context: &CanvasRenderingContext2d, state: &GameState) {
    // background and center line
    context.set_fill_style_str("#FFFFFF");
    context.clear_rect(0.0, 0.0, CANVAS_WIDTH, CANVAS_HEIGHT);
    (0..CANVAS_HEIGHT as i32).step_by(20).for_each(|i| context.fill_rect(CANVAS_WIDTH / 2.0 - 2.0, i as f64, 4.0, 10.0));

    // paddles
    context.fill_rect(10.0, state.player_1_y, PADDLE_WIDTH, PADDLE_HEIGHT);
    context.fill_rect(CANVAS_WIDTH - 10.0 - PADDLE_WIDTH, state.player_2_y, PADDLE_WIDTH, PADDLE_HEIGHT);

    // ball
    let half_size = BALL_SIZE / 2.0;
    context.fill_rect(state.ball_x - half_size, state.ball_y - half_size, BALL_SIZE, BALL_SIZE);

    // particles
    state.particles.iter().for_each(|particle| {
        let alpha = particle.life / particle.max_life;
        context.set_fill_style_str(&format!("rgba(255, 255, 255, {})", alpha));
        context.fill_rect(particle.x - 2.0, particle.y - 2.0, 4.0, 4.0);
    });
}

fn apply_screen_shake(state: &GameState) {
    let game_elem: Element = dom::get_element_by_id("game-container").unwrap();
    let transform = if state.shake_intensity > 0.0 {
        format!("transform: translate({}px, {}px);", state.shake_x, state.shake_y)
    } else {
        "transform: translate(0px, 0px);".to_string()
    };
    let _ = game_elem.set_attribute("style", &transform);
}

fn update_game_physics(state: &mut GameState) {
    // ball movement
    let (new_x, new_y, new_vx, new_vy) = calculate_ball_movement(state.ball_x, state.ball_y, state.ball_vx, state.ball_vy);
    state.ball_x = new_x;
    state.ball_y = new_y;
    state.ball_vx = new_vx;
    state.ball_vy = new_vy;

    // wall collision
    check_wall_collision(state.ball_y, state.ball_vy).pipe(|collision_opt| {
        collision_opt.map(|collision| {
            state.ball_vy = collision.new_vy;
            (state.ball_x, state.ball_y).pipe(|(x, y)| {
                state.add_particles(x, y, PARTICLE_COUNT_WALL);
                state.trigger_shake(collision.intensity);
                audio::play_beep(400.0, 0.1);
            });
        })
    });

    // paddle collision
    check_paddle_collision(state.ball_x, state.ball_y, state.ball_vx, state.ball_vy, state.player_1_y, state.player_2_y).pipe(|collision_opt| {
        collision_opt.map(|collision| {
            state.ball_vx = collision.new_vx;
            state.ball_vy = collision.new_vy;
            (collision.collision_x, collision.collision_y).pipe(|(x, y)| {
                state.add_particles(x, y, PARTICLE_COUNT_PADDLE);
                state.trigger_shake(collision.intensity);
                audio::play_beep(600.0, 0.15);
            });
        })
    });

    // scoring
    check_scoring(state.ball_x).pipe(|score_opt| {
        score_opt.map(|scored_player| match scored_player {
            1 => {
                state.score_1 += 1;
                let (x, y, vx, vy) = generate_ball_reset_state(false);
                (state.ball_x, state.ball_y, state.ball_vx, state.ball_vy) = (x, y, vx, vy);
                audio::play_beep(200.0, 0.3);
            }
            2 => {
                state.score_2 += 1;
                let (x, y, vx, vy) = generate_ball_reset_state(true);
                (state.ball_x, state.ball_y, state.ball_vx, state.ball_vy) = (x, y, vx, vy);
                audio::play_beep(200.0, 0.3);
            }
            _ => {}
        })
    });
}

fn update_paddle_movement(state: &mut GameState, keys: &std::collections::HashSet<String>, is_host: bool) {
    let up_pressed = keys.contains("ArrowUp");
    let down_pressed = keys.contains("ArrowDown");

    if is_host {
        state.player_1_y = calculate_paddle_movement(state.player_1_y, up_pressed, down_pressed);
    } else {
        state.player_2_y = calculate_paddle_movement(state.player_2_y, up_pressed, down_pressed);
    }
}

fn handle_game_tick(state: &mut GameState, keys: &std::collections::HashSet<String>, context: &CanvasRenderingContext2d, is_host: bool, peer_connection: &Rc<RefCell<Option<p2p::PeerConnection>>>) {
    update_paddle_movement(state, keys, is_host);

    if is_host {
        update_game_physics(state);
    }

    if let Some(con) = &*peer_connection.borrow() {
        if let Ok(msg) = serde_json::to_string(state) {
            con.send_message(&msg);
        }
    }

    state.update_particles();
    state.update_shake();

    apply_screen_shake(state);
    render_frame(context, state);
    update_ui_elements(state);
}

fn setup_input_handlers() -> Rc<RefCell<std::collections::HashSet<String>>> {
    let key_state = Rc::new(RefCell::new(std::collections::HashSet::new()));

    dom::document().body().unwrap().pipe(|body| {
        let key_state_down = key_state.clone();
        dom::onkeydown(&body, move |event: KeyboardEvent| {
            key_state_down.borrow_mut().insert(event.key());
        });

        let key_state_up = key_state.clone();
        dom::onkeyup(&body, move |event: KeyboardEvent| {
            key_state_up.borrow_mut().remove(&event.key());
        });
    });

    key_state
}

fn setup_canvas_and_context() -> (HtmlCanvasElement, CanvasRenderingContext2d) {
    let canvas: HtmlCanvasElement = dom::get_element_by_id("game").unwrap();
    let context = dom::get_canvas_context(&canvas).unwrap();
    dom::setup_canvas_scaling(&canvas, &context);
    (canvas, context)
}

pub fn render_game(peer_connection: Rc<RefCell<Option<p2p::PeerConnection>>>, is_host: bool, game_state: Rc<RefCell<GameState>>) {
    dom::document().body().unwrap().set_inner_html(PONG_HTML);
    let _ = dom::document().body().unwrap().set_attribute("style", "background-color: #000");
    let (_canvas, context) = setup_canvas_and_context();
    let key_state = setup_input_handlers();

    type GameLoopHandle = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;
    let f: GameLoopHandle = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut state = game_state.borrow_mut();
        let keys = key_state.borrow();

        handle_game_tick(&mut state, &keys, &context, is_host, &peer_connection);

        dom::window().request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }) as Box<dyn FnMut()>));

    dom::window().request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    view::init();

    let game_state = Rc::new(RefCell::new(GameState::new()));
    let is_host = Rc::new(RefCell::new(false));

    let on_message = {
        let game_state = game_state.clone();
        let is_host = is_host.clone();
        Rc::new(move |msg: String| {
            if let Ok(new_state) = serde_json::from_str::<GameState>(&msg) {
                let mut state = game_state.borrow_mut();
                if *is_host.borrow() {
                    // host only receives guest paddle position
                    state.player_2_y = new_state.player_2_y;
                } else {
                    // guest receives game state but preserves own paddle position
                    let own_paddle_y = state.player_2_y;
                    *state = new_state;
                    state.player_2_y = own_paddle_y;
                }
            }
        })
    };

    let on_connection_established_host = {
        let game_state = game_state.clone();
        let is_host = is_host.clone();
        Rc::new(move |peer_connection: Rc<RefCell<Option<p2p::PeerConnection>>>| {
            *is_host.borrow_mut() = true;
            render_game(peer_connection, true, game_state.clone());
        })
    };
    let on_connection_established_guest = {
        let game_state = game_state.clone();
        let is_host = is_host.clone();
        Rc::new(move |peer_connection: Rc<RefCell<Option<p2p::PeerConnection>>>| {
            *is_host.borrow_mut() = false;
            render_game(peer_connection, false, game_state.clone());
        })
    };

    let callbacks_host = view::ActorCallbacks {
        on_connection_established: on_connection_established_host,
        on_message: on_message.clone(),
    };
    let callbacks_guest = view::ActorCallbacks {
        on_connection_established: on_connection_established_guest,
        on_message: on_message.clone(),
    };

    view::render_role_selection(move || view::render_host(callbacks_host.clone()), move || view::render_guest(callbacks_guest.clone()));

    Ok(())
}
