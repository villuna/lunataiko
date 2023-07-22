use std::{io, path::Path, rc::Rc};

use crate::{
    app::credits::CreditsScreen,
    parser::parse_tja_file,
    render::{
        self,
        texture::{Sprite, Texture},
    },
    track::Song,
};
use egui::RichText;
use kira::{
    manager::AudioManager,
    sound::{
        static_sound::{StaticSoundData, StaticSoundSettings},
        streaming::{StreamingSoundData, StreamingSoundHandle, StreamingSoundSettings},
        FromFileError,
    },
    tween::Tween,
};
use lazy_static::lazy_static;

use super::{taiko_mode::TaikoMode, GameState};

type SongHandle = StreamingSoundHandle<FromFileError>;

lazy_static! {
    static ref IN_TWEEN: Tween = Tween {
        start_time: kira::StartTime::Immediate,
        duration: std::time::Duration::from_secs_f32(0.2),
        easing: kira::tween::Easing::OutPowi(2),
    };
    static ref OUT_TWEEN: Tween = Tween {
        start_time: kira::StartTime::Immediate,
        duration: std::time::Duration::from_secs_f32(0.2),
        easing: kira::tween::Easing::InPowi(2),
    };
}

const SONGS_DIR: &str = "songs";

pub struct SongSelect {
    test_tracks: Vec<Rc<Song>>,
    selected: Option<usize>,
    difficulty: usize,
    song_handle: Option<SongHandle>,
    bg_sprite: Sprite,
    go_to_credits: bool,
    exit: bool,

    don_tex: Rc<Texture>,
    kat_tex: Rc<Texture>,
    big_don_tex: Rc<Texture>,
    big_kat_tex: Rc<Texture>,

    go_to_song: Option<(usize, usize)>,
}

fn read_song_list_dir<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Rc<Song>>> {
    let dir = std::fs::read_dir(path)?;
    let mut res = Vec::new();

    for file in dir.flatten() {
        if file.file_type().map(|ty| ty.is_dir()).unwrap_or(false) {
            let subdir_path = file.path();

            match read_song_dir(&subdir_path) {
                Ok(song) => res.push(Rc::new(song)),
                Err(e) => eprintln!(
                    "error encountered while trying to read song at directory {}: {e}",
                    subdir_path.to_string_lossy()
                ),
            }
        }
    }

    Ok(res)
}

fn read_song_dir<P: AsRef<Path>>(path: P) -> anyhow::Result<Song> {
    let dir_name = path.as_ref().file_name().ok_or(io::Error::new(
        io::ErrorKind::InvalidData,
        "couldn't read directory name",
    ))?;

    let tja_file_path = path
        .as_ref()
        .join(format!("{}.tja", dir_name.to_string_lossy()));
    let tja_file_contents = std::fs::read_to_string(tja_file_path)?;

    let mut song = parse_tja_file(&tja_file_contents)?;

    let audio_filename = path
        .as_ref()
        .join(&song.audio_filename)
        .to_string_lossy()
        .into_owned();

    song.audio_filename = audio_filename;
    Ok(song)
}

impl SongSelect {
    pub fn new(
        bg_sprite: Sprite,
        don_tex: Rc<Texture>,
        kat_tex: Rc<Texture>,
        big_don_tex: Rc<Texture>,
        big_kat_tex: Rc<Texture>,
    ) -> anyhow::Result<Self> {
        let test_tracks = read_song_list_dir(SONGS_DIR)?;

        Ok(SongSelect {
            test_tracks,
            bg_sprite,
            selected: None,
            difficulty: 0,
            song_handle: None,
            go_to_credits: false,
            exit: false,
            go_to_song: None,
            don_tex,
            kat_tex,
            big_don_tex,
            big_kat_tex,
        })
    }

    fn play_preview(
        &mut self,
        audio: &mut AudioManager,
        selected: usize,
    ) -> anyhow::Result<StreamingSoundHandle<FromFileError>> {
        let selected = &self.test_tracks[selected];

        let settings = StreamingSoundSettings::default()
            .start_position(selected.demostart as _)
            .fade_in_tween(Some(*IN_TWEEN))
            .loop_behavior(Some(kira::LoopBehavior {
                start_position: selected.demostart as _,
            }));

        let song = StreamingSoundData::from_file(&selected.audio_filename, settings)?;

        Ok(audio.play(song)?)
    }
}

impl GameState for SongSelect {
    fn update(
        &mut self,
        _delta: f32,
        audio: &mut AudioManager,
        renderer: &render::Renderer,
    ) -> super::StateTransition {
        if self.go_to_credits {
            if let Some(handle) = self.song_handle.as_mut() {
                handle.stop(*OUT_TWEEN).unwrap();
            }

            self.go_to_credits = false;
            super::StateTransition::Push(Box::new(CreditsScreen::new()))
        } else if let Some((song_id, difficulty)) = self.go_to_song {
            let sound_data = StaticSoundData::from_file(
                &self.test_tracks[song_id].audio_filename,
                StaticSoundSettings::default(),
            )
            .unwrap();

            self.go_to_song = None;

            if let Some(handle) = self.song_handle.as_mut() {
                handle.stop(Default::default()).unwrap();
            }

            super::StateTransition::Push(Box::new(TaikoMode::new(
                Rc::clone(&self.test_tracks[song_id]),
                difficulty,
                sound_data,
                audio,
                &self.don_tex,
                &self.kat_tex,
                &self.big_don_tex,
                &self.big_kat_tex,
                renderer,
            )))
        } else if self.exit {
            super::StateTransition::Exit
        } else {
            super::StateTransition::Continue
        }
    }
    fn render<'a>(&'a mut self, ctx: &mut render::RenderContext<'a>) {
        ctx.render(&self.bg_sprite)
    }

    fn debug_ui(&mut self, ctx: egui::Context, audio: &mut AudioManager) {
        egui::SidePanel::left("main menu")
            .resizable(false)
            .show(&ctx, |ui| {
                ui.label(
                    RichText::new("LunaTaiko Demo!")
                        .text_style(egui::TextStyle::Heading)
                        .size(40.0)
                        .color(egui::Color32::from_rgb(255, 84, 54))
                        .strong(),
                );

                ui.label(RichText::new("\"That's a working title!\"").italics());

                ui.add_space(50.0);

                let old_song = self.selected;

                egui::ComboBox::from_label("Song select")
                    .selected_text(
                        RichText::new(
                            self.selected
                                .map(|id| self.test_tracks[id].title.as_str())
                                .unwrap_or("None"),
                        )
                        .size(20.0),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.selected,
                            None,
                            RichText::new("none").size(15.0),
                        );

                        for (id, song) in self.test_tracks.iter().enumerate() {
                            ui.selectable_value(
                                &mut self.selected,
                                Some(id),
                                RichText::new(&song.title).size(15.0),
                            );
                        }
                    });

                if self.selected != old_song {
                    if let Some(handle) = self.song_handle.as_mut() {
                        handle.stop(*OUT_TWEEN).unwrap();
                    }

                    self.song_handle = self
                        .selected
                        .map(|id| self.play_preview(audio, id).unwrap());
                }

                ui.add_space(800.0);

                if ui.button(RichText::new("credits").size(20.0)).clicked() {
                    self.go_to_credits = true;
                }

                ui.add_space(10.0);

                if ui.button(RichText::new("exit").size(20.0)).clicked() {
                    self.exit = true;
                }
            });

        if let Some(song_index) = self.selected {
            egui::Window::new("difficulty select").show(&ctx, |ui| {
                const DIFFICULTY_NAMES: [&str; 5] = ["Easy", "Normal", "Hard", "Oni", "Ura"];

                egui::TopBottomPanel::top("difficulty select panel").show_inside(ui, |ui| {
                    for (i, difficulty) in self.test_tracks[song_index]
                        .difficulties
                        .iter()
                        .enumerate()
                        .filter_map(|(i, d)| d.as_ref().map(|dinner| (i, dinner)))
                    {
                        egui::SidePanel::left(format!("{} difficulty block", DIFFICULTY_NAMES[i]))
                            .show_inside(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.difficulty,
                                    i,
                                    RichText::new(format!(
                                        "{}\n{}★",
                                        DIFFICULTY_NAMES[i], difficulty.star_level
                                    ))
                                    .size(20.0),
                                );
                            });
                    }
                });

                if ui.button(RichText::new("Play!").size(17.0)).clicked() {
                    self.go_to_song = Some((song_index, self.difficulty));
                }
            });
        }
    }
}
