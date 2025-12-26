use std::{
    io::{BufReader, Cursor},
    path::PathBuf,
};

// launches a thread and uses rodio to play mp3 files
pub fn play_audio(audio_path: Option<PathBuf>, volume: f32) {
    std::thread::spawn(move || {
        if let Ok(mut stream_handle) = rodio::OutputStreamBuilder::open_default_stream() {
            stream_handle.log_on_drop(false);
            let mixer = stream_handle.mixer();
            let file = audio_path.and_then(|path| std::fs::File::open(path).ok());

            let sink = if let Some(file) = file {
                rodio::play(mixer, BufReader::new(file))
            } else {
                rodio::play(
                    mixer,
                    BufReader::new(Cursor::new(include_bytes!("../img/work_end.mp3"))),
                )
            };

            if let Ok(sink) = sink {
                sink.set_volume(volume);
                sink.sleep_until_end();
            }
        }
    });
}
