use web_sys::*;

pub fn play_beep(frequency: f32, duration: f64) {
    if let Ok(audio_context) = AudioContext::new() {
        if let (Ok(oscillator), Ok(gain)) = (audio_context.create_oscillator(), audio_context.create_gain()) {
            let _ = oscillator.set_type(web_sys::OscillatorType::Square);
            let _ = oscillator.frequency().set_value(frequency);
            let _ = gain.connect_with_audio_node(&audio_context.destination());
            let _ = oscillator.connect_with_audio_node(&gain);
            let _ = oscillator.start();
            let _ = oscillator.stop_with_when(audio_context.current_time() + duration);
        }
    }
}
