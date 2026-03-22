import SwiftUI

// MARK: - Shadow Definitions

enum OffriiShadow {
    struct ShadowStyle {
        let color: Color
        let radius: CGFloat
        let x: CGFloat
        let y: CGFloat
    }

    static func sm(scheme: ColorScheme = .light) -> ShadowStyle {
        ShadowStyle(
            color: .black.opacity(scheme == .dark ? 0.25 : 0.06),
            radius: 4, x: 0, y: 2
        )
    }

    static func md(scheme: ColorScheme = .light) -> ShadowStyle {
        ShadowStyle(
            color: .black.opacity(scheme == .dark ? 0.35 : 0.10),
            radius: 12, x: 0, y: 4
        )
    }

    static func lg(scheme: ColorScheme = .light) -> ShadowStyle {
        ShadowStyle(
            color: .black.opacity(scheme == .dark ? 0.45 : 0.15),
            radius: 24, x: 0, y: 8
        )
    }

    static func fab(scheme: ColorScheme = .light) -> ShadowStyle {
        ShadowStyle(
            color: OffriiTheme.primary.opacity(scheme == .dark ? 0.3 : 0.35),
            radius: 16, x: 0, y: 6
        )
    }
}

// MARK: - Shadow View Modifier

struct OffriiShadowModifier: ViewModifier {
    let style: OffriiShadow.ShadowStyle

    func body(content: Content) -> some View {
        content
            .shadow(color: style.color, radius: style.radius, x: style.x, y: style.y)
    }
}

extension View {
    func offriiShadow(_ style: OffriiShadow.ShadowStyle) -> some View {
        modifier(OffriiShadowModifier(style: style))
    }
}
