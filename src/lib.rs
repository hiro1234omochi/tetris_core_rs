#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;
use MinoDirection::*;
use MinoType::*;
use core::hash::Hash;
use enum_map::{Enum, EnumMap, enum_map};
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(not(feature = "alloc"))]
use heapless::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "alloc"))]
pub const NO_ALLOC_MINO_QUEUE_CAPACITY: usize = 128;
#[cfg(not(feature = "alloc"))]
pub const NO_ALLOC_ATTACK_LINE_CAPACITY: usize = 256;
pub const DEFAULT_BOARD_SIZE: (usize, usize) = (10, 42);

#[cfg(feature = "alloc")]
type Field = Vec<Vec<Cell>>;
#[cfg(not(feature = "alloc"))]
type Field = Vec<Vec<Cell, { DEFAULT_BOARD_SIZE.0 }>, { DEFAULT_BOARD_SIZE.1 }>;
#[cfg(not(feature = "alloc"))]
fn create_empty_field() -> Field {
    let mut field: Field = Vec::new();
    let mut field_child = Vec::new();
    for _ in 0..DEFAULT_BOARD_SIZE.0 {
        field_child.push(Cell::Empty);
    }
    for _ in 0..DEFAULT_BOARD_SIZE.1 {
        field.push(field_child.clone());
    }
    field
}
#[cfg(feature = "alloc")]
type FieldWithNextMinoWillSpawn = Vec<Vec<(bool, Cell)>>;
#[cfg(not(feature = "alloc"))]
type FieldWithNextMinoWillSpawn =
    Vec<Vec<(bool, Cell), { DEFAULT_BOARD_SIZE.0 }>, { DEFAULT_BOARD_SIZE.1 }>;

#[cfg(feature = "alloc")]
type NextsField = Vec<MinoType>;
#[cfg(not(feature = "alloc"))]
type NextsField = heapless::Vec<MinoType, NO_ALLOC_MINO_QUEUE_CAPACITY>;

#[cfg(feature = "alloc")]
type AttackedLines = Vec<AttackedLine>;
#[cfg(not(feature = "alloc"))]
type AttackedLines = heapless::Vec<AttackedLine, NO_ALLOC_ATTACK_LINE_CAPACITY>;

type OffsetsType = EnumMap<MinoDirection, EnumMap<RotationType, &'static [(i64, i64)]>>;
#[allow(clippy::complexity)]
static ROTATIONS: Lazy<EnumMap<MinoType, EnumMap<MinoDirection, [[i64; 4]; 4]>>> =
    Lazy::new(|| {
        enum_map! {
            MinoT => enum_map! {
                North => [[0, 1, 0, 0], [1, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
                East => [[0, 1, 0, 0], [0, 1, 1, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
                South=>[[0, 0, 0, 0], [1, 1, 1, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
                West=>[[0, 1, 0, 0], [1, 1, 0, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
            },
            MinoS=>enum_map! {
                North=>[[0, 1, 1, 0], [1, 1, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
                East=>[[0, 1, 0, 0], [0, 1, 1, 0], [0, 0, 1, 0], [0, 0, 0, 0]],
                South=>[[0, 0, 0, 0], [0, 1, 1, 0], [1, 1, 0, 0], [0, 0, 0, 0]],
                West=>[[1, 0, 0, 0], [1, 1, 0, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
            },
            MinoZ=>enum_map!{
                North=>[[1, 1, 0, 0], [0, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
                East=>[[0, 0, 1, 0], [0, 1, 1, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
                South=>[[0, 0, 0, 0], [1, 1, 0, 0], [0, 1, 1, 0], [0, 0, 0, 0]],
                West=>[[0, 1, 0, 0], [1, 1, 0, 0], [1, 0, 0, 0], [0, 0, 0, 0]],
            },
            MinoL=>enum_map! {
                North=>[[0, 0, 1, 0], [1, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
                East=>[[0, 1, 0, 0], [0, 1, 0, 0], [0, 1, 1, 0], [0, 0, 0, 0]],
                South=>[[0, 0, 0, 0], [1, 1, 1, 0], [1, 0, 0, 0], [0, 0, 0, 0]],
                West=>[[1, 1, 0, 0], [0, 1, 0, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
            },
            MinoJ=>enum_map!{
                North=>[[1, 0, 0, 0], [1, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
                East=>[[0, 1, 1, 0], [0, 1, 0, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
                South=>[[0, 0, 0, 0], [1, 1, 1, 0], [0, 0, 1, 0], [0, 0, 0, 0]],
                West=>[[0, 1, 0, 0], [0, 1, 0, 0], [1, 1, 0, 0], [0, 0, 0, 0]],
            },
            MinoO=>enum_map!{
                North=>[[0, 0, 0, 0], [0, 1, 1, 0], [0, 1, 1, 0], [0, 0, 0, 0]],
                East=>[[0, 0, 0, 0], [0, 1, 1, 0], [0, 1, 1, 0], [0, 0, 0, 0]],
                South=>[[0, 0, 0, 0], [0, 1, 1, 0], [0, 1, 1, 0], [0, 0, 0, 0]],
                West=>[[0, 0, 0, 0], [0, 1, 1, 0], [0, 1, 1, 0], [0, 0, 0, 0]],
            },
            MinoI=>enum_map!{
                North=>[[0, 0, 0, 0], [1, 1, 1, 1], [0, 0, 0, 0], [0, 0, 0, 0]],
                East=>[[0, 0, 1, 0], [0, 0, 1, 0], [0, 0, 1, 0], [0, 0, 1, 0]],
                South=>[[0, 0, 0, 0], [0, 0, 0, 0], [1, 1, 1, 1], [0, 0, 0, 0]],
                West=>[[0, 1, 0, 0], [0, 1, 0, 0], [0, 1, 0, 0], [0, 1, 0, 0]],
            }
        }
    });

static OFFSETS: Lazy<OffsetsType> = Lazy::new(|| {
    enum_map! {
        North=>enum_map!{
                RotationType::Clockwise=>[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)].as_slice(),
                RotationType::CounterClockwise=>[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)].as_slice(),
                RotationType::Rotate180=>[(1, 0),(2, 0),(1, 1),(2, 1),(-1, 0),(-2, 0),(-1, 1),(-2, 1),(0, -1),(3, 0),(-3, 0)].as_slice(),
        },
        East=>enum_map!{
                RotationType::Clockwise=>[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)].as_slice(),
                RotationType::CounterClockwise=>[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)].as_slice(),
                RotationType::Rotate180=> [(0, 1), (0, 2), (-1, 1), (-1, 2), (0, -1), (0, -2), (-1, -1), (-1, -2), (1, 0), (0, 3), (0, -3),].as_slice(),
        },
        South=>enum_map!{
            RotationType::Clockwise=>[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)].as_slice(),
            RotationType::CounterClockwise=>[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)].as_slice(),
            RotationType::Rotate180=>[(-1, 0),(-2, 0),(-1, -1),(-2, -1),(1, 0),(2, 0), (1, -1),(2, -1),(0, 1),(-3, 0),(3, 0)].as_slice(),
        },
        West=>enum_map!{
            RotationType::Clockwise=>[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)].as_slice(),
            RotationType::CounterClockwise=>[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)].as_slice(),
            RotationType::Rotate180=>[(0, 1),(0, 2),(1, 1),(1, 2),(0, -1),(0, -2),(1, -1),(1, -2),(-1, 0),(0, 3),(0, -3)].as_slice(),
        }
    }
});

static OFFSETS_MINO_I: Lazy<OffsetsType> = Lazy::new(|| {
    enum_map! {
        North=>enum_map!{
            RotationType::Clockwise=>[(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)].as_slice(),
            RotationType::CounterClockwise=>[(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)].as_slice(),
            RotationType::Rotate180=>[(-1, 0), (-2, 0), (1, 0), (2, 0), (0, 1)].as_slice(),
        },
        East=>enum_map!{
            RotationType::Clockwise=>[(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)].as_slice(),
            RotationType::CounterClockwise=>[(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)].as_slice(),
            RotationType::Rotate180=>[(0, 1), (0, 2), (0, -1), (0, -2), (-1, 0)].as_slice(),
        },
        South=>enum_map! {
            RotationType::Clockwise=>[(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)].as_slice(),
            RotationType::CounterClockwise=>[(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)].as_slice(),
            RotationType::Rotate180=>[(1, 0), (2, 0), (-1, 0), (-2, 0), (0, -1)].as_slice(),
        },
        West=>enum_map! {
            RotationType::Clockwise=> [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)].as_slice(),
            RotationType::CounterClockwise=>[(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)].as_slice(),
            RotationType::Rotate180=>[(0, 1), (0, 2), (0, -1), (0, -2), (1, 0)].as_slice(),
        },
    }
});

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug, Enum)]
enum MinoDirection {
    North,
    East,
    South,
    West,
}
impl MinoDirection {
    // 次の方向を返す
    fn next_direction(&self) -> Self {
        match self {
            North => East,
            East => South,
            South => West,
            West => North,
        }
    }
    fn rotate(&self, rotation_type: RotationType) -> Self {
        match rotation_type {
            RotationType::Clockwise => self.next_direction(),
            RotationType::Rotate180 => self.next_direction().next_direction(),
            RotationType::CounterClockwise => {
                self.next_direction().next_direction().next_direction()
            }
        }
    }
}
#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug, Enum)]
enum RotationType {
    Clockwise,        // 時計回り
    Rotate180,        // 180度
    CounterClockwise, // 反時計回り
}
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug, Enum)]
pub enum MinoType {
    MinoI,
    MinoO,
    MinoS,
    MinoZ,
    MinoJ,
    MinoL,
    MinoT,
}
impl MinoType {
    pub fn get_field_to_draw(&self) -> [[Cell; 4]; 4] {
        ROTATIONS[*self][North].map(|row| {
            row.map(|cell| {
                if cell == 0 {
                    Cell::Empty
                } else {
                    Cell::MinoBlock(*self)
                }
            })
        })
    }
}
const MINO_ARRAY: [MinoType; 7] = [MinoI, MinoO, MinoS, MinoZ, MinoJ, MinoL, MinoT];
#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum HorizontalDirection {
    Left,
    Right,
}
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "alloc", derive(Eq, PartialEq))]
#[derive(Clone, Debug)]
struct MinoQueue {
    #[cfg(feature = "alloc")]
    queue: alloc::collections::VecDeque<MinoType>,
    #[cfg(not(feature = "alloc"))]
    queue: heapless::Deque<MinoType, NO_ALLOC_MINO_QUEUE_CAPACITY>,
    hold: Option<MinoType>,
    current: MinoType,
    rng: rand_chacha::ChaCha20Rng,
}
#[cfg(not(feature = "alloc"))]
impl core::cmp::PartialEq for MinoQueue {
    fn eq(&self, other: &Self) -> bool {
        self.hold == other.hold
            && self
                .queue
                .iter()
                .copied()
                .collect::<Vec<MinoType, NO_ALLOC_MINO_QUEUE_CAPACITY>>()
                == other
                    .queue
                    .iter()
                    .copied()
                    .collect::<Vec<MinoType, NO_ALLOC_MINO_QUEUE_CAPACITY>>()
            && self.hold == other.hold
            && self.current == other.current
            && self.rng == other.rng
    }
}
#[cfg(not(feature = "alloc"))]
impl core::cmp::Eq for MinoQueue {}
impl Default for MinoQueue {
    fn default() -> Self {
        #[cfg(feature = "alloc")]
        let queue = alloc::collections::VecDeque::new();

        #[cfg(not(feature = "alloc"))]
        let queue = heapless::Deque::new();
        Self {
            rng: rand_seeder::Seeder::from("test").into_rng(),
            hold: None,
            queue,
            current: MinoO,
        }
    }
}
impl MinoQueue {
    pub fn new(rng_seed: &impl Hash) -> Self {
        let rng = rand_seeder::Seeder::from(rng_seed).into_rng();
        let mut mino_queue = Self {
            rng,
            ..Default::default()
        };
        mino_queue.next();
        mino_queue
    }
    pub fn next(&mut self) {
        self.generate_if_needed(1);
        self.current = self.queue.pop_front().unwrap();
    }
    pub fn get_next_minos(&mut self, num: usize) -> NextsField {
        self.generate_if_needed(num);
        self.queue.iter().copied().take(num).collect()
    }
    fn hold(&mut self) {
        if let Some(hold) = self.hold {
            (self.current, self.hold) = (hold, Some(self.current));
        } else {
            self.hold = Some(self.current);
            self.next();
        }
    }
    fn generate_if_needed(&mut self, required_item_num: usize) {
        if required_item_num > self.queue.len() {
            #[cfg(not(feature = "alloc"))]
            if MINO_ARRAY.len() + self.queue.len() > self.queue.capacity() {
                panic!("too many!");
            }
            let mut mino_array = MINO_ARRAY;
            mino_array.shuffle(&mut self.rng);
            for mino in mino_array {
                self.queue.push_back(mino);
            }
            self.generate_if_needed(required_item_num);
        }
    }
    pub fn get_hold(&self) -> Option<MinoType> {
        self.hold
    }
    pub fn get_current(&self) -> MinoType {
        self.current
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum Cell {
    Empty,
    Wall,
    Obstruction(bool), //クリアされるべきならtrue, クリアできないならfalse
    MinoBlock(MinoType),
    MinoInMotion(MinoType),
    Ghost(MinoType),
}

impl Cell {
    fn has_collision(&self) -> bool {
        matches!(self, Self::Wall | Self::Obstruction(_) | Self::MinoBlock(_))
    }
    fn can_be_cleared(&self) -> bool {
        matches!(
            self,
            Self::Wall | Self::Obstruction(true) | Self::MinoBlock(_)
        )
    }
}
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Mino {
    x: i64,
    y: i64,
    maximum_y: i64,
    mino_direction: MinoDirection,
    mino_type: MinoType,
    does_rotate: bool,
    rotation: EnumMap<MinoDirection, [[i64; 4]; 4]>,
    move_reset_count: usize,
    mino_state: MinoState,
    should_be_locked: bool,
    is_last_move_spin: bool,
    is_last_move_mini_spin: bool,
}
impl Default for Mino {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            move_reset_count: 0,
            maximum_y: 0,
            mino_direction: North,
            mino_type: MinoO,
            does_rotate: false,
            rotation: ROTATIONS[MinoO],
            mino_state: MinoState::AirBorne,
            should_be_locked: false,
            is_last_move_spin: false,
            is_last_move_mini_spin: false,
        }
    }
}

impl Mino {
    pub fn new(mino_type: MinoType, x: i64, y: i64, field: &Field) -> Result<Self, ()> {
        let mut template = Self {
            x,
            y,
            maximum_y: y,
            mino_type,
            rotation: ROTATIONS[mino_type],
            ..Default::default()
        };
        if !template.can_replace(x, y, template.mino_direction, field) {
            return Err(());
        }
        template.mino_state = if template.can_down(field) {
            MinoState::AirBorne
        } else {
            MinoState::JustLanded
        };
        Ok(match mino_type {
            MinoO => template,
            _ => Self {
                does_rotate: true,
                ..template
            },
        })
    }
    fn new_for_preview_next_mino(mino_type: MinoType, x: i64, y: i64) -> Self {
        Self {
            x,
            y,
            maximum_y: y,
            mino_type,
            rotation: ROTATIONS[mino_type],
            ..Default::default()
        }
    }
    pub fn horizontal_move(
        &mut self,
        horizontal_direction: HorizontalDirection,
        field: &Field,
        move_reset_limit: Option<usize>,
    ) -> bool {
        let r = self.replace(
            self.x
                + if horizontal_direction == HorizontalDirection::Left {
                    -1
                } else {
                    1
                },
            self.y,
            self.mino_direction,
            field,
            move_reset_limit,
        );
        if r {
            self.last_move_is_not_spin();
        }
        r
    }
    pub fn down(&mut self, field: &Field) -> bool {
        let r = self.replace(self.x, self.y + 1, self.mino_direction, field, None);
        if r {
            self.last_move_is_not_spin();
        }
        r
    }
    pub fn can_down(&self, field: &Field) -> bool {
        self.can_replace(self.x, self.y + 1, self.mino_direction, field)
    }
    pub fn rotate(
        &mut self,
        rotation_type: RotationType,
        field: &Field,
        move_reset_limit: Option<usize>,
    ) -> bool {
        let mut offsets: &[(i64, i64)] = if self.mino_type == MinoI {
            OFFSETS_MINO_I[self.mino_direction][rotation_type]
        } else {
            OFFSETS[self.mino_direction][rotation_type]
        };
        if !self.does_rotate {
            offsets = &[(0, 0)]; //move_resetとspinの処理だけしたい
        }
        for (offset_n, offset) in offsets.iter().enumerate() {
            if self.replace(
                self.x + offset.0,
                self.y - offset.1,
                self.mino_direction.rotate(rotation_type),
                field,
                move_reset_limit,
            ) {
                if self.mino_type == MinoT {
                    let center = (self.x + 1, self.y + 1);
                    let mut corner_count = 0;
                    for corner_offset_y in [-1, 1] {
                        for corner_offset_x in [-1, 1] {
                            let corner = (center.0 + corner_offset_x, center.1 + corner_offset_y);
                            if self.has_collision(field, corner.0, corner.1) {
                                corner_count += 1;
                            }
                        }
                    }
                    if corner_count >= 3 {
                        self.is_last_move_spin = true;

                        let certain_corner_offsets = match self.mino_direction {
                            North => [(0, 0), (2, 0)],
                            East => [(2, 0), (2, 2)],
                            South => [(0, 2), (2, 2)],
                            West => [(2, 0), (2, 2)],
                        };
                        let mut certain_corner_count = 0;
                        for certain_corner_offset in certain_corner_offsets {
                            let certain_corner = (
                                center.0 + certain_corner_offset.0,
                                center.1 + certain_corner_offset.1,
                            );
                            if self.has_collision(field, certain_corner.0, certain_corner.1) {
                                certain_corner_count += 1;
                            }
                        }
                        self.is_last_move_mini_spin = certain_corner_count != 2 // t-spin-mini
                    } else {
                        self.last_move_is_not_spin();
                    }
                } else {
                    self.last_move_is_not_spin();
                    //もしかしたら実装するかもしれません。
                }
                return true;
            }
        }
        false
    }

    fn can_replace(&self, x: i64, y: i64, mino_direction: MinoDirection, field: &Field) -> bool {
        for (iy, yy) in (y..y + 4).enumerate() {
            for (ix, xx) in (x..x + 4).enumerate() {
                if self.rotation[mino_direction][iy][ix] == 1 {
                    if yy < 0 || xx < 0 {
                        return false;
                    }
                    if field.get(yy as usize).is_none_or(|row| {
                        row.get(xx as usize).is_none_or(|cell| cell.has_collision()) //cellが存在しないか、衝突判定を持っているなら
                    }) {
                        return false;
                    }
                }
            }
        }
        true
    }
    fn replace(
        &mut self,
        x: i64,
        y: i64,
        mino_direction: MinoDirection,
        field: &Field,
        move_reset_limit: Option<usize>,
    ) -> bool {
        self.check_mino_status();
        if !self.can_replace(x, y, mino_direction, field) {
            return false;
        }
        if self.maximum_y < y {
            self.move_reset_count = 0;
            self.should_be_locked = false;
        } else {
            if let Some(move_reset_limit) = move_reset_limit {
                if move_reset_limit < self.move_reset_count {
                    self.should_be_locked = true;
                    if self.mino_state != MinoState::AirBorne {
                        return false;
                    }
                }
            }
            self.move_reset_count += 1;
        }
        self.x = x;
        self.y = y;
        self.mino_direction = mino_direction;
        self.maximum_y = core::cmp::max(self.maximum_y, self.y);
        self.mino_state = if self.can_down(field) {
            MinoState::AirBorne
        } else {
            MinoState::JustLanded
        };
        true
    }
    fn lock(&self, field: &mut Field) {
        self.draw(field, Cell::MinoBlock(self.mino_type));
    }
    fn draw(&self, field: &mut Field, cell: Cell) {
        for (iy, yy) in (self.y..self.y + 4).enumerate() {
            for (ix, xx) in (self.x..self.x + 4).enumerate() {
                if self.rotation[self.mino_direction][iy][ix] == 1 {
                    field[yy as usize][xx as usize] = cell;
                }
            }
        }
    }
    fn draw_ghost(&self, field: &mut Field) {
        let mut drawer = self.clone();
        {
            while drawer.down(field) {}
        }
        drawer.draw(field, Cell::Ghost(self.mino_type));
    }
    fn draw_next_mino(&self, field: &Field) -> FieldWithNextMinoWillSpawn {
        let mut r = FieldWithNextMinoWillSpawn::new();
        for (y, row) in field.iter().enumerate() {
            r.push(Vec::new());
            for &cell in row.iter() {
                r[y].push((false, cell));
            }
        }
        for (iy, yy) in (self.y..self.y + 4).enumerate() {
            for (ix, xx) in (self.x..self.x + 4).enumerate() {
                let yy = yy as usize;
                let xx = xx as usize;
                r[yy][xx] = (
                    self.rotation[self.mino_direction][iy][ix] == 1,
                    field[yy][xx],
                );
            }
        }
        r
    }
    fn last_move_is_not_spin(&mut self) {
        self.is_last_move_spin = false;
        self.is_last_move_mini_spin = false;
    }
    fn check_mino_status(&mut self) {
        if self.mino_state == MinoState::JustLanded {
            self.mino_state = MinoState::Grounded
        }
    }
    fn has_collision(&self, field: &Field, x: i64, y: i64) -> bool {
        if x < 0 || y < 0 {
            return false;
        }
        let (x, y) = (x as usize, y as usize);
        field
            .get(y)
            .is_none_or(|row| row.get(x).is_none_or(|cell| cell.has_collision()))
    }
}
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TetrisConfig {
    move_reset_limit: Option<usize>,
    appearance_position: (i64, i64), //MinoOのときだけy座標が-1される
    can_hold_infinity: bool,
}
impl Default for TetrisConfig {
    fn default() -> Self {
        Self {
            move_reset_limit: Some(15),
            appearance_position: (3, 19),
            can_hold_infinity: false,
        }
    }
}
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TetrisManager {
    width: usize,
    height: usize,
    field: Field,
    tetris_config: TetrisConfig,
    mino_queue: MinoQueue,
    current_mino: Mino,
    attacked_lines_stock: AttackedLines,
    has_held: bool,
}
impl Default for TetrisManager {
    fn default() -> Self {
        #[cfg(feature = "alloc")]
        let field =
            alloc::vec![alloc::vec![Cell::Empty; DEFAULT_BOARD_SIZE.0]; DEFAULT_BOARD_SIZE.1];
        #[cfg(not(feature = "alloc"))]
        let field = create_empty_field();
        let tetris_config = TetrisConfig::default();
        Self {
            width: DEFAULT_BOARD_SIZE.0,
            height: DEFAULT_BOARD_SIZE.1,
            field,
            mino_queue: MinoQueue::default(),
            current_mino: Mino::default(),
            tetris_config,
            attacked_lines_stock: AttackedLines::new(),
            has_held: false,
        }
    }
}
impl TetrisManager {
    #[cfg(feature = "alloc")]
    pub fn new(
        tetris_config: TetrisConfig,
        rng_seed: &impl Hash,
        width: usize,
        height: usize,
    ) -> Self {
        let mut tetris_manager = Self {
            width,
            height,
            field: alloc::vec![alloc::vec![Cell::Empty; width]; height],
            tetris_config,
            mino_queue: MinoQueue::new(rng_seed),
            ..Default::default()
        };
        tetris_manager.spawn_current_mino().unwrap();
        tetris_manager
    }
    #[cfg(not(feature = "alloc"))]
    pub fn new(tetris_config: TetrisConfig, rng_seed: &impl Hash) -> Self {
        let mut tetris_manager = Self {
            field: create_empty_field(),
            tetris_config,
            mino_queue: MinoQueue::new(rng_seed),
            ..Default::default()
        };
        tetris_manager.spawn_current_mino().unwrap();
        tetris_manager
    }
    pub fn get_field(&self) -> Field {
        self.field.clone()
    }
    pub fn get_field_to_draw(&self) -> Field {
        let mut field = self.field.clone();
        self.current_mino.draw_ghost(&mut field);
        self.current_mino
            .draw(&mut field, Cell::MinoInMotion(self.mino_queue.current));
        field
    }
    pub fn get_field_to_draw_with_preview_next_mino(&mut self) -> FieldWithNextMinoWillSpawn {
        let next_mino_type = self.mino_queue.get_next_minos(1)[0];
        let next_pos = self.get_spawn_point(next_mino_type);
        let drawer = Mino::new_for_preview_next_mino(next_mino_type, next_pos.0, next_pos.1);
        drawer.draw_next_mino(&self.get_field_to_draw())
    }
    fn lock_check(&mut self) -> Option<(Result<MinoState, ()>, Option<LineClear>)> {
        if self.current_mino.should_be_locked && self.current_mino.mino_state != MinoState::AirBorne
        {
            return Some(self.command(MovementCommand::Lock));
        }
        None
    }
    pub fn command(
        &mut self,
        movement_command: MovementCommand,
    ) -> (Result<MinoState, ()>, Option<LineClear>) {
        match movement_command {
            MovementCommand::Left => {
                self.current_mino.horizontal_move(
                    HorizontalDirection::Left,
                    &self.field,
                    self.tetris_config.move_reset_limit,
                );
            }
            MovementCommand::Right => {
                self.current_mino.horizontal_move(
                    HorizontalDirection::Right,
                    &self.field,
                    self.tetris_config.move_reset_limit,
                );
            }
            MovementCommand::Down => {
                self.current_mino.down(&self.field);
            }
            MovementCommand::RotateClockWise => {
                self.current_mino.rotate(
                    RotationType::Clockwise,
                    &self.field,
                    self.tetris_config.move_reset_limit,
                );
            }
            MovementCommand::RotateCounterClockWise => {
                self.current_mino.rotate(
                    RotationType::CounterClockwise,
                    &self.field,
                    self.tetris_config.move_reset_limit,
                );
            }
            MovementCommand::Rotate180 => {
                self.current_mino.rotate(
                    RotationType::Rotate180,
                    &self.field,
                    self.tetris_config.move_reset_limit,
                );
            }
            MovementCommand::Hold => {
                if !self.has_held || self.tetris_config.can_hold_infinity {
                    self.has_held = true;
                    self.mino_queue.hold();
                    if self.spawn_current_mino().is_err() {
                        return (Err(()), None);
                    }
                }
            }
            MovementCommand::HardDrop => {
                while self.current_mino.down(&self.field) {}
                return self.command(MovementCommand::Lock);
            }
            MovementCommand::Lock => {
                self.current_mino.lock(&mut self.field);
                self.has_held = false;

                let mut cleared_line_count = 0;
                for (y, row) in self.field.clone().iter().enumerate() {
                    if row.iter().all(|cell| cell.can_be_cleared()) {
                        self.delete_line(y);
                        self.current_mino.y += 1; //一緒に落ちる
                        cleared_line_count += 1;
                    }
                }
                let line_clear = LineClear {
                    cleared_line_count: if cleared_line_count == 0 {
                        None
                    } else {
                        Some(cleared_line_count)
                    },
                    is_perfect: self
                        .field
                        .iter()
                        .all(|row| row.iter().all(|cell| !cell.has_collision())),
                    mino_type: self.current_mino.mino_type,
                    is_spin: self.current_mino.is_last_move_spin,
                    is_spin_mini: self.current_mino.is_last_move_mini_spin,
                };
                self.mino_queue.next();

                self.release_stock_attacked_line();
                return if self.spawn_current_mino().is_err() {
                    (Err(()), Some(line_clear))
                } else {
                    (Ok(self.current_mino.mino_state), Some(line_clear))
                };
            }
            MovementCommand::Attacked(attacked_line) => {
                self.attacked_lines_stock.push(attacked_line);
            }
        };
        if let Some(r) = self.lock_check() {
            return r;
        }
        (Ok(self.current_mino.mino_state), None)
    }
    pub fn spawn_current_mino(&mut self) -> Result<(), ()> {
        let next_pos = self.get_spawn_point(self.mino_queue.current);
        self.current_mino =
            Mino::new(self.mino_queue.current, next_pos.0, next_pos.1, &self.field)?;
        Ok(())
    }

    pub fn get_mino_state(&mut self) -> MinoState {
        self.current_mino.check_mino_status();
        self.current_mino.mino_state
    }
    fn delete_line(&mut self, index: usize) {
        self.field[0..=index].rotate_right(1);
        #[cfg(feature = "alloc")]
        {
            self.field[0] = alloc::vec![Cell::Empty; self.width];
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.field[0] = {
                let mut field = Vec::new();
                for _ in 0..DEFAULT_BOARD_SIZE.0 {
                    field.push(Cell::Empty);
                }
                field
            };
        }
    }
    pub fn release_stock_attacked_line(&mut self) {
        for attacked_line in &self.attacked_lines_stock {
            self.field.rotate_left(1);
            #[cfg(feature = "alloc")]
            let mut inserted_line =
                alloc::vec![Cell::Obstruction(attacked_line.can_be_cleared); self.width];
            #[cfg(not(feature = "alloc"))]
            let mut inserted_line = {
                let mut line = Vec::new();
                for _ in 0..DEFAULT_BOARD_SIZE.0 {
                    line.push(Cell::Obstruction(attacked_line.can_be_cleared));
                }
                line
            };
            if let Some(hole_indexes) = &attacked_line.hole_indexes {
                for hole_index in hole_indexes {
                    inserted_line[*hole_index] = Cell::Empty;
                }
            }
            *self.field.last_mut().unwrap() = inserted_line;
        }
        self.attacked_lines_stock.clear();
    }
    pub fn get_next_minos(&mut self, num: usize) -> NextsField {
        self.mino_queue.get_next_minos(num)
    }
    pub fn get_hold_mino(&self) -> Option<MinoType> {
        self.mino_queue.hold
    }
    pub fn get_spawn_point(&self, mino_type: MinoType) -> (i64, i64) {
        (
            self.tetris_config.appearance_position.0,
            self.tetris_config.appearance_position.1 + if mino_type == MinoO { -1 } else { 0 },
        )
    }
    pub fn get_minimum_y(&self) -> usize {
        for (y, row) in self.field.iter().enumerate() {
            if row.iter().any(|cell| cell.has_collision()) {
                return y;
            }
        }
        self.height
    }
}
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum MinoState {
    AirBorne,   //空中にある状態(timerを停止)
    JustLanded, //地面に接触した瞬間もしくは接触中に移動してなお接触中(timerのreset)
    Grounded,   //地面に接触中
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MovementCommand {
    Left,
    Right,
    Down,
    RotateClockWise,
    Rotate180,
    RotateCounterClockWise,
    Hold,
    Lock,
    HardDrop,
    Attacked(AttackedLine),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LineClear {
    cleared_line_count: Option<usize>,
    is_perfect: bool,
    mino_type: MinoType,
    is_spin: bool,
    is_spin_mini: bool,
}
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttackedLine {
    #[cfg(feature = "alloc")]
    pub hole_indexes: Option<Vec<usize>>,
    #[cfg(not(feature = "alloc"))]
    pub hole_indexes: Option<Vec<usize, { DEFAULT_BOARD_SIZE.0 }>>,
    pub can_be_cleared: bool,
}
