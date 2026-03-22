use stitch_core::RgbaColor;

pub fn parse_rgba_color(input: &str) -> Result<RgbaColor, String> {
    let value = input.strip_prefix('#').unwrap_or(input);

    let color = match value.len() {
        6 => RgbaColor {
            r: parse_byte(&value[0..2])?,
            g: parse_byte(&value[2..4])?,
            b: parse_byte(&value[4..6])?,
            a: 255,
        },
        8 => RgbaColor {
            r: parse_byte(&value[0..2])?,
            g: parse_byte(&value[2..4])?,
            b: parse_byte(&value[4..6])?,
            a: parse_byte(&value[6..8])?,
        },
        _ => return Err("expected color in #RRGGBB or #RRGGBBAA format".to_string()),
    };

    Ok(color)
}

fn parse_byte(input: &str) -> Result<u8, String> {
    u8::from_str_radix(input, 16).map_err(|_| "invalid hex color component".to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_rgba_color;
    use stitch_core::RgbaColor;

    #[test]
    fn parses_rgb_hex_with_hash() {
        assert_eq!(
            parse_rgba_color("#aabbcc").unwrap(),
            RgbaColor {
                r: 0xaa,
                g: 0xbb,
                b: 0xcc,
                a: 0xff,
            }
        );
    }

    #[test]
    fn parses_rgba_hex_without_hash() {
        assert_eq!(
            parse_rgba_color("01020304").unwrap(),
            RgbaColor {
                r: 0x01,
                g: 0x02,
                b: 0x03,
                a: 0x04,
            }
        );
    }

    #[test]
    fn rejects_invalid_length() {
        assert_eq!(
            parse_rgba_color("#12345").unwrap_err(),
            "expected color in #RRGGBB or #RRGGBBAA format"
        );
    }

    #[test]
    fn rejects_invalid_hex() {
        assert_eq!(
            parse_rgba_color("#gg0000").unwrap_err(),
            "invalid hex color component"
        );
    }
}
