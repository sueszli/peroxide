#[derive(Debug, Clone, PartialEq)]
pub struct CollisionInfo {
    pub new_vx: f64,
    pub new_vy: f64,
    pub collision_x: f64,
    pub collision_y: f64,
    pub intensity: f64,
}

pub const CANVAS_WIDTH: f64 = 800.0;
pub const CANVAS_HEIGHT: f64 = 600.0;
pub const PADDLE_WIDTH: f64 = 15.0;
pub const PADDLE_HEIGHT: f64 = 100.0;
pub const BALL_SIZE: f64 = 16.0;
pub const PADDLE_SPEED: f64 = 8.0;
pub const INITIAL_BALL_SPEED: f64 = 4.0;
pub const BALL_ACCELERATION: f64 = 1.08;
pub const SPIN_FACTOR: f64 = 6.0;
pub const VELOCITY_CARRY: f64 = 0.3;

pub fn calculate_ball_movement(ball_x: f64, ball_y: f64, ball_vx: f64, ball_vy: f64) -> (f64, f64, f64, f64) {
    (ball_x + ball_vx, ball_y + ball_vy, ball_vx, ball_vy)
}

pub fn check_wall_collision(ball_y: f64, ball_vy: f64) -> Option<CollisionInfo> {
    let ball_radius = BALL_SIZE / 2.0;
    let top_collision = ball_y <= ball_radius;
    let bottom_collision = ball_y >= CANVAS_HEIGHT - ball_radius;

    (top_collision || bottom_collision).then(|| CollisionInfo {
        new_vx: 0.0, // wall collision doesn't affect horizontal velocity
        new_vy: -ball_vy,
        collision_x: 0.0, // not relevant for wall collision
        collision_y: ball_y,
        intensity: 2.0,
    })
}

pub fn check_paddle_collision(ball_x: f64, ball_y: f64, ball_vx: f64, ball_vy: f64, player1_y: f64, player2_y: f64) -> Option<CollisionInfo> {
    let ball_radius = BALL_SIZE / 2.0;
    let left_paddle_x = 10.0 + PADDLE_WIDTH;
    let right_paddle_x = CANVAS_WIDTH - 10.0 - PADDLE_WIDTH;

    // left paddle collision
    if ball_x <= left_paddle_x + ball_radius && ball_y >= player1_y && ball_y <= player1_y + PADDLE_HEIGHT && ball_vx < 0.0 {
        let hit_pos = (ball_y - (player1_y + PADDLE_HEIGHT / 2.0)) / (PADDLE_HEIGHT / 2.0);
        return Some(CollisionInfo {
            new_vx: -ball_vx * BALL_ACCELERATION,
            new_vy: hit_pos * SPIN_FACTOR + ball_vy * VELOCITY_CARRY,
            collision_x: ball_x,
            collision_y: ball_y,
            intensity: 3.0,
        });
    }

    // right paddle collision
    if ball_x >= right_paddle_x - ball_radius && ball_y >= player2_y && ball_y <= player2_y + PADDLE_HEIGHT && ball_vx > 0.0 {
        let hit_pos = (ball_y - (player2_y + PADDLE_HEIGHT / 2.0)) / (PADDLE_HEIGHT / 2.0);
        return Some(CollisionInfo {
            new_vx: -ball_vx * BALL_ACCELERATION,
            new_vy: hit_pos * SPIN_FACTOR + ball_vy * VELOCITY_CARRY,
            collision_x: ball_x,
            collision_y: ball_y,
            intensity: 3.0,
        });
    }

    None
}

pub fn calculate_paddle_movement(current_y: f64, up_pressed: bool, down_pressed: bool) -> f64 {
    let delta = match (up_pressed, down_pressed) {
        (true, false) => -PADDLE_SPEED,
        (false, true) => PADDLE_SPEED,
        _ => 0.0,
    };
    (current_y + delta).clamp(0.0, CANVAS_HEIGHT - PADDLE_HEIGHT)
}

pub fn generate_ball_reset_state(towards_player1: bool) -> (f64, f64, f64, f64) {
    let x = CANVAS_WIDTH / 2.0;
    let y = CANVAS_HEIGHT / 2.0;
    let vx = if towards_player1 { -INITIAL_BALL_SPEED } else { INITIAL_BALL_SPEED };

    #[cfg(target_arch = "wasm32")]
    let vy = (js_sys::Math::random() - 0.5) * SPIN_FACTOR;

    #[cfg(not(target_arch = "wasm32"))]
    let vy = 0.0; // for testing on non-WASM targets

    (x, y, vx, vy)
}

pub fn check_scoring(ball_x: f64) -> Option<u8> {
    if ball_x < 0.0 {
        Some(2) // player 2 scores
    } else if ball_x > CANVAS_WIDTH {
        Some(1) // player 1 scores
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_ball_movement() {
        let (new_x, new_y, new_vx, new_vy) = calculate_ball_movement(100.0, 200.0, 5.0, -3.0);
        assert_eq!(new_x, 105.0);
        assert_eq!(new_y, 197.0);
        assert_eq!(new_vx, 5.0);
        assert_eq!(new_vy, -3.0);
    }

    #[test]
    fn test_check_wall_collision_top() {
        let collision = check_wall_collision(5.0, -2.0);
        assert!(collision.is_some());
        let info = collision.unwrap();
        assert_eq!(info.new_vy, 2.0);
        assert_eq!(info.intensity, 2.0);
    }

    #[test]
    fn test_check_wall_collision_bottom() {
        let collision = check_wall_collision(595.0, 2.0);
        assert!(collision.is_some());
        let info = collision.unwrap();
        assert_eq!(info.new_vy, -2.0);
    }

    #[test]
    fn test_check_wall_collision_none() {
        let collision = check_wall_collision(300.0, 2.0);
        assert!(collision.is_none());
    }

    #[test]
    fn test_calculate_paddle_movement_up() {
        let new_y = calculate_paddle_movement(250.0, true, false);
        assert_eq!(new_y, 242.0);
    }

    #[test]
    fn test_calculate_paddle_movement_down() {
        let new_y = calculate_paddle_movement(250.0, false, true);
        assert_eq!(new_y, 258.0);
    }

    #[test]
    fn test_calculate_paddle_movement_bounds() {
        let new_y = calculate_paddle_movement(0.0, true, false);
        assert_eq!(new_y, 0.0); // can't go below 0

        let new_y = calculate_paddle_movement(500.0, false, true);
        assert_eq!(new_y, 500.0); // can't go above max
    }

    #[test]
    fn test_check_scoring() {
        assert_eq!(check_scoring(-5.0), Some(2));
        assert_eq!(check_scoring(805.0), Some(1));
        assert_eq!(check_scoring(400.0), None);
    }

    #[test]
    fn test_generate_ball_reset_state() {
        let (x, y, vx_left, vy) = generate_ball_reset_state(true);
        assert_eq!(x, CANVAS_WIDTH / 2.0);
        assert_eq!(y, CANVAS_HEIGHT / 2.0);
        assert_eq!(vx_left, -INITIAL_BALL_SPEED);

        let (x2, y2, vx_right, _vy2) = generate_ball_reset_state(false);
        assert_eq!(x2, CANVAS_WIDTH / 2.0);
        assert_eq!(y2, CANVAS_HEIGHT / 2.0);
        assert_eq!(vx_right, INITIAL_BALL_SPEED);

        // On WASM, vy should be random; on native, it should be 0
        #[cfg(target_arch = "wasm32")]
        assert!((-3.0..=3.0).contains(&vy));

        #[cfg(not(target_arch = "wasm32"))]
        {
            assert_eq!(vy, 0.0);
            assert_eq!(_vy2, 0.0);
        }
    }

    #[test]
    fn test_check_paddle_collision_left() {
        let collision = check_paddle_collision(25.0, 250.0, -5.0, 0.0, 200.0, 300.0);
        assert!(collision.is_some());
        let info = collision.unwrap();
        assert!(info.new_vx > 0.0); // ball should reverse and accelerate
        assert_eq!(info.intensity, 3.0);
    }
}
