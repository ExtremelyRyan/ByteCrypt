pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Custom(String),
}

#[derive(Clone, Copy)]
pub struct CharacterSet {
    pub h_line: char,
    pub v_line: char,
    pub joint: char,
    pub node: char,
}

#[allow(dead_code)]
impl CharacterSet {
    pub const U8_SLINE: Self = Self {
        h_line: '─',
        v_line: '│',
        joint: '├',
        node: '└',
    };

    pub const U8_SLINE_BOLD: Self = Self {
        h_line: '━',
        v_line: '┃',
        joint: '┣',
        node: '┗',
    };

    pub const U8_SLINE_CURVE: Self = Self {
        h_line: '─',
        v_line: '│',
        joint: '├',
        node: '╰',
    };

    pub const U8_DLINE: Self = Self {
        h_line: '═',
        v_line: '║',
        joint: '╠',
        node: '╚',
    };

    pub const ASCII_SLINE: Self = Self {
        h_line: '-',
        v_line: '|',
        joint: '|',
        node: '`',
    };
}
