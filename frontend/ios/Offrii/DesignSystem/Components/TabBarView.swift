import SwiftUI

// MARK: - Tab Item Enum

enum TabItem: Int, CaseIterable, Identifiable {
    case envies
    case cercles
    case entraide
    case profil

    var id: Int { rawValue }

    var label: LocalizedStringKey {
        switch self {
        case .envies:   return "Envies"
        case .cercles:  return "Cercles"
        case .entraide: return "Entraide"
        case .profil:   return "Profil"
        }
    }

    var iconName: String {
        switch self {
        case .envies:   return "heart.fill"
        case .cercles:  return "person.2.fill"
        case .entraide: return "hand.raised.fill"
        case .profil:   return "person.fill"
        }
    }
}

// MARK: - TabBarView

struct TabBarView: View {
    @Binding var selectedTab: TabItem

    var body: some View {
        HStack {
            ForEach(TabItem.allCases) { tab in
                tabButton(for: tab)
            }
        }
        .padding(.top, OffriiTheme.spacingSM)
        .padding(.bottom, OffriiTheme.spacingXS)
        .background(
            OffriiTheme.card
                .shadow(
                    color: Color.black.opacity(0.08),
                    radius: 8,
                    x: 0,
                    y: -2
                )
                .ignoresSafeArea(edges: .bottom)
        )
    }

    // MARK: - Tab Button

    private func tabButton(for tab: TabItem) -> some View {
        Button {
            withAnimation(OffriiTheme.defaultAnimation) {
                selectedTab = tab
            }
        } label: {
            VStack(spacing: 4) {
                Image(systemName: tab.iconName)
                    .font(.system(size: 22))
                    .foregroundColor(tabColor(for: tab))

                Text(tab.label)
                    .font(OffriiTypography.caption2)
                    .foregroundColor(tabColor(for: tab))
            }
            .frame(maxWidth: .infinity)
        }
        .buttonStyle(.plain)
        .accessibilityLabel(tab.label)
    }

    // MARK: - Tab Color

    private func tabColor(for tab: TabItem) -> Color {
        selectedTab == tab ? OffriiTheme.primary : OffriiTheme.textMuted
    }
}

// MARK: - Preview

#if DEBUG
struct TabBarView_Previews: PreviewProvider {
    struct PreviewWrapper: View {
        @State private var selectedTab: TabItem = .envies

        var body: some View {
            VStack {
                Spacer()

                Text("Onglet : \(selectedTab.iconName)")
                    .font(OffriiTypography.title2)
                    .foregroundColor(OffriiTheme.text)

                Spacer()

                TabBarView(selectedTab: $selectedTab)
            }
            .background(OffriiTheme.cardSurface)
        }
    }

    static var previews: some View {
        PreviewWrapper()
            .previewLayout(.device)
    }
}
#endif
