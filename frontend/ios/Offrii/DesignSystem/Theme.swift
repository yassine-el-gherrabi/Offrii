import SwiftUI

// MARK: - Design Tokens

enum OffriiTheme {

    // MARK: - Primary Colors

    static let primary = Color("offriiPrimary", bundle: nil)
    static let primaryLight = Color("primaryLight", bundle: nil)
    static let primaryDark = Color("primaryDark", bundle: nil)
    static let secondary = Color("offriiSecondary", bundle: nil)
    static let secondaryLight = Color("secondaryLight", bundle: nil)
    static let accent = Color("offriiAccent", bundle: nil)
    static let accentLight = Color("accentLight", bundle: nil)

    // MARK: - Semantic Colors

    static let success = Color("success", bundle: nil)
    static let successLight = Color("successLight", bundle: nil)
    static let warning = Color("warning", bundle: nil)
    static let warningLight = Color("warningLight", bundle: nil)
    static let danger = Color("danger", bundle: nil)
    static let dangerLight = Color("dangerLight", bundle: nil)

    // MARK: - Surface Colors

    static let background = Color("background", bundle: nil)
    static let surface = Color("surface", bundle: nil)
    static let card = Color("card", bundle: nil)
    static let elevated = Color("elevated", bundle: nil)

    // MARK: - Text Colors

    static let text = Color("text", bundle: nil)
    static let textSecondary = Color("textSecondary", bundle: nil)
    static let textMuted = Color("textMuted", bundle: nil)
    static let textInverse = Color("textInverse", bundle: nil)
    static let border = Color("border", bundle: nil)
    static let borderFocused = Color("borderFocused", bundle: nil)

    // MARK: - Category Colors

    static let categoryEducationBg = Color("categoryEducationBg", bundle: nil)
    static let categoryEducationText = Color("categoryEducationText", bundle: nil)
    static let categoryClothingBg = Color("categoryClothingBg", bundle: nil)
    static let categoryClothingText = Color("categoryClothingText", bundle: nil)
    static let categoryHealthBg = Color("categoryHealthBg", bundle: nil)
    static let categoryHealthText = Color("categoryHealthText", bundle: nil)
    static let categoryReligionBg = Color("categoryReligionBg", bundle: nil)
    static let categoryReligionText = Color("categoryReligionText", bundle: nil)
    static let categoryHomeBg = Color("categoryHomeBg", bundle: nil)
    static let categoryHomeText = Color("categoryHomeText", bundle: nil)
    static let categoryChildrenBg = Color("categoryChildrenBg", bundle: nil)
    static let categoryChildrenText = Color("categoryChildrenText", bundle: nil)
    static let categoryOtherBg = Color("categoryOtherBg", bundle: nil)
    static let categoryOtherText = Color("categoryOtherText", bundle: nil)

    // MARK: - Legacy Aliases (backward compatibility during migration)

    static let cardSurface = surface

    // MARK: - Spacing

    static let spacingXXXS: CGFloat = 2
    static let spacingXXS: CGFloat = 4
    static let spacingXS: CGFloat = 6
    static let spacingSM: CGFloat = 8
    static let spacingMD: CGFloat = 12
    static let spacingBase: CGFloat = 16
    static let spacingLG: CGFloat = 20
    static let spacingXL: CGFloat = 24
    static let spacingXXL: CGFloat = 32
    static let spacingXXXL: CGFloat = 40
    static let spacingHuge: CGFloat = 48
    static let spacingGiant: CGFloat = 64

    // MARK: - Corner Radius

    static let cornerRadiusXS: CGFloat = 4
    static let cornerRadiusSM: CGFloat = 8
    static let cornerRadiusMD: CGFloat = 12
    static let cornerRadiusLG: CGFloat = 16
    static let cornerRadiusXL: CGFloat = 20
    static let cornerRadiusXXL: CGFloat = 28
    static let cornerRadiusFull: CGFloat = 9999

    // MARK: - Shadows

    struct ShadowStyle {
        let color: Color
        let radius: CGFloat
        let x: CGFloat
        let y: CGFloat
    }

    static func shadowSM(_ scheme: ColorScheme = .light) -> ShadowStyle {
        let opacity = scheme == .dark ? 0.25 : 0.06
        return ShadowStyle(color: Color.black.opacity(opacity), radius: 4, x: 0, y: 2)
    }

    static func shadowMD(_ scheme: ColorScheme = .light) -> ShadowStyle {
        let opacity = scheme == .dark ? 0.35 : 0.10
        return ShadowStyle(color: Color.black.opacity(opacity), radius: 12, x: 0, y: 4)
    }

    static func shadowLG(_ scheme: ColorScheme = .light) -> ShadowStyle {
        let opacity = scheme == .dark ? 0.45 : 0.15
        return ShadowStyle(color: Color.black.opacity(opacity), radius: 24, x: 0, y: 8)
    }

    static let cardShadowColor = Color.black.opacity(0.10)
    static let cardShadowRadius: CGFloat = 12
    static let cardShadowY: CGFloat = 4

    // MARK: - Animation

    static let defaultAnimation: Animation = .spring(response: 0.35, dampingFraction: 0.7)
}
