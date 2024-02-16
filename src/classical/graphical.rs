use std::time::Duration;

use glium::glutin::event::ElementState;
use glium::{implement_vertex, uniform, Program, Surface};

use crate::classical;
use crate::graphical::Canvas;

use self::{Move, MoveIdx};

use super::super::generic;
use super::*;

#[derive(Debug)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

struct Textures {
    white_pawn: glium::texture::Texture2d,
    white_rook: glium::texture::Texture2d,
    white_knight: glium::texture::Texture2d,
    white_bishop: glium::texture::Texture2d,
    white_queen: glium::texture::Texture2d,
    white_king: glium::texture::Texture2d,
    black_pawn: glium::texture::Texture2d,
    black_rook: glium::texture::Texture2d,
    black_knight: glium::texture::Texture2d,
    black_bishop: glium::texture::Texture2d,
    black_queen: glium::texture::Texture2d,
    black_king: glium::texture::Texture2d,
}

fn load_texture(
    facade: &impl glium::backend::Facade,
    filename: &'static str,
) -> glium::texture::Texture2d {
    let image = image::load(
        std::io::BufReader::new(
            std::fs::File::open(String::from("src/classical/icons/") + &filename).unwrap(),
        ),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let image_dimensions = image.dimensions();
    let image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    glium::texture::Texture2d::new(facade, image).unwrap()
}

impl Textures {
    fn new(facade: &impl glium::backend::Facade) -> Self {
        Self {
            white_pawn: load_texture(facade, "white pawn.png"),
            white_rook: load_texture(facade, "white rook.png"),
            white_knight: load_texture(facade, "white knight.png"),
            white_bishop: load_texture(facade, "white bishop.png"),
            white_queen: load_texture(facade, "white queen.png"),
            white_king: load_texture(facade, "white king.png"),
            black_pawn: load_texture(facade, "black pawn.png"),
            black_rook: load_texture(facade, "black rook.png"),
            black_knight: load_texture(facade, "black knight.png"),
            black_bishop: load_texture(facade, "black bishop.png"),
            black_queen: load_texture(facade, "black queen.png"),
            black_king: load_texture(facade, "black king.png"),
        }
    }
}

struct MoveButton {
    pos: (u8, u8),
    colour: (f32, f32, f32),
    move_idx: generic::MoveIdx,
}

pub struct GameInterface {
    // board: Board,
    board: Board,
    moves: Vec<Move>,
    board_ai: Option<generic::ai::AiOn>,
    // moves: Vec<generic::info::Move>,
    // best_move_idx: Option<MoveIdx>,
    move_buttons: Vec<MoveButton>,
    selected: Option<(u8, u8)>,
    textures: Textures,
    board_program: Program,
    texture_program: Program,
    highlight_program: Program,
}

impl GameInterface {
    fn get_board_pixel_rect(&self, state: &crate::graphical::State) -> Rect {
        if state.display_size.0 >= state.display_size.1 {
            let pad = (state.display_size.0 as f64 - state.display_size.1 as f64) / 2.0;
            Rect {
                x: pad,
                y: 0.0,
                w: state.display_size.1 as f64,
                h: state.display_size.1 as f64,
            }
        } else {
            let pad = (state.display_size.1 as f64 - state.display_size.0 as f64) / 2.0;
            Rect {
                x: 0.0,
                y: pad,
                w: state.display_size.0 as f64,
                h: state.display_size.0 as f64,
            }
        }
    }

    fn pixel_to_square(
        &self,
        state: &crate::graphical::State,
        pixels: (f64, f64),
    ) -> Option<(u8, u8)> {
        let rect = self.get_board_pixel_rect(state);
        let x_frac = (8.0 * (pixels.0 - rect.x) / rect.w).floor() as i128;
        let y_frac = (8.0 * (pixels.1 - rect.y) / rect.h).floor() as i128;
        if 0 <= x_frac && x_frac < 8 && 0 <= y_frac && y_frac < 8 {
            Some((x_frac as u8, y_frac as u8))
        } else {
            None
        }
    }
}

impl Canvas for GameInterface {
    fn new(facade: &impl glium::backend::Facade) -> Self {
        // let mut board = create_game();
        // let turn = board.get_turn();
        // let mut info = board.generate_info();
        // let best_move_idx = info.best_move(&mut board);
        // let moves = info.get_moves(turn).clone();
        let board = create_game();
        let board_ai_off = generic::ai::AiOff::new(board.clone());
        let moves = board_ai_off.get_moves();
        let board_ai = board_ai_off.start();

        Self {
            // board,
            // moves,
            // best_move_idx: board.get_best_move(),
            board,
            moves,
            board_ai: Some(board_ai),
            move_buttons: vec![],
            selected: None,
            textures: Textures::new(facade),
            board_program: {
                let vertex_shader_src = r#"
                    #version 330

                    in vec2 vert;
                    out vec2 v_vert;
                    uniform vec2 display_size;
    
                    void main() {
                        float board_x;
                        float board_y;
                        float board_w;
                        float board_h;
                        if (display_size.x > display_size.y) {
                            float pad = display_size.x - display_size.y;
                            board_x = 2.0 * (0.5 * pad / display_size.x) - 1.0;
                            board_y = 1.0;
                            board_w = 2.0 * (display_size.x - pad) / display_size.x;
                            board_h = -2.0;
                        } else {
                            float pad = display_size.y - display_size.x;
                            board_x = -1.0;
                            board_y = -2.0 * (0.5 * pad / display_size.y) + 1.0;
                            board_w = 2.0;
                            board_h = -2.0 * (display_size.y - pad) / display_size.y;
                        }

                        gl_Position = vec4(board_x + vert.x * board_w, board_y + vert.y * board_h, 0.0, 1.0);
                        v_vert = vert;
                    }
                "#;

                let fragment_shader_src = r#"
                    #version 330
                    
                    in vec2 v_vert;

                    out vec4 f_color;
    
                    void main() {
                        int c = 0;
                        if (mod(v_vert.x * 8, 2) < 1) {
                            c ++;
                        }
                        if (mod(v_vert.y * 8, 2) < 1) {
                            c ++;
                        }
                        if (c == 1)  {
                            f_color = vec4(0.6, 0.3, 0.05, 1.0);
                        } else {
                            f_color = vec4(0.9, 0.5, 0.15, 1.0);
                        }
                    }
                "#;

                glium::Program::from_source(facade, vertex_shader_src, fragment_shader_src, None)
                    .unwrap()
            },
            texture_program: {
                let vertex_shader_src = r#"
                    #version 330

                    in vec2 vert;
                    out vec2 v_vert;
                    uniform vec2 display_size;
                    uniform vec2 square;
    
                    void main() {
                        float board_x;
                        float board_y;
                        float board_w;
                        float board_h;
                        if (display_size.x > display_size.y) {
                            float pad = display_size.x - display_size.y;
                            board_x = 2.0 * (0.5 * pad / display_size.x) - 1.0;
                            board_y = 1.0;
                            board_w = 2.0 * (display_size.x - pad) / display_size.x;
                            board_h = -2.0;
                        } else {
                            float pad = display_size.y - display_size.x;
                            board_x = -1.0;
                            board_y = -2.0 * (0.5 * pad / display_size.y) + 1.0;
                            board_w = 2.0;
                            board_h = -2.0 * (display_size.y - pad) / display_size.y;
                        }

                        gl_Position = vec4(board_x + board_w * vert.x / 8.0 + square.x * board_w / 8.0, board_y + board_h * vert.y / 8.0 + square.y * board_h / 8.0, 0.0, 1.0);
                        v_vert = vert;
                    }
                "#;

                let fragment_shader_src = r#"
                    #version 330
                    
                    in vec2 v_vert;

                    uniform sampler2D tex;

                    out vec4 f_color;
    
                    void main() {
                        f_color = texture(tex, vec2(v_vert.x, 1 - v_vert.y));
                    }
                "#;

                glium::Program::from_source(facade, vertex_shader_src, fragment_shader_src, None)
                    .unwrap()
            },
            highlight_program: {
                let vertex_shader_src = r#"
                    #version 330

                    in vec2 vert;
                    out vec2 v_vert;
                    uniform vec2 display_size;
                    uniform vec2 square;
    
                    void main() {
                        float board_x;
                        float board_y;
                        float board_w;
                        float board_h;
                        if (display_size.x > display_size.y) {
                            float pad = display_size.x - display_size.y;
                            board_x = 2.0 * (0.5 * pad / display_size.x) - 1.0;
                            board_y = 1.0;
                            board_w = 2.0 * (display_size.x - pad) / display_size.x;
                            board_h = -2.0;
                        } else {
                            float pad = display_size.y - display_size.x;
                            board_x = -1.0;
                            board_y = -2.0 * (0.5 * pad / display_size.y) + 1.0;
                            board_w = 2.0;
                            board_h = -2.0 * (display_size.y - pad) / display_size.y;
                        }

                        gl_Position = vec4(board_x + board_w * vert.x / 8.0 + square.x * board_w / 8.0, board_y + board_h * vert.y / 8.0 + square.y * board_h / 8.0, 0.0, 1.0);
                        v_vert = vert;
                    }
                "#;

                let fragment_shader_src = r#"
                    #version 330
                    
                    in vec2 v_vert;

                    uniform vec3 colour;

                    out vec4 f_color;
    
                    void main() {
                        f_color = vec4(colour, 0.5);
                    }
                "#;

                glium::Program::from_source(facade, vertex_shader_src, fragment_shader_src, None)
                    .unwrap()
            },
        }
    }

    fn tick(&mut self, state: &crate::graphical::State, dt: f64) {}

    fn draw(&mut self, state: &crate::graphical::State, display: &glium::Display) {
        let mut target = display.draw();
        target.clear_color(0.0, 0.3, 0.0, 1.0);

        {
            #[derive(Copy, Clone)]
            struct Vertex {
                vert: [f32; 2],
            }
            implement_vertex!(Vertex, vert);

            let shape = vec![
                Vertex { vert: [0.0, 0.0] },
                Vertex { vert: [0.0, 1.0] },
                Vertex { vert: [1.0, 1.0] },
                Vertex { vert: [1.0, 0.0] },
            ];

            let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
            let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);

            target
                .draw(
                    &vertex_buffer,
                    &indices,
                    &self.board_program,
                    // &glium::uniforms::EmptyUniforms,
                    &uniform! {
                        display_size : (state.display_size.0 as f32, state.display_size.1 as f32),
                    },
                    &Default::default(),
                )
                .unwrap();
        }

        {
            #[derive(Copy, Clone)]
            struct Vertex {
                vert: [f32; 2],
            }
            implement_vertex!(Vertex, vert);

            let shape = vec![
                Vertex { vert: [0.0, 0.0] },
                Vertex { vert: [0.0, 1.0] },
                Vertex { vert: [1.0, 1.0] },
                Vertex { vert: [1.0, 0.0] },
            ];

            let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
            let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);

            for (sq_idx, piece) in self.board.get_pieces() {
                let sq = classical::sq_to_grid(sq_idx);
                let tex = match piece {
                    Piece {
                        team: Team::White,
                        kind: PieceKind::Pawn,
                        ..
                    } => &self.textures.white_pawn,
                    Piece {
                        team: Team::White,
                        kind: PieceKind::Rook,
                        ..
                    } => &self.textures.white_rook,
                    Piece {
                        team: Team::White,
                        kind: PieceKind::Knight,
                        ..
                    } => &self.textures.white_knight,
                    Piece {
                        team: Team::White,
                        kind: PieceKind::Bishop,
                        ..
                    } => &self.textures.white_bishop,
                    Piece {
                        team: Team::White,
                        kind: PieceKind::Queen,
                        ..
                    } => &self.textures.white_queen,
                    Piece {
                        team: Team::White,
                        kind: PieceKind::King,
                        ..
                    } => &self.textures.white_king,
                    Piece {
                        team: Team::Black,
                        kind: PieceKind::Pawn,
                        ..
                    } => &self.textures.black_pawn,
                    Piece {
                        team: Team::Black,
                        kind: PieceKind::Rook,
                        ..
                    } => &self.textures.black_rook,
                    Piece {
                        team: Team::Black,
                        kind: PieceKind::Knight,
                        ..
                    } => &self.textures.black_knight,
                    Piece {
                        team: Team::Black,
                        kind: PieceKind::Bishop,
                        ..
                    } => &self.textures.black_bishop,
                    Piece {
                        team: Team::Black,
                        kind: PieceKind::Queen,
                        ..
                    } => &self.textures.black_queen,
                    Piece {
                        team: Team::Black,
                        kind: PieceKind::King,
                        ..
                    } => &self.textures.black_king,
                };
                target
                .draw(
                    &vertex_buffer,
                    &indices,
                    &self.texture_program,
                    // &glium::uniforms::EmptyUniforms,
                    &uniform! {
                        display_size : (state.display_size.0 as f32, state.display_size.1 as f32),
                        square : (sq.0 as f32, sq.1 as f32),
                        tex: tex,
                    },
                    &glium::DrawParameters {
                        blend: glium::Blend::alpha_blending(),
                        ..Default::default()
                    },
                )
                .unwrap();
            }
        }

        {
            #[derive(Copy, Clone)]
            struct Vertex {
                vert: [f32; 2],
            }
            implement_vertex!(Vertex, vert);

            let shape = vec![
                Vertex { vert: [0.0, 0.0] },
                Vertex { vert: [0.0, 1.0] },
                Vertex { vert: [1.0, 1.0] },
                Vertex { vert: [1.0, 0.0] },
            ];

            let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
            let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);

            match self.selected {
                Some(square) => {
                    target
                    .draw(
                        &vertex_buffer,
                        &indices,
                        &self.highlight_program,
                        // &glium::uniforms::EmptyUniforms,
                        &uniform! {
                            display_size : (state.display_size.0 as f32, state.display_size.1 as f32),
                            square : (square.0 as f32, square.1 as f32),
                            colour : (0.0f32, 1.0f32, 0.0f32),
                        },
                        &glium::DrawParameters {
                            blend: glium::Blend::alpha_blending(),
                            ..Default::default()
                        },
                    )
                    .unwrap();
                }
                None => {}
            }

            for move_button in &self.move_buttons {
                target
                    .draw(
                        &vertex_buffer,
                        &indices,
                        &self.highlight_program,
                        // &glium::uniforms::EmptyUniforms,
                        &uniform! {
                            display_size : (state.display_size.0 as f32, state.display_size.1 as f32),
                            square : (move_button.pos.0 as f32, move_button.pos.1 as f32),
                            colour : move_button.colour,
                        },
                        &glium::DrawParameters {
                            blend: glium::Blend::alpha_blending(),
                            ..Default::default()
                        },
                    )
                    .unwrap();
            }

            match self.board_ai.as_ref().unwrap().current_best_move() {
                Some(m_idx) => {
                    let m = self.moves[m_idx.idx];
                    let squares = match m {
                        Move::Standard { from_sq, to_sq, .. } => vec![from_sq, to_sq],
                    };
                    for square in squares.iter().map(|s| sq_to_grid(*s)) {
                        target
                    .draw(
                        &vertex_buffer,
                        &indices,
                        &self.highlight_program,
                        // &glium::uniforms::EmptyUniforms,
                        &uniform! {
                            display_size : (state.display_size.0 as f32, state.display_size.1 as f32),
                            square : (square.0 as f32, square.1 as f32),
                            colour : (1.0f32, 0.2f32, 0.0f32),
                        },
                        &glium::DrawParameters {
                            blend: glium::Blend::alpha_blending(),
                            ..Default::default()
                        },
                    )
                    .unwrap();
                    }
                }
                None => {}
            }
        }

        target.finish().unwrap();
    }

    fn event(
        &mut self,
        interface_state: &crate::graphical::State,
        ev: &glium::glutin::event::Event<'_, ()>,
    ) {
        match ev {
            glium::glutin::event::Event::DeviceEvent { device_id, event } => match event {
                glium::glutin::event::DeviceEvent::Button { button, state } => {
                    match (button, state) {
                        (1, ElementState::Pressed) => {
                            match self.pixel_to_square(interface_state, interface_state.mouse_pos) {
                                Some(clicked) => {
                                    let mut move_idx_opt = None;

                                    for move_button in self.move_buttons.iter() {
                                        if move_button.pos == clicked {
                                            move_idx_opt = Some(move_button.move_idx);
                                        }
                                    }

                                    match move_idx_opt {
                                        Some(move_idx) => {
                                            self.make_move(move_idx);
                                        }
                                        None => {
                                            match self
                                                .board
                                                .get_square(grid_to_sq(clicked.0, clicked.1))
                                            {
                                                Some(piece) => {
                                                    if piece.team == self.board.get_turn() {
                                                        {
                                                            self.set_selected(Some(clicked));
                                                        }
                                                    } else {
                                                        {
                                                            self.set_selected(None);
                                                        }
                                                    }
                                                }
                                                None => {
                                                    self.set_selected(None);
                                                }
                                            }
                                        }
                                    }
                                }
                                None => {
                                    self.set_selected(None);
                                }
                            }
                        }
                        _ => {}
                    };
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl GameInterface {
    fn set_selected(&mut self, selected: Option<(u8, u8)>) {
        self.selected = selected;
        self.move_buttons = vec![];
        match self.selected {
            Some(pos) => {
                let sq = grid_to_sq(pos.0, pos.1);
                for (m_idx, m) in self
                    .moves
                    .iter()
                    .enumerate()
                    .map(|(idx, m)| (MoveIdx { idx }, *m))
                {
                    match m {
                        Move::Standard {
                            piece,
                            victim: victim_opt,
                            promotion: promotion_opt,
                            from_sq,
                            to_sq,
                        } => {
                            if grid_to_sq(pos.0, pos.1) == from_sq {
                                self.move_buttons.push(MoveButton {
                                    pos: sq_to_grid(to_sq),
                                    colour: match (victim_opt) {
                                        Some(_) => (1.0, 0.0, 0.0),
                                        None => (0.0, 0.5, 1.0),
                                    },
                                    move_idx: m_idx,
                                });
                            }
                        }
                    }
                }
            }
            None => {}
        }
    }

    fn make_move(&mut self, m: MoveIdx) {
        self.set_selected(None);
        let (mut ai_off, best_move) = self.board_ai.take().unwrap().finish();
        ai_off.make_move(m);
        self.board = ai_off.get_board().clone();
        self.moves = ai_off.get_moves();
        self.board_ai = Some(ai_off.start());
    }
}
