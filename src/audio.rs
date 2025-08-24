use web_sys::*;

pub fn play_beep(frequency: f32, duration: f64) {
    let Ok(audio_context) = AudioContext::new() else {
        return;
    };

    let (Ok(oscillator), Ok(gain)) = (audio_context.create_oscillator(), audio_context.create_gain()) else {
        return;
    };

    oscillator.set_type(web_sys::OscillatorType::Square);
    oscillator.frequency().set_value(frequency);
    gain.connect_with_audio_node(&audio_context.destination()).ok();
    oscillator.connect_with_audio_node(&gain).ok();
    oscillator.start().ok();
    oscillator.stop_with_when(audio_context.current_time() + duration).ok();
}
