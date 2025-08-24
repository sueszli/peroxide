mod audio;
mod dom;
mod p2p;
mod utils;
mod view;

use std::{cell::RefCell, rc::Rc};

use js_sys::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;
use web_sys::*;

use utils::combinatorics::{Kestrel, Thrush};

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
            life: 60.0,
            max_life: 60.0,
        }
    }

    fn update(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
        self.vy += 0.1; // gravity
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
            ball_x: 400.0,
            ball_y: 300.0,
            ball_vx: 5.0,
            ball_vy: 2.0,
            player_1_y: 250.0,
            player_2_y: 250.0,
            score_1: 0,
            score_2: 0,
            particles: Vec::new(),
            shake_x: 0.0,
            shake_y: 0.0,
            shake_intensity: 0.0,
            canvas_width: 800.0,
            canvas_height: 600.0,
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
            self.shake_x = (js_sys::Math::random() - 0.5) * self.shake_intensity;
            self.shake_y = (js_sys::Math::random() - 0.5) * self.shake_intensity;
            self.shake_intensity *= 0.9;
            if self.shake_intensity < 0.1 {
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

const PONG_HTML: &str = r#"
    <div id="game-container">
        <h2>Pong</h2>
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
        
        h2 {
            color: #fff;
            font-family: 'Lucida Console', monospace;
            margin-bottom: 1rem;
            text-align: center;
            font-size: 2rem;
        }
    </style>
"#;

pub fn render_game(peer_connection: Rc<RefCell<Option<p2p::PeerConnection>>>, is_host: bool, game_state: Rc<RefCell<GameState>>) {
    dom::document().body().unwrap().set_inner_html(PONG_HTML);

    let canvas = dom::document().get_element_by_id("game").unwrap().dyn_into::<HtmlCanvasElement>().unwrap();

    let context = canvas.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap();

    // Audio is now handled per-call

    // Setup canvas scaling
    let window = web_sys::window().unwrap();
    let device_pixel_ratio = window.device_pixel_ratio();
    let display_width = (canvas.client_width() as f64 * device_pixel_ratio) as u32;
    let display_height = (canvas.client_height() as f64 * device_pixel_ratio) as u32;

    canvas.set_width(display_width);
    canvas.set_height(display_height);

    context.scale(device_pixel_ratio, device_pixel_ratio).unwrap();
    context.set_image_smoothing_enabled(false);

    let game_state_clone = game_state.clone();

    let document = dom::document();
    let key_state = Rc::new(RefCell::new(std::collections::HashSet::new()));

    document.body().unwrap().pipe(|body| {
        let key_state_down = key_state.clone();
        let keydown_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            key_state_down.borrow_mut().insert(event.key());
        }) as Box<dyn FnMut(_)>);

        body.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref()).unwrap();
        keydown_closure.forget();

        let key_state_up = key_state.clone();
        let keyup_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            key_state_up.borrow_mut().remove(&event.key());
        }) as Box<dyn FnMut(_)>);

        body.add_event_listener_with_callback("keyup", keyup_closure.as_ref().unchecked_ref()).unwrap();
        keyup_closure.forget();
    });

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut state = game_state_clone.borrow_mut();
        let keys = key_state.borrow();

        if is_host {
            if keys.contains("ArrowUp") {
                state.player_1_y -= 8.0;
            }
            if keys.contains("ArrowDown") {
                state.player_1_y += 8.0;
            }
            state.player_1_y = state.player_1_y.max(0.0).min(500.0);

            // Ball physics
            state.ball_x += state.ball_vx;
            state.ball_y += state.ball_vy;

            // Wall collision
            if state.ball_y <= 10.0 || state.ball_y >= 590.0 {
                state.ball_vy = -state.ball_vy;
                (state.ball_x, state.ball_y).pipe(|(x, y)| {
                    state.add_particles(x, y, 8);
                    state.trigger_shake(5.0);
                    audio::play_beep(400.0, 0.1);
                });
            }

            // Paddle collision
            let paddle_collision = if state.ball_x <= 25.0 && state.ball_y >= state.player_1_y && state.ball_y <= state.player_1_y + 100.0 && state.ball_vx < 0.0 {
                true
            } else if state.ball_x >= 775.0 && state.ball_y >= state.player_2_y && state.ball_y <= state.player_2_y + 100.0 && state.ball_vx > 0.0 {
                true
            } else {
                false
            };

            if paddle_collision {
                state.ball_vx = -state.ball_vx * 1.05; // Slight speed increase
                (state.ball_x, state.ball_y).pipe(|(x, y)| {
                    state.add_particles(x, y, 12);
                    state.trigger_shake(8.0);
                    audio::play_beep(600.0, 0.15);
                });
            }

            // Score
            if state.ball_x < 0.0 {
                state.score_2 += 1;
                state.ball_x = 400.0;
                state.ball_y = 300.0;
                state.ball_vx = 5.0;
                state.ball_vy = (js_sys::Math::random() - 0.5) * 4.0;
                audio::play_beep(200.0, 0.3);
            }
            if state.ball_x > 800.0 {
                state.score_1 += 1;
                state.ball_x = 400.0;
                state.ball_y = 300.0;
                state.ball_vx = -5.0;
                state.ball_vy = (js_sys::Math::random() - 0.5) * 4.0;
                audio::play_beep(200.0, 0.3);
            }

            if let Some(con) = &*peer_connection.borrow() {
                let msg = serde_json::to_string(&*state).unwrap();
                con.send_message(&msg);
            }
        } else {
            if keys.contains("ArrowUp") {
                state.player_2_y -= 8.0;
            }
            if keys.contains("ArrowDown") {
                state.player_2_y += 8.0;
            }
            state.player_2_y = state.player_2_y.max(0.0).min(500.0);

            if let Some(con) = &*peer_connection.borrow() {
                let msg = serde_json::to_string(&*state).unwrap();
                con.send_message(&msg);
            }
        }

        // Update systems
        state.update_particles();
        state.update_shake();

        // Apply screen shake
        if state.shake_intensity > 0.0 {
            let game_elem = dom::document().get_element_by_id("game-container").unwrap();
            let _ = game_elem.set_attribute("style", &format!("transform: translate({}px, {}px);", state.shake_x, state.shake_y));
        } else {
            let game_elem = dom::document().get_element_by_id("game-container").unwrap();
            let _ = game_elem.set_attribute("style", "transform: translate(0px, 0px);");
        }

        // Pixel-perfect drawing
        context.set_fill_style_str("#FFFFFF");
        context.clear_rect(0.0, 0.0, 800.0, 600.0);

        // Draw center line
        for i in (0..600).step_by(20) {
            context.fill_rect(398.0, i as f64, 4.0, 10.0);
        }

        // Draw paddles (pixel rectangles)
        context.fill_rect(10.0, state.player_1_y, 15.0, 100.0);
        context.fill_rect(775.0, state.player_2_y, 15.0, 100.0);

        // Draw ball (pixel square)
        context.fill_rect(state.ball_x - 8.0, state.ball_y - 8.0, 16.0, 16.0);

        // Draw particles
        for particle in &state.particles {
            let alpha = particle.life / particle.max_life;
            context.set_fill_style_str(&format!("rgba(255, 255, 255, {})", alpha));
            context.fill_rect(particle.x - 2.0, particle.y - 2.0, 4.0, 4.0);
        }

        // Update score display
        state.score_1.to_string().pipe(|score_text| {
            dom::document().get_element_by_id("player1-score").unwrap().tap(|elem| elem.set_text_content(Some(&score_text)));
        });
        state.score_2.to_string().pipe(|score_text| {
            dom::document().get_element_by_id("player2-score").unwrap().tap(|elem| elem.set_text_content(Some(&score_text)));
        });

        let window = web_sys::window().unwrap();
        window.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }) as Box<dyn FnMut()>));

    let window = web_sys::window().unwrap();
    window.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
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
                    state.player_2_y = new_state.player_2_y;
                } else {
                    *state = new_state;
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
