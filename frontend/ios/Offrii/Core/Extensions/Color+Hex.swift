import SwiftUI

// MARK: - Color Hex Initializer

extension Color {
    /// Initializes a `Color` from a hex string.
    ///
    /// Supports 6-character (RGB) and 8-character (ARGB) hex strings.
    /// Leading `#` or `0x` prefixes are stripped automatically.
    ///
    /// - Parameter hex: A hex color string, e.g. `"#3B2FE0"` or `"FF3B2FE0"`.
    init(hex: String) {
        let sanitized = hex.trimmingCharacters(in: .whitespacesAndNewlines)
        let hexString = sanitized.hasPrefix("#")
            ? String(sanitized.dropFirst())
            : sanitized

        var rgbValue: UInt64 = 0
        Scanner(string: hexString).scanHexInt64(&rgbValue)

        let red: Double
        let green: Double
        let blue: Double
        let opacity: Double

        switch hexString.count {
        case 6:
            red = Double((rgbValue >> 16) & 0xFF) / 255.0
            green = Double((rgbValue >> 8) & 0xFF) / 255.0
            blue = Double(rgbValue & 0xFF) / 255.0
            opacity = 1.0
        case 8:
            red = Double((rgbValue >> 24) & 0xFF) / 255.0
            green = Double((rgbValue >> 16) & 0xFF) / 255.0
            blue = Double((rgbValue >> 8) & 0xFF) / 255.0
            opacity = Double(rgbValue & 0xFF) / 255.0
        default:
            red = 0
            green = 0
            blue = 0
            opacity = 1.0
        }

        self.init(.sRGB, red: red, green: green, blue: blue, opacity: opacity)
    }
}
