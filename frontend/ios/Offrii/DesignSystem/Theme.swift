import SwiftUI

// MARK: - Design Tokens

enum OffriiTheme {

    // MARK: Colors

    static let primary = Color(hex: "#3B2FE0")
    static let accent = Color(hex: "#F59E0B")
    static let card = Color.white
    static let cardSurface = Color(hex: "#F4F4F8")
    static let success = Color(hex: "#10B981")
    static let danger = Color(hex: "#EF4444")
    static let text = Color(hex: "#1A1A2E")
    static let textSecondary = Color(hex: "#6B7280")
    static let textMuted = Color(hex: "#9CA3AF")
    static let border = Color(hex: "#E8E8EE")

    // MARK: Spacing

    static let spacingXS: CGFloat = 4
    static let spacingSM: CGFloat = 8
    static let spacingMD: CGFloat = 16
    static let spacingLG: CGFloat = 24
    static let spacingXL: CGFloat = 32
    static let spacingXXL: CGFloat = 48

    // MARK: Corner Radius

    static let cornerRadiusSM: CGFloat = 8
    static let cornerRadiusMD: CGFloat = 14
    static let cornerRadiusLG: CGFloat = 20
    static let cornerRadiusXL: CGFloat = 30

    // MARK: Shadows

    static let cardShadowColor = Color.black.opacity(0.06)
    static let cardShadowRadius: CGFloat = 12
    static let cardShadowY: CGFloat = 4

    // MARK: Animation

    static let defaultAnimation: Animation = .easeInOut(duration: 0.2)
}
