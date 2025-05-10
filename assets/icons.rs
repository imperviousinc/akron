pub const FONT: iced::Font = iced::Font::with_name("icons");
pub enum Icon {
    Assembly,
    Settings,
    CurrencyBitcoin,
    Copy,
    BuildingBank,
    ArrowBigUpLines,
    FolderDown,
    ArrowDownToArc,
    ArrowDownFromArc,
    Signature,
    ChevronLeft,
    At,
    NewSection,
}
impl Icon {
    pub fn as_char(&self) -> char {
        match self {
            Icon::Assembly => '\u{E000}',
            Icon::Settings => '\u{E001}',
            Icon::CurrencyBitcoin => '\u{E002}',
            Icon::Copy => '\u{E003}',
            Icon::BuildingBank => '\u{E004}',
            Icon::ArrowBigUpLines => '\u{E005}',
            Icon::FolderDown => '\u{E007}',
            Icon::ArrowDownToArc => '\u{E008}',
            Icon::ArrowDownFromArc => '\u{E009}',
            Icon::Signature => '\u{E00A}',
            Icon::ChevronLeft => '\u{E00B}',
            Icon::At => '\u{E00C}',
            Icon::NewSection => '\u{E00D}',
        }
    }
}
