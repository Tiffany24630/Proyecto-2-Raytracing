use super::SceneState;

const PAGE_DURATION_SECONDS: f32 = 3.6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NarrativeMessage {
    pub first_line: &'static str,
    pub second_line: &'static str,
    pub corrupted: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct NarrativeController {
    scene: SceneState,
    page: usize,
    elapsed_seconds: f32,
}

impl Default for NarrativeController {
    fn default() -> Self {
        Self::new(SceneState::Exterior)
    }
}

impl NarrativeController {
    pub const fn new(scene: SceneState) -> Self {
        Self {
            scene,
            page: 0,
            elapsed_seconds: 0.0,
        }
    }

    pub fn update(&mut self, scene: SceneState, delta_seconds: f32) -> bool {
        if scene != self.scene {
            self.scene = scene;
            self.page = 0;
            self.elapsed_seconds = 0.0;
            return true;
        }
        if delta_seconds <= 0.0 || self.page + 1 >= messages(scene).len() {
            return false;
        }

        self.elapsed_seconds += delta_seconds;
        if self.elapsed_seconds >= PAGE_DURATION_SECONDS {
            self.elapsed_seconds -= PAGE_DURATION_SECONDS;
            self.page += 1;
            return true;
        }
        false
    }

    pub fn message(self) -> Option<NarrativeMessage> {
        messages(self.scene).get(self.page).copied()
    }
}

fn messages(scene: SceneState) -> &'static [NarrativeMessage] {
    match scene {
        SceneState::Exterior => &[
            NarrativeMessage {
                first_line: "ESTE LUGAR NO FUE CONSTRUIDO",
                second_line: "PARA SER HABITADO",
                corrupted: false,
            },
            NarrativeMessage {
                first_line: "FUE CONSTRUIDO PARA RECORDAR",
                second_line: "ENTRA EN EL ESPACIO PRESERVADO",
                corrupted: false,
            },
        ],
        SceneState::Entering => &[],
        SceneState::Temple => &[
            NarrativeMessage {
                first_line: "ACTIVA EL OJO DE DIOS",
                second_line: "LAS MINIATURAS DESPLEGARAN SUS MUNDOS",
                corrupted: false,
            },
            NarrativeMessage {
                first_line: "UNA MEMORIA PRESERVADA",
                second_line: "TAMBIEN PUEDE SER UNA PRISION",
                corrupted: false,
            },
        ],
        SceneState::Puzzle => &[
            NarrativeMessage {
                first_line: "DEX PUEDE ARRASTRAR EL ESPACIO",
                second_line: "VINCULA UNA PIEZA CON E",
                corrupted: false,
            },
            NarrativeMessage {
                first_line: "LA POSICION ES MEDIA VERDAD",
                second_line: "USA LAS SILUETAS Y LA PERSPECTIVA",
                corrupted: false,
            },
        ],
        SceneState::MemoryRestored => &[NarrativeMessage {
            first_line: "LOS FRAGMENTOS RECUERDAN",
            second_line: "LO QUE LA SALA OLVIDO",
            corrupted: false,
        }],
        SceneState::Melanta => &[NarrativeMessage {
            first_line: "LA RESTAURACION NO ESTA PERMITIDA",
            second_line: "LA MEMORIA DEBE PERMANECER QUIETA",
            corrupted: true,
        }],
        SceneState::DesertPavilion => &[NarrativeMessage {
            first_line: "LA MINIATURA HA DESPLEGADO SU ESPACIO",
            second_line: "EL DESIERTO RECUERDA EN SILENCIO",
            corrupted: false,
        }],
        SceneState::MahavaipulyaChamber => &[NarrativeMessage {
            first_line: "LAS PAGINAS SEPARADAS PIERDEN SU VOZ",
            second_line: "DEVUELVELAS AL LIBRO ANTES DE MELANTA",
            corrupted: false,
        }],
        SceneState::LuyangAcademy => &[NarrativeMessage {
            first_line: "UNA PINTURA TAMBIEN CONSERVA UN MUNDO",
            second_line: "ORDENA SUS PARTES ANTES DE CONFIRMAR",
            corrupted: false,
        }],
        SceneState::Final => &[NarrativeMessage {
            first_line: "MEMORY RESTORED",
            second_line: "PRESERVAR TAMBIEN ES ELEGIR",
            corrupted: false,
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::{NarrativeController, SceneState};

    #[test]
    fn narrative_advances_pages_and_resets_on_state_change() {
        let mut narrative = NarrativeController::default();
        let first = narrative.message().expect("exterior narrative");
        assert!(narrative.update(SceneState::Exterior, 4.0));
        assert_ne!(narrative.message(), Some(first));

        assert!(narrative.update(SceneState::Melanta, 0.0));
        assert!(narrative.message().expect("melanta narrative").corrupted);
        assert!(!narrative.update(SceneState::Melanta, 10.0));
    }

    #[test]
    fn entering_does_not_cover_the_portal_transition() {
        let narrative = NarrativeController::new(SceneState::Entering);
        assert_eq!(narrative.message(), None);
    }
}
