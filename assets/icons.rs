pub const FONT: iced::Font = iced::Font::with_name("icons");
pub enum Icon {
    Settings,
    WalletMinimal,
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
            Icon::Copy => '\u{E002}',
            Icon::CircleDot => '\u{E003}',
            Icon::ArrowBigUpDash => '\u{E004}',
            Icon::ChevronDown => '\u{E005}',
            Icon::ChevronRight => '\u{E006}',
            Icon::FolderDown => '\u{E007}',
            Icon::ArrowBigDownDash => '\u{E008}',
            Icon::ArrowsUpFromLine => '\u{E009}',
            Icon::Store => '\u{E00A}',
            Icon::ChevronLeft => '\u{E00B}',
            Icon::UserRoundPen => '\u{E00C}',
            Icon::Bitcoin => '\u{E00D}',
            Icon::Circle => '\u{E00E}',
            Icon::Bolt => '\u{E00F}',
            Icon::AtSign => '\u{E010}',
        }
    }
}
