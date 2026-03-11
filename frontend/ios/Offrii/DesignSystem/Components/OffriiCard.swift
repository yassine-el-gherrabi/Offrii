import SwiftUI

// MARK: - OffriiCard

struct OffriiCard<Content: View>: View {
    let showAccentBar: Bool
    let showBorder: Bool
    let content: Content

    init(
        showAccentBar: Bool = false,
        showBorder: Bool = false,
        @ViewBuilder content: () -> Content
    ) {
        self.showAccentBar = showAccentBar
        self.showBorder = showBorder
        self.content = content()
    }

    var body: some View {
        HStack(spacing: 0) {
            if showAccentBar {
                RoundedRectangle(cornerRadius: 2)
                    .fill(OffriiTheme.primary)
                    .frame(width: 4)
            }

            content
                .padding(OffriiTheme.spacingBase)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusLG)
        .overlay(
            Group {
                if showBorder {
                    RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusLG)
                        .strokeBorder(OffriiTheme.primaryLight, lineWidth: 1)
                }
            }
        )
        .shadow(
            color: OffriiTheme.cardShadowColor,
            radius: OffriiTheme.cardShadowRadius,
            x: 0,
            y: OffriiTheme.cardShadowY
        )
    }
}

// MARK: - Card Modifier (alternative API)

struct OffriiCardModifier: ViewModifier {
    func body(content: Content) -> some View {
        content
            .padding(OffriiTheme.spacingBase)
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .shadow(
                color: OffriiTheme.cardShadowColor,
                radius: OffriiTheme.cardShadowRadius,
                x: 0,
                y: OffriiTheme.cardShadowY
            )
    }
}

extension View {
    func offriiCard() -> some View {
        modifier(OffriiCardModifier())
    }
}
