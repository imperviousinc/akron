pub use spaces_protocol::slabel::SLabel;
pub use spaces_wallet::{
    Listing,
    bitcoin::{Amount, FeeRate},
};

pub fn is_slabel_input(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase() || c == '-')
}

pub fn slabel_from_str(s: &str) -> Option<SLabel> {
    SLabel::from_str_unprefixed(s)
        .ok()
        .filter(|slabel| !slabel.is_reserved())
}

pub fn is_recipient_input(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase() || c == '-' || c == '@')
}

pub fn recipient_from_str(s: &str) -> Option<String> {
    // TODO: check
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

pub fn is_amount_input(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}

pub fn amount_from_str(s: &str) -> Option<Amount> {
    Amount::from_str_in(s, spaces_wallet::bitcoin::Denomination::Satoshi).ok()
}

pub fn is_fee_rate_input(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}

pub fn fee_rate_from_str(s: &str) -> Option<Option<FeeRate>> {
    if s.is_empty() {
        Some(None)
    } else {
        s.parse().ok().map(FeeRate::from_sat_per_vb)
    }
}

pub fn listing_from_str(s: &str) -> Option<Listing> {
    serde_json::from_str(s).ok()
}

pub fn format_amount_number(mut n: u64) -> String {
    if n == 0 {
        return "0 sat".to_string();
    }

    let mut digits = Vec::new();
    while n > 0 {
        digits.push((n % 10) as u8);
        n /= 10;
    }

    let l = digits.len();
    let mut result = String::with_capacity(l + (l - 1) / 3 + 4);

    for (i, &digit) in digits.iter().rev().enumerate() {
        if i > 0 && (l - i) % 3 == 0 {
            result.push('\u{2009}');
        }
        result.push(char::from_digit(digit as u32, 10).unwrap());
    }

    result.push_str(" sat");
    result
}

pub fn format_amount(amount: crate::helpers::Amount) -> String {
    format_amount_number(amount.to_sat())
}

pub fn height_to_future_est(block_height: u32, tip_height: u32) -> String {
    if block_height <= tip_height {
        return "now".to_string();
    }

    let remaining_blocks = block_height - tip_height;

    if remaining_blocks <= 5 {
        return format!("in {} minutes", remaining_blocks * 10);
    }

    if remaining_blocks <= 144 {
        let hours = remaining_blocks / 6;
        let remaining_blocks = remaining_blocks % 6;
        let minutes = remaining_blocks * 10;
        if minutes == 0 {
            return format!("in {} hours", hours);
        }
        return format!("in {} hours {} minutes", hours, minutes);
    }

    let days = remaining_blocks / 144;
    let remaining_blocks = remaining_blocks % 144;
    let hours = remaining_blocks / 6;

    if hours == 0 {
        return format!("in {} days", days);
    }
    format!("in {} days {} hours", days, hours)
}

pub fn height_to_past_est(block_height: u32, tip_height: u32) -> String {
    if block_height >= tip_height {
        return "just now".to_string();
    }

    let remaining_blocks = tip_height - block_height;

    if remaining_blocks <= 5 {
        return format!("{} minutes ago", remaining_blocks * 10);
    }

    if remaining_blocks <= 144 {
        let hours = (remaining_blocks + 3) / 6;
        return format!("{} hours ago", hours);
    }

    let days = (remaining_blocks + 72) / 144;
    format!("{} days ago", days)
}
