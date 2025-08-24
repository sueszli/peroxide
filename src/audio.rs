use wasm_bindgen::JsCast;
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

#[allow(unused)]
pub fn play_background_music() {
    let Ok(audio_context) = AudioContext::new() else {
        return;
    };

    // 8 bars, 4/4 time
    let melody = [
        (523.25, 0.25), // C5
        (659.25, 0.25), // E5
        (783.99, 0.5),  // G5
        (659.25, 0.25), // E5
        (523.25, 0.5),  // C5
        (392.0, 0.25),  // G4
        (440.0, 0.25),  // A4
        (493.88, 0.25), // B4
        (523.25, 0.5),  // C5
        (783.99, 0.25), // G5
        (659.25, 0.25), // E5
        (587.33, 0.5),  // D5
        (523.25, 0.25), // C5
        (587.33, 0.25), // D5
        (659.25, 0.25), // E5
        (698.46, 0.25), // F5
        (783.99, 0.5),  // G5
        (659.25, 0.25), // E5
        (523.25, 0.25), // C5
        (392.0, 0.25),  // G4
        (440.0, 0.25),  // A4
        (493.88, 0.5),  // B4
        (523.25, 0.75), // C5 (longer for ending)
        (392.0, 0.25),  // G4
    ];
    let bass = [
        (130.81, 1.0), // C3
        (196.0, 1.0),  // G3
        (164.81, 1.0), // E3
        (146.83, 1.0), // D3
        (130.81, 1.0), // C3
        (174.61, 1.0), // F3
        (196.0, 1.0),  // G3
        (130.81, 1.0), // C3
    ];

    let mut total_duration = 0.0;

    for (frequency, duration) in melody.iter() {
        let start_time = audio_context.current_time() + total_duration;

        if let (Ok(oscillator), Ok(gain)) = (audio_context.create_oscillator(), audio_context.create_gain()) {
            let wave_type = match ((total_duration * 4.0) as usize) % 4 {
                0 => web_sys::OscillatorType::Square,
                1 => web_sys::OscillatorType::Triangle,
                2 => web_sys::OscillatorType::Sawtooth,
                _ => web_sys::OscillatorType::Square,
            };
            oscillator.set_type(wave_type);
            oscillator.frequency().set_value(*frequency);

            let volume = 0.1; // important to make fx audible

            gain.gain().set_value_at_time(0.0, start_time).ok();
            gain.gain().linear_ramp_to_value_at_time(volume, start_time + 0.02).ok();
            gain.gain().set_value_at_time(volume * 0.8, start_time + duration - 0.05).ok();
            gain.gain().linear_ramp_to_value_at_time(0.0, start_time + duration).ok();

            gain.connect_with_audio_node(&audio_context.destination()).ok();
            oscillator.connect_with_audio_node(&gain).ok();
            oscillator.start_with_when(start_time).ok();
            oscillator.stop_with_when(start_time + duration).ok();
        }

        total_duration += duration;
    }

    let mut bass_time = 0.0;
    for (bass_freq, bass_duration) in bass.iter() {
        let start_time = audio_context.current_time() + bass_time;

        if let (Ok(bass_osc), Ok(bass_gain)) = (audio_context.create_oscillator(), audio_context.create_gain()) {
            bass_osc.set_type(web_sys::OscillatorType::Square);
            bass_osc.frequency().set_value(*bass_freq);

            bass_gain.gain().set_value_at_time(0.0, start_time).ok();
            bass_gain.gain().linear_ramp_to_value_at_time(0.15, start_time + 0.05).ok();
            bass_gain.gain().exponential_ramp_to_value_at_time(0.05, start_time + 0.2).ok();
            bass_gain.gain().set_value_at_time(0.05, start_time + bass_duration - 0.1).ok();
            bass_gain.gain().exponential_ramp_to_value_at_time(0.001, start_time + bass_duration).ok();

            bass_gain.connect_with_audio_node(&audio_context.destination()).ok();
            bass_osc.connect_with_audio_node(&bass_gain).ok();
            bass_osc.start_with_when(start_time).ok();
            bass_osc.stop_with_when(start_time + bass_duration).ok();
        }

        bass_time += bass_duration;
    }

    let percussion_hits = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    for &hit_time in percussion_hits.iter() {
        let perc_start = audio_context.current_time() + hit_time;

        if let (Ok(perc_osc), Ok(perc_gain)) = (audio_context.create_oscillator(), audio_context.create_gain()) {
            perc_osc.set_type(web_sys::OscillatorType::Square);
            perc_osc.frequency().set_value(200.0);

            perc_gain.gain().set_value_at_time(0.0, perc_start).ok();
            perc_gain.gain().linear_ramp_to_value_at_time(0.15, perc_start + 0.01).ok();
            perc_gain.gain().exponential_ramp_to_value_at_time(0.001, perc_start + 0.08).ok();

            perc_gain.connect_with_audio_node(&audio_context.destination()).ok();
            perc_osc.connect_with_audio_node(&perc_gain).ok();
            perc_osc.start_with_when(perc_start).ok();
            perc_osc.stop_with_when(perc_start + 0.08).ok();
        }
    }

    // 8s loop
    let loop_duration = 8.0;
    let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        play_background_music();
    }) as Box<dyn Fn()>);

    web_sys::window()
        .unwrap()
        .set_timeout_with_callback_and_timeout_and_arguments_0(callback.as_ref().unchecked_ref(), (loop_duration * 1000.0) as i32)
        .ok();

    callback.forget();
}
