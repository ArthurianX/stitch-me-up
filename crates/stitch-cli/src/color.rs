use stitch_core::RgbaColor;

pub fn parse_rgba_color(input: &str) -> Result<RgbaColor, String> {
    RgbaColor::parse_hex(input).map_err(|error| error.to_string())
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
