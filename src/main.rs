extern crate glium;
mod audio_player;

fn main() {
    let _result = audio_player::audio_player::play_file(String::from("./../03 - Rhymes Like Dimes.flac"));

    std::thread::sleep(std::time::Duration::from_secs(30));
}
