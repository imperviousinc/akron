pub const FONT: iced::Font = iced::Font::with_name("icons");
pub enum Icon {
    Settings,
    WalletMinimal,
    RotateCcwKey,
    Copy,
    CircleDot,
    ArrowBigUpDash,
    ChevronDown,
    ChevronRight,
    FolderDown,
    ArrowBigDownDash,
    ArrowsUpFromLine,
    Store,
    ChevronLeft,
    UserRoundPen,
    Bitcoin,
    Circle,
    Bolt,
    AtSign,
}
impl Icon {
    pub fn as_char(&self) -> char {
        match self {
            Icon::Settings => '\u{E000}',
            Icon::WalletMinimal => '\u{E001}',
            Icon::RotateCcwKey => '\u{E002}',
            Icon::Copy => '\u{E003}',
            Icon::CircleDot => '\u{E004}',
            Icon::ArrowBigUpDash => '\u{E005}',
            Icon::ChevronDown => '\u{E006}',
            Icon::ChevronRight => '\u{E007}',
            Icon::FolderDown => '\u{E008}',
            Icon::ArrowBigDownDash => '\u{E009}',
            Icon::ArrowsUpFromLine => '\u{E00A}',
            Icon::Store => '\u{E00B}',
            Icon::ChevronLeft => '\u{E00C}',
            Icon::UserRoundPen => '\u{E00D}',
            Icon::Bitcoin => '\u{E00E}',
            Icon::Circle => '\u{E00F}',
            Icon::Bolt => '\u{E010}',
            Icon::AtSign => '\u{E011}',
        }
    }
}
