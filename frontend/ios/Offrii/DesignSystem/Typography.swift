import SwiftUI

// MARK: - Typography Scale (SF Pro with Dynamic Type)

enum OffriiTypography {
    static let displayLarge = Font.system(.largeTitle, weight: .bold)
    static let largeTitle = Font.system(.largeTitle, weight: .bold)
    static let titleLarge = Font.system(.title, weight: .bold)
    static let title = Font.system(.title, weight: .bold)
    static let titleMedium = Font.system(.title2, weight: .bold)
    static let title2 = Font.system(.title2, weight: .bold)
    static let titleSmall = Font.system(.title3, weight: .semibold)
    static let title3 = Font.system(.title3, weight: .semibold)
    static let headline = Font.system(.headline, weight: .semibold)
    static let body = Font.system(.body, weight: .regular)
    static let callout = Font.system(.callout, weight: .regular)
    static let subheadline = Font.system(.subheadline, weight: .regular)
    static let footnote = Font.system(.footnote, weight: .regular)
    static let caption = Font.system(.caption, weight: .regular)
    static let caption2 = Font.system(.caption2, weight: .regular)
    static let captionSmall = Font.system(.caption2, weight: .regular)
}

// MARK: - Text Style Modifiers

extension View {
    func offriiTextStyle(_ font: Font, color: Color = OffriiTheme.text) -> some View {
        self
            .font(font)
            .foregroundColor(color)
    }
}
