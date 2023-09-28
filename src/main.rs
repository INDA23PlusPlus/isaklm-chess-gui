use chess::chess;
use ggez::event;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use ggez::ContextBuilder;
use ggez::glam::*;

const BOARD_SIZE: i32 = 8;
const SQUARE_SIZE: i32 = 96;
const WINDOW_WIDTH: i32 = 768;
const WINDOW_HEIGHT: i32 = 768;

struct Images
{
    board: graphics::Image,
    pawn: graphics::Image,
    rook: graphics::Image,
    knight: graphics::Image,
    bishop: graphics::Image,
    queen: graphics::Image,
    king: graphics::Image
}

#[derive(Clone, Copy)]
struct BoardPosition
{
    x: i32,
    y: i32
}

struct MainState
{
    game: chess::Game,
    images: Images,
    has_selected_piece: bool,
    piece_position: BoardPosition,
    flip_board: bool
}

impl MainState
{
    fn new(ctx: &Context) -> GameResult<MainState>
    {
        let state = MainState { 
            game: chess::Game::new(chess::Board::new(chess::BOARD_DEFAULT_SETUP)),
            images: Images{
                board: graphics::Image::from_path(ctx, "/board.png")?,
                pawn: graphics::Image::from_path(ctx, "/pawn.png")?,
                rook: graphics::Image::from_path(ctx, "/rook.png")?,
                knight: graphics::Image::from_path(ctx, "/knight.png")?,
                bishop: graphics::Image::from_path(ctx, "/bishop.png")?,
                queen: graphics::Image::from_path(ctx, "/queen.png")?,
                king: graphics::Image::from_path(ctx, "/king.png")?
            },
            has_selected_piece: false,
            piece_position: BoardPosition { x: 0, y: 0 },
            flip_board: true
         };
        Ok(state)
    }
}

impl event::EventHandler<ggez::GameError> for MainState
{
    fn update(&mut self, _ctx: &mut Context) -> GameResult
    {
        //if _ctx.keyboard.is_key_just_pressed(ggez::input::keyboard::KeyCode::W) // event::MouseButton::Right
        if _ctx.mouse.button_just_pressed(ggez::input::mouse::MouseButton::Left) && !checkmate(&mut self.game)
        {
            let position = _ctx.mouse.position();

            let mut board_position = BoardPosition{ x: (position.x / (SQUARE_SIZE as f32)) as i32, y: (position.y / (SQUARE_SIZE as f32)) as i32 };

            if self.flip_board
            {
                board_position.y = vertical_flip(board_position.y);
            }


            if board_position.x >= 0 && board_position.x < BOARD_SIZE && board_position.y >= 0 && board_position.y < BOARD_SIZE
            {
                if self.has_selected_piece
                {
                    if board_position.x == self.piece_position.x && board_position.y == self.piece_position.y
                    {
                        self.has_selected_piece = false;
                    }
                    else
                    {
                        if self.game.try_make_move(self.piece_position.x as usize, self.piece_position.y as usize, board_position.x as usize, board_position.y as usize)
                        {
                            self.has_selected_piece = false;
                            self.flip_board = !self.flip_board;
                        }
                    }
                }
                else
                {
                    if let Some(piece) = self.game.get_piece(board_position.x as usize, board_position.y as usize)
                    {
                        if piece.piece_color() == self.game.player_to_move()
                        {
                            self.piece_position = board_position.clone();
                            self.has_selected_piece = true;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult
    {
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            graphics::Color::from([0.0, 0.0, 0.0, 1.0]),
        );


        draw_board(&mut canvas, self);


        if self.has_selected_piece
        {
            {
                let highlight_color = Color{ r: 1.0, g: 1.0, b: 1.0, a: 0.4 };
                let position = board_to_window_position(self.piece_position.x, self.piece_position.y, self.flip_board);
                let border = ggez::graphics::Rect::new(position.x, position.y, SQUARE_SIZE as f32, SQUARE_SIZE as f32);
                let square = ggez::graphics::Mesh::new_rectangle(ctx, ggez::graphics::DrawMode::fill(), border, highlight_color)?;
        
                canvas.draw(&square, graphics::DrawParam::default()); // highlighting selected piece
            }

            
            let moves = self.game.get_legal_moves(self.piece_position.x as usize, self.piece_position.y as usize);

            for y in 0..BOARD_SIZE
            {
                for x in 0..BOARD_SIZE
                {
                    if moves.contains(&(x as usize, y as usize))
                    {
                        let highlight_color = Color{ r: 1.0, g: 1.0, b: 0.0, a: 0.2 };
                        let position = board_to_window_position(x, y, self.flip_board);
                        let border = ggez::graphics::Rect::new(position.x, position.y, SQUARE_SIZE as f32, SQUARE_SIZE as f32);
                        let square = ggez::graphics::Mesh::new_rectangle(ctx, ggez::graphics::DrawMode::fill(), border, highlight_color)?;
                
                        canvas.draw(&square, graphics::DrawParam::default()); // highlighting possible moves
                    }
                }
            }
        }

        
        if checkmate(&mut self.game)
        {
            let mut text = ggez::graphics::Text::new("Checkmate!");

            text.set_scale(ggez::graphics::PxScale::from(100.0));

            let text_size = text.measure(ctx)?;

            let position = [(WINDOW_WIDTH as f32 - text_size.x) / 2.0, (WINDOW_HEIGHT as f32 - text_size.y) / 2.0];

            canvas.draw(&text, ggez::graphics::DrawParam::default().dest(position)); // renders checkmate text
        }


        canvas.finish(ctx)?;
        Ok(())
    }
}

fn draw_board(canvas: &mut graphics::Canvas, state: &mut MainState)
{
    canvas.draw(&state.images.board, graphics::DrawParam::default());

    for y in 0..BOARD_SIZE
    {
        for x in 0..BOARD_SIZE
        {
            if let Some(piece) = state.game.get_piece(x as usize, y as usize)
            {
                let image = match piece.piece_type()
                {
                    chess::PieceType::Pawn => &state.images.pawn,
                    chess::PieceType::Rook => &state.images.rook,
                    chess::PieceType::Knight => &state.images.knight,
                    chess::PieceType::Bishop => &state.images.bishop,
                    chess::PieceType::Queen => &state.images.queen,
                    chess::PieceType::King => &state.images.king
                };


                let piece_color;

                if piece.piece_color() == chess::Color::White
                {
                    piece_color = Color{ r: 0.988, g: 0.847, b: 0.435, a: 1.0 };
                }
                else
                {
                    piece_color = Color{ r: 0.494, g: 0.423, b: 0.217, a: 1.0 };
                }


                let position = board_to_window_position(x, y, state.flip_board);

                canvas.draw(image, graphics::DrawParam::default().dest(position).color(piece_color));
            }
        }
    }
}

fn board_to_window_position(x: i32, y: i32, flip_board: bool) -> Vec2
{
    let mut new_y = y;

    if flip_board
    {
        new_y = vertical_flip(y);
    }

    return Vec2::new((x * SQUARE_SIZE) as f32, (new_y * SQUARE_SIZE) as f32);
}

fn vertical_flip(y: i32) -> i32
{
    return BOARD_SIZE - 1 - y;
}

fn checkmate(game: &mut chess::Game) -> bool // checkmate in library doesn't work, so I implemented it myself
{
    for y in 0..BOARD_SIZE
    {
        for x in 0..BOARD_SIZE
        {
            if game.get_piece(x as usize, y as usize).is_some()
            {
                let moves = game.get_legal_moves(x as usize, y as usize);

                if !moves.is_empty()
                {
                    return false;
                }
            }
        }
    }

    return true;
}

pub fn main() -> GameResult
{
    let (ctx, event_loop) = ContextBuilder::new("chess-gui", "Isak Livner Mäkitalo")
    .add_resource_path("./resources")
    .window_setup(ggez::conf::WindowSetup::default().title("isaklm-chess-gui"))
    .window_mode(ggez::conf::WindowMode::default().dimensions(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32))
    .build()?;

    let state = MainState::new(&ctx)?;

    event::run(ctx, event_loop, state)
}
